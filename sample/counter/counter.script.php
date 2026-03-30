<?php
/*
UfoAjax is licensed under the Apache License 2.0 license
https://github.com/TRP-Solutions/ufo-ajax/blob/master/LICENSE
*/

session_start();

require_once __DIR__ . '/../../lib/ufo.php';

if(!isset($_SESSION['count'])) {
	$_SESSION['count'] = 0;
}

$_SESSION['count']++;

Ufo::output('counter', $_SESSION['count']);
