<?php

require "header.php";

$body = head("Socket sample");

$main = $body->el('div')->at([
	'id'=>'main',
]);

$body->at([
	'onload' => "Ufo.get('main','webchat/load.script.php');"
]);


$container = $main->container();
$row = $container->row();
$col = $row->col('col-md-6','offset-md-3','mt-5');

$form = $col->form()->at([
	'id' => 'sampleform',
	'onsubmit' => "Ufo.post('main','webchat/form.script.php','sampleform'); return false;"
]);

$form->form_group()
	->input('text')
	->at([
		'name'=>'text',
		'id'=>'text',
		'placeholder'=>'Text',
		'required'
	]);

$form->form_group()
	->input('username')
	->at([
		'name'=>'username',
		'id'=>'username',
		'placeholder'=>'Username',
		'required'
	]);


$form->button('Send')->at([
	'type'=>'submit',
	'class'=>'btn btn-primary mt-3'
]);

$form->el('div')->at(['id'=>'message','class'=>'mt-3']);
