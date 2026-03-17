<?php

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
Ufo::attribute('email','class','form-control is-invalid');
Ufo::attribute('password','class','form-control is-invalid');

// optional JS call
Ufo::call('alert','Welcome!');