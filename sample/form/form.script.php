<?php
/*
UfoAjax is licensed under the Apache License 2.0 license
https://github.com/TRP-Solutions/ufo-ajax/blob/master/LICENSE
*/

require_once __DIR__.'/../../lib/ufo.php';

$email = $_POST['email'] ?? '';
$password = $_POST['password'] ?? '';

// simple validation
if($email !== 'test' || $password !== 'test') {

	Ufo::output('message','<div class="alert alert-danger">Invalid login</div>');

	Ufo::attribute('email','class','form-control is-invalid');
	Ufo::attribute('password','class','form-control is-invalid');

	return;
}

// success
Ufo::output('message','<div class="alert alert-success">Login successful!</div>');

// clear inputs
Ufo::attribute('email','class','form-control');
Ufo::attribute('password','class','form-control');

// optional JS call
Ufo::call('alert','Welcome!');
