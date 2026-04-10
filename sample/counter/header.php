<?php
/*
UfoAjax is licensed under the Apache License 2.0 license
https://github.com/TRP-Solutions/ufo-ajax/blob/master/LICENSE
*/

declare(strict_types=1);

require_once __DIR__ . '/../../lib/ufo.php';
require_once __DIR__ . '/../../../heal-document/lib/HealDocument.php';
require_once __DIR__ . '/../../../boot-some/lib/BootSome.php';

function head($title="Counter sample")
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
	BootSome::$head->el('script',['src'=>'/boot-some/lib/BootSome.js']);


	return BootSome::$body;
}
