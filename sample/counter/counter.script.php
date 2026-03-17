<?php

session_start();

require_once __DIR__ . '/../../lib/ufo.php';

if(!isset($_SESSION['count'])) {
	$_SESSION['count'] = 0;
}

$_SESSION['count']++;

Ufo::output('counter', $_SESSION['count']);