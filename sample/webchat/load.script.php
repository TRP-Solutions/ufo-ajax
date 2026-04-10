<?php

require_once "header.php";

Ufo::call('ws_connect','127.0.0.1', 8080, 'connect.script.php');

if (!isset($_SESSION['chat'])) {
	$_SESSION['chat'] = [];
}