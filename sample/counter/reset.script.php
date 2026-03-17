<?php

session_start();

require_once __DIR__.'/../../lib/ufo.php';

$_SESSION['count'] = 0;

Ufo::output('counter','0');