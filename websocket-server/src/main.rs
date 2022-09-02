use config::Config;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, sync::Arc};
use syslog::{Facility, Formatter3164, LoggerBackend};
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{ws::Message, ws::WebSocket, Filter};

type Sender = mpsc::UnboundedSender<Result<Message, warp::Error>>;

#[derive(Debug, Clone)]
struct Client {
    sender: Option<Sender>,
    permissions: Vec<String>,
    ready: bool,
}

impl Client {
    fn new(sender: Sender) -> Self {
        Self {
            sender: Some(sender),
            permissions: vec![],
            ready: false,
        }
    }
}

struct State {
    clients: Mutex<HashMap<Uuid, Client>>,
    subscriptions: Mutex<HashMap<String, Vec<Uuid>>>,
    syslog: Mutex<syslog::Logger<LoggerBackend, Formatter3164>>,
}

impl State {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            clients: Mutex::new(HashMap::new()),
            subscriptions: Mutex::new(HashMap::new()),
            syslog: Mutex::new(State::init_syslog()),
        })
    }

    fn init_syslog() -> syslog::Logger<LoggerBackend, Formatter3164> {
        let formatter = Formatter3164 {
            facility: Facility::LOG_LOCAL7,
            process: "Ufo-Websocket".to_string(),
            hostname: None,
            ..Default::default()
        };

        syslog::unix(formatter).expect("Failed to connect to syslog")
    }

    async fn new_client(&self, sender: Sender) -> Uuid {
        let uuid = Uuid::new_v4();
        let mut locked_clients = self.clients.lock().await;
        locked_clients.insert(uuid, Client::new(sender));
        self.log_info(&format!("Client {} connected", uuid)).await;
        if let Some(client) = locked_clients.get(&uuid) {
            if let Some(sender) = &client.sender {
                self.log_error(
                    UfoMessage::Uid { uid: uuid }.send(sender),
                    "Sending uid to client",
                )
                .await;
            }
        }
        uuid
    }

    async fn set_permission(&self, client_id: Uuid, permissions: &Vec<&str>) {
        let mut locked_clients = self.clients.lock().await;
        if let Some(client) = locked_clients.get_mut(&client_id) {
            for permission in permissions {
                client.permissions.push(permission.to_string());
            }
            self.log_debug(&format!(
                "Client {} received permissions: {:?}",
                client_id, permissions
            ))
            .await;
            if !client.ready {
                if let Some(sender) = &client.sender {
                    self.log_error(UfoMessage::Ready.send(sender), "Sending ready to client")
                        .await;
                }
                client.ready = true;
            }
        }
    }

    async fn add_subscription(&self, client_id: Uuid, channel: &str) {
        /* Important:
         * if any other function needs to lock both subscriptions and clients,
         * subscriptions need to be locked first to prevent deadlocks.
         */
        let mut locked_subscriptions = self.subscriptions.lock().await;
        let locked_clients = self.clients.lock().await;
        if let Some(client) = locked_clients.get(&client_id) {
            if client.permissions.iter().any(|p| p == channel) {
                if let Some(client_list) = locked_subscriptions.get_mut(channel) {
                    client_list.push(client_id);
                } else {
                    locked_subscriptions.insert(channel.to_string(), vec![client_id]);
                }
                self.log_debug(&format!("Client {} subscribed to {:?}", client_id, channel))
                    .await;
            } else {
                self.log_notice(&format!(
                    "Client {} tried subscribing to {:?} but didn't have permission",
                    client_id, channel
                ))
                .await;
            }
        }
    }

    async fn reset_subscription(&self, client_id: Uuid) {
        let mut locked_subscriptions = self.subscriptions.lock().await;
        for (_channel, client_list) in locked_subscriptions.iter_mut() {
            client_list.retain(|uuid| uuid != &client_id);
        }
    }

    async fn client_disconnect(&self, client_id: Uuid) {
        self.reset_subscription(client_id).await;
        self.clients.lock().await.remove(&client_id);
        self.log_info(&format!("Client {} disconnected", client_id))
            .await;
    }

    async fn broadcast_message(&self, channel: &str, message: &str) {
        /* Important:
         * if any other function needs to lock both subscriptions and clients,
         * subscriptions need to be locked first to prevent deadlocks.
         */
        let locked_subscriptions = self.subscriptions.lock().await;
        if let Some(client_list) = locked_subscriptions.get(channel) {
            let locked_clients = self.clients.lock().await;
            let msg = match (UfoMessage::Broadcast { channel, message }).output() {
                Ok(msg) => msg,
                Err(e) => {
                    self.log_error_raw(e, "Building broadcast").await;
                    return;
                }
            };
            self.log_debug(&format!(
                "Broadcasting to {} clients on channel {:?}",
                client_list.len(),
                channel
            ))
            .await;
            for client_id in client_list {
                if let Some(client) = locked_clients.get(client_id) {
                    if let Some(sender) = &client.sender {
                        self.log_error(
                            sender.send(Ok(Message::text(&msg))),
                            &format!("Broadcasting to clients on channel {:?}", channel),
                        )
                        .await;
                    }
                }
            }
        }
    }

    async fn log_error(&self, res: Result<(), impl Into<UfoError>>, context: &str) {
        if let Err(e) = res {
            self.log_error_raw(e.into(), context).await;
        }
    }

    async fn log_error_raw(&self, err: UfoError, context: &str) {
        let err_string = format!("{}: {:?}", context, err);
        self.syslog
            .lock()
            .await
            .err(&err_string)
            .unwrap_or_else(|e| {
                println!("Error writing to syslog: {}\n[Error]: {}", e, err_string)
            });
    }

    async fn log_info(&self, msg: &str) {
        self.syslog
            .lock()
            .await
            .info(msg)
            .unwrap_or_else(|e| println!("Error writing to syslog: {}\n[Info]: {}", e, msg));
    }

    async fn log_debug(&self, msg: &str) {
        self.syslog
            .lock()
            .await
            .debug(msg)
            .unwrap_or_else(|e| println!("Error writing to syslog: {}\n[Debug]: {}", e, msg));
    }

    async fn log_notice(&self, msg: &str) {
        self.syslog
            .lock()
            .await
            .notice(msg)
            .unwrap_or_else(|e| println!("Error writing to syslog: {}\n[Notice]: {}", e, msg));
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum UfoMessage<'a> {
    Permission {
        uid: Uuid,
        permissions: Vec<&'a str>,
    },
    Message {
        channel: &'a str,
        message: &'a str,
    },
    Broadcast {
        channel: &'a str,
        message: &'a str,
    },
    Uid {
        uid: Uuid,
    },
    Subscribe {
        channel: &'a str,
    },
    Ready,
}

impl<'a> UfoMessage<'a> {
    fn output(&self) -> Result<String, UfoError> {
        Ok(serde_json::to_string(self)?)
    }

    fn send(&self, sender: &Sender) -> Result<(), UfoError> {
        Ok(sender.send(Ok(Message::text(self.output()?)))?)
    }
}

#[derive(Debug)]
enum UfoError {
    Known(String),
}

impl<T> From<T> for UfoError
where
    T: std::error::Error + 'static,
{
    fn from(e: T) -> Self {
        Self::Known(format!("{:?}", e))
    }
}

#[tokio::main]
async fn main() {
    let state = State::new();

    let config_builder = Config::builder().add_source(config::File::with_name("config"));

    let (host, frontend_port, backend_port) = match config_builder.build() {
        Ok(config) => (
            config.get("host").unwrap_or_else(|_| [127, 0, 0, 1].into()),
            config.get("port_frontend").unwrap_or(8080),
            config.get("port_backend").unwrap_or(8081),
        ),
        Err(e) => {
            state
                .log_notice(&format!(
                    "Failed reading config file. Using default values. (Error: {:?})",
                    e
                ))
                .await;
            ([127, 0, 0, 1].into(), 8080, 8081)
        }
    };

    let backend = backend_server(state.clone(), (host, backend_port));

    let state_clone = state.clone();
    let frontend_websocket = warp::ws().map(move |ws: warp::ws::Ws| {
        let state = state_clone.clone();
        ws.on_upgrade(move |socket| client_connection(socket, state))
    });

    state
        .log_info(&format!("Frontend listening on {}:{}", host, frontend_port))
        .await;
    tokio::join!(
        warp::serve(frontend_websocket).run((host, frontend_port)),
        backend
    );

    state.log_info("Program ended").await;
}

async fn backend_server(state: Arc<State>, addr: (IpAddr, u16)) {
    match TcpListener::bind(addr).await {
        Ok(listener) => {
            state
                .log_info(&format!("Backend listening on {}:{}", addr.0, addr.1))
                .await;
            loop {
                state
                    .log_error(backend_listen(&listener, &state).await, "Failed to listen")
                    .await;
            }
        }
        Err(e) => state.log_error_raw(e.into(), "Failed to bind").await,
    }
}

async fn backend_listen(listener: &TcpListener, state: &Arc<State>) -> Result<(), UfoError> {
    let (socket, _addr) = listener.accept().await?;

    tokio::spawn(backend_connection(socket, state.clone()));
    Ok(())
}

async fn backend_connection(mut socket: TcpStream, state: Arc<State>) {
    if let Ok(addr) = socket.peer_addr() {
        state
            .log_info(&format!("Backend incoming connection from {:?}", addr))
            .await;
    }
    let mut string_buffer = String::new();
    loop {
        state
            .log_error(
                backend_handle(&mut socket, &state, &mut string_buffer).await,
                "Error handling message from backend",
            )
            .await;
    }
}

async fn backend_handle(
    socket: &mut TcpStream,
    state: &Arc<State>,
    buffer: &mut String,
) -> Result<(), UfoError> {
    let bytes_read = socket.read_to_string(buffer).await?;
    if bytes_read == 0 {
        return Ok(());
    }
    use UfoMessage::*;
    match serde_json::from_str(buffer)? {
        Permission { uid, permissions } => state.set_permission(uid, &permissions).await,
        Message { channel, message } => state.broadcast_message(channel, message).await,
        _ => (),
    };

    Ok(())
}

async fn client_connection(ws: WebSocket, state: Arc<State>) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(
        client_rcv.forward(client_ws_sender), //.map(move |result| { state_clone.log_error(result, "Sending to client").await;})
    );

    let uuid = state.new_client(client_sender).await;

    while let Some(result) = client_ws_rcv.next().await {
        state
            .log_error(
                client_msg(uuid, result, &state).await,
                "Error handling message from client",
            )
            .await;
    }
    state.client_disconnect(uuid).await;
}

async fn client_msg(
    client_id: Uuid,
    message: Result<Message, warp::Error>,
    state: &Arc<State>,
) -> Result<(), UfoError> {
    let message = message?;
    if let Ok(msg) = message.to_str() {
        match serde_json::from_str(msg)? {
            UfoMessage::Subscribe { channel } => state.add_subscription(client_id, channel).await,
            _ => (),
        }
    };
    Ok(())
}
