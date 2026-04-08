<?php

require "header.php";

//if message was in request
$text = $_POST['text'] ?? null;
$username = $_POST['username'] ?? null;

// publish to webchat
if ($text !== null && $text !== '') {
	Ufo::websocket_message('chat', $username.": ".$text);
}

echo Ufo::get_clean();