<?php

require_once __DIR__ . '/../../lib/ufo.php';
require_once __DIR__ . '/../../../heal-document/lib/HealDocument.php';
require_once __DIR__ . '/../../../bootsome/lib/BootSome.php';

function head($title="Form Sample")
{
	BootSome::document($title,'en');

	BootSome::$head->el('meta')->at(['charset'=>'utf-8']);

	BootSome::$head->el('meta')->at([
		'name'=>'viewport',
		'content'=>'width=device-width, initial-scale=1'
	]);

	BootSome::$head->css('/bootsome/lib/BootSome.css');

	BootSome::$head->el('script',['src'=>'/bootsome/lib/bootstrap.bundle.min.js']);
	BootSome::$head->el('script',['src'=>'/ufo-ajax/lib/ufo.js']);
	BootSome::$head->el('script',['src'=>'/bootsome/lib/BootSome.js']);

	return BootSome::$body;
}