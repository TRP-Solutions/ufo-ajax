<?php
/*
UfoAjax is licensed under the Apache License 2.0 license
https://github.com/TRP-Solutions/ufo-ajax/blob/master/LICENSE
*/

require "header.php";

$body = head("Form sample");

$body->el('div')->at(['id'=>'main']);

$container = $body->container();
$row = $container->row();
$col = $row->col('col-md-6','offset-md-3','mt-5');

$form = $col->form()->at([
	'id' => 'sampleform',
	'onsubmit' => "Ufo.post('main','form.script.php','sampleform'); return false;"
]);

$form->el('h4')->te("sample form");

$form->form_group()
	->input('email')
	->at([
		'name'=>'email',
		'id'=>'email',
		'placeholder'=>'Email',
		'required'
	]);

$form->form_group()
	->password('password')
	->at([
		'name'=>'password',
		'id'=>'password',
		'placeholder'=>'Password',
		'required'
	]);

// error message
$form->el('div')->at(['id'=>'message','class'=>'mt-3']);

$form->button('Login')->at([
	'type'=>'submit',
	'class'=>'btn btn-primary mt-3'
]);
