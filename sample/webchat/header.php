<?php

require_once __DIR__ . '/../../lib/ufo.php';
require_once __DIR__ . '/../../../heal-document/lib/HealDocument.php';
require_once __DIR__ . '/../../../boot-some/lib/BootSome.php';

define('UFO_WEBSOCKET_HOST', '127.0.0.1');
define('UFO_WEBSOCKET_BACKEND_PORT', 8081);

function head($title="Socket sample")
{
	BootSome::document($title,'en');

	BootSome::$head->el('meta')->at([
		'charset' => 'utf-8'
	]);

	BootSome::$head->el('meta')->at([
		'name' => 'viewport',
		'content' => 'width=device-width, initial-scale=1'
	]);

	BootSome::$head->css('/boot-some/lib/BootSome.css');

	BootSome::$head->el('script',['src'=>'/boot-some/lib/bootstrap.bundle.min.js']);
	BootSome::$head->el('script',['src'=>'/ufo-ajax/lib/ufo.js']);
	BootSome::$head->el('script',['src'=>'/ufo-ajax/sample/webchat/site.js']);
	BootSome::$head->el('script',['src'=>'/boot-some/lib/BootSome.js']);


	return BootSome::$body;
}