Ufo.websocket_callback("chat", function(msg) {
    console.log(msg);

    const box = document.getElementById("message");

    const el = document.createElement("div");
    el.textContent = msg;

    box.prepend(el);
});

Ufo.callback_functions.ws_connect = function(host, port, auth) {
    Ufo.websocket_connect(host, port, auth);
};