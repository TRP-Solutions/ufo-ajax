use config::Config;
use env_logger;
use futures::{FutureExt, StreamExt};
use log::{debug, error, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, collections::HashSet, net::IpAddr, sync::Arc};
use syslog::{BasicLogger, Facility, Formatter3164};
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
    subscriptions: Mutex<HashMap<String, HashSet<Uuid>>>,
}

impl State {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            clients: Mutex::new(HashMap::new()),
            subscriptions: Mutex::new(HashMap::new()),
        })
    }

    async fn new_client(&self, sender: Sender) -> Uuid {
        let uuid = Uuid::new_v4();
        let mut locked_clients = self.clients.lock().await;
        locked_clients.insert(uuid, Client::new(sender));
        info!("Client {} connected", uuid);
        if let Some(client) = locked_clients.get(&uuid) {
            if let Some(sender) = &client.sender {
                if let Err(e) = (UfoMessage::Uid { uid: uuid }).send(sender) {
                    error!("Sending uid to client: {:?}", e);
                }
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
            debug!(
                "Client {} received permissions: {:?}",
                client_id, permissions
            );
            if !client.ready {
                if let Some(sender) = &client.sender {
                    if let Err(e) = UfoMessage::Ready.send(sender) {
                        error!("Sending ready to client: {:?}", e);
                    }
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
                    client_list.insert(client_id);
                } else {
                    let mut set = HashSet::new();
                    set.insert(client_id);
                    locked_subscriptions.insert(channel.to_string(), set);
                }
                debug!("Client {} subscribed to {:?}", client_id, channel);
            } else {
                warn!(
                    "Client {} tried subscribing to {:?} but didn't have permission",
                    client_id, channel
                );
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
        info!("Client {} disconnected", client_id);
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
                    error!("Building broadcast: {:?}", e);
                    return;
                }
            };
            debug!(
                "Broadcasting to {} clients on channel {:?}",
                client_list.len(),
                channel
            );
            for client_id in client_list {
                if let Some(client) = locked_clients.get(client_id) {
                    if let Some(sender) = &client.sender {
                        if let Err(e) = sender.send(Ok(Message::text(&msg))) {
                            error!("Broadcasting to clients on channel {:?}: {:?}", channel, e)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum LoggingMethod {
    Syslog(String),
    Print,
}

impl LoggingMethod {
    fn from_config(config: Config) -> Self {
        if let Ok(value) = config.get::<String>("log") {
            match value.as_str() {
                "syslog" => {
                    return Self::Syslog(
                        config
                            .get("syslog_name")
                            .unwrap_or_else(|_| "Ufo-Websocket".to_string()),
                    )
                }
                "print" => return Self::Print,
                _ => (),
            }
        };
        println!("Undefined logging method. Defaulting to env_logger.");
        Self::Print
    }

    fn init(self) {
        match self {
            Self::Syslog(process) => {
                let formatter = Formatter3164 {
                    facility: Facility::LOG_LOCAL7,
                    process,
                    hostname: None,
                    ..Default::default()
                };

                let logger = syslog::unix(formatter).expect("Failed to connect to syslog");
                log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
                    .map(|()| log::set_max_level(LevelFilter::Debug))
                    .expect("could not register logger");
            }
            Self::Print => {
                env_logger::Builder::from_env(
                    env_logger::Env::default().default_filter_or("debug"),
                )
                .init();
            }
        }
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

    let (host, frontend_port, backend_port, logging_method) = match config_builder.build() {
        Ok(config) => (
            config.get("host").unwrap_or_else(|_| [127, 0, 0, 1].into()),
            config.get("port_frontend").unwrap_or(8080),
            config.get("port_backend").unwrap_or(8081),
            LoggingMethod::from_config(config),
        ),
        Err(e) => {
            println!(
                "Failed reading config file. Using default values. ({:?})",
                e
            );
            ([127, 0, 0, 1].into(), 8080, 8081, LoggingMethod::Print)
        }
    };

    logging_method.init();

    let backend = backend_server(state.clone(), (host, backend_port));

    let state_clone = state.clone();
    let frontend_websocket = warp::ws().map(move |ws: warp::ws::Ws| {
        let state = state_clone.clone();
        ws.on_upgrade(move |socket| client_connection(socket, state))
    });

    info!("Frontend listening on {}:{}", host, frontend_port);
    tokio::join!(
        warp::serve(frontend_websocket).run((host, frontend_port)),
        backend
    );

    info!("Program ended");
}

async fn backend_server(state: Arc<State>, addr: (IpAddr, u16)) {
    match TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Backend listening on {}:{}", addr.0, addr.1);
            loop {
                if let Err(e) = backend_listen(&listener, &state).await {
                    error!("Failed to listen: {:?}", e);
                }
            }
        }
        Err(e) => error!("Failed to bind: {:?}", e),
    }
}

async fn backend_listen(listener: &TcpListener, state: &Arc<State>) -> Result<(), UfoError> {
    let (socket, _addr) = listener.accept().await?;

    tokio::spawn(backend_connection(socket, state.clone()));
    Ok(())
}

async fn backend_connection(mut socket: TcpStream, state: Arc<State>) {
    if let Ok(addr) = socket.peer_addr() {
        info!("Backend incoming connection from {:?}", addr);
    }
    let mut string_buffer = String::new();
    loop {
        if let Err(e) = backend_handle(&mut socket, &state, &mut string_buffer).await {
            error!("Error handling message from backend: {:?}", e);
        }
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

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("Sending to client: {:?}", e);
        }
    }));

    let uuid = state.new_client(client_sender).await;

    while let Some(result) = client_ws_rcv.next().await {
        if let Err(e) = client_msg(uuid, result, &state).await {
            error!("Error handling message from client: {:?}", e);
        }
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
