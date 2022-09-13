# ufo-ajax

# Live updates with WebSocket

Ufo supports connecting to a `websocket-server` and through it broadcasting messages from backend (PHP) to frontend (JavaScript).

The `websocket-server` is implemented in Rust and all dependencies are Rust libraries.

## Compiling
To compile `websocket-server` you need to have Cargo and a C compiler installed (`sudo apt install build-essentials` if you're running a Debian-based system).

Installing Cargo is easiest by using the installation instructions on rust-lang.org: https://www.rust-lang.org/tools/install

```bash
cd websocket-server
cargo build --release
```

## Using from frontend
The browser connects to `websocket-server` by calling `Ufo.websocket_connet(host, port, auth)`.

* `host` is the hostname of the server.
* `port` is the port the frontend uses to connect. `websocket-server` defaults to port `8080`
* `auth` is the url of a http(s) endpoint that is sent a `POST` request with an `ufo_websocket_uid` value.

To react to broadcasts on a given channel the frontend need to register a callback function for the channel:
```
var channel = "some_channel";
var callback = function(message){
	console.log(message);
};
Ufo.websocket_callback(channel, callback);
```
The channel can be any string. The callback function should take one parameter that is the message being broadcasted.

## Using from backend
To know where to connect to the `websocket-server` Ufo expects the constants `UFO_WEBSOCKET_HOST` and `UFO_WEBSOCKET_BACKEND_PORT` to be defined.

```PHP
define('UFO_WEBSOCKET_HOST','127.0.0.1');
define('UFO_WEBSOCKET_BACKEND_PORT','8081');
```
(The values shown here are the default values `websocket-server` uses with no configuration file.)

Communicating via `websocket-server` from the PHP backend works by using the four `Ufo::websocket_*` static methods.

|Method|Description|
|------|-----------|
|`Ufo::websocket_message($channel, $message)`|Broadcasts a message to all clients subscribed to the given channel.|
|`Ufo::websocket_permission($uid, $permissions)`|Grants the client with the given `$uid` permission to subscribe to the listed channels. `$permissions` must be an array of strings (channel names).|
|`Ufo::websocket_subscribe($channel)`|Sends a regular (not websocket) Ufo-message that instructs the client to subscribe to the given channel.|
|`Ufo::websocket_accept_uid($permissions)`|Helper method that reads an `ufo_websocket_uid` value from `$_POST` and grants the given permissions. Intended for use in the `auth` endpoint that the frontend needs.|

### `auth` : Authorization Endpoint
The authorization endpoint is required to let the backend control which clients are allowed to subscribe to channels. The endpoint should authenticate the client based on session data or similar methods, and if authenticated, call `Ufo::websocket_accept_uid` to assign permissions to the incoming client UID.

## Configuring the websocket-server
Websocket-server tries to read a `config.json` file in the current working directory. If there is one, it reads the following keys:

|Key|Default Value|
|---|-------------|
|host|"127.0.0.1"|
|port_frontend|8080|
|port_backend|8081|
|log|"print"|
|syslog_name|"Ufo-Websocket"|

If `log` is set to `"syslog"`, then it will attempt to connect to syslog for all logging purposes and use `syslog_name` as the process name.