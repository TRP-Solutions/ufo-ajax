<?php
/*
UfoAjax is licensed under the Apache License 2.0 license
https://github.com/TRP-Solutions/ufo-ajax/blob/master/LICENSE
*/

require "header.php";

$body = head("Function call sample");

$body->at(['class'=>'m-0 p-0']);

$main = $body->el('div')->at([
	'id'=>'main',
	'class'=>'vh-100 d-flex justify-content-center align-items-center bg-light'
]);

$col = $main->el('div')->at([
	'class'=>'col-12 col-sm-8 col-md-6 col-lg-4'
]);

$card = $col->card()->at(['class'=>'border-0 rounded-4 overflow-hidden']);

$cardBody = $card->body()->at(['class'=>'text-center p-4 pt-2']);

// buttons
$btnRow = $cardBody->el('div')->at([
	'class'=>'d-flex justify-content-center gap-3'
]);

$btnRow->button("Call")->at([
	'class'=>'btn btn-success px-4 py-2',
	'onclick'=>"Ufo.get('main','call.script.php');"
]);
