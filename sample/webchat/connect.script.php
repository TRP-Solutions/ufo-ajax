<?php

require_once 'header.php';

$uid = Ufo::websocket_accept_uid(["chat"]);

if ($uid) {
	Ufo::websocket_subscribe("chat");
}
