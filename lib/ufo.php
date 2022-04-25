<?php
/*
UfoAjax is licensed under the Apache License 2.0 license
https://github.com/TRP-Solutions/ufo-ajax/blob/master/LICENSE
*/
class Ufo {
	public static function get_clean(){
		$instance = self::get_instance();
		$messages = json_encode($instance->messages);
		$instance->messages = [];
		return $messages;
	}

	public static function log(...$args){
		self::add('log',['args'=>$args]);
	}

	public static function output($target, $content){
		self::add('output',['target'=>$target,'content'=>(string)$content]);
	}

	public static function attribute($target, $name, $content){
		self::add('attribute',['target'=>$target,'name'=>$name,'content'=>$content]);
	}

	public static function close($target){
		self::add('close',['target'=>$target]);
	}

	public static function post($id, $url){
		self::add('post',['id'=>$id,'url'=>$url]);
	}

	public static function get($id, $url){
		self::add('get',['id'=>$id,'url'=>$url]);
	}

	public static function interval($id, $interval){
		self::add('interval',['id'=>$id,'interval'=>$interval]);
	}

	public static function update($id){
		self::add('update',['id'=>$id]);
	}

	public static function stop($id){
		self::add('stop',['id'=>$id]);
	}

	public static function remove($id){
		self::add('unset',['id'=>$id]);
	}

	public static function abort($id){
		self::add('abort',['id'=>$id]);
	}

	public static function callbackadd($id,$point,$func,...$args){
		self::add('callbackadd',['id'=>$id,'point'=>$point,'func'=>$func,'args'=>$args]);
	}

	public static function callbackremove($id,$point,$func){
		self::add('callbackremove',['id'=>$id,'point'=>$point,'func'=>$func]);
	}

	public static function callbackclear($id){
		self::add('callbackclear',['id'=>$id]);
	}

	public static function call($func,...$args){
		self::add('call',['func'=>$func,'args'=>$args]);
	}

	public static function dataset($key, $value){
		self::add('dataset',['key'=>$key,'value'=>$value]);
	}

	public static function nop(){
		self::add('nop');
	}

	private static $instance;
	private static function get_instance(){
		if(!isset(self::$instance)) self::$instance = new self();
		return self::$instance;
	}

	private static function add($type, $data = []){
		$data['type'] = $type;
		$instance = self::get_instance();
		$instance->messages[] = $data;
	}

	private $messages = [];
	private function __construct(){
		$this->handle_connection_close_request();
		register_shutdown_function([$this,'write']);
	}

	private function handle_connection_close_request(){
		if(!headers_sent() && !empty($_SERVER['HTTP_UFO_CONNECTION']) && strtolower($_SERVER['HTTP_UFO_CONNECTION']) == 'close'){
			header('Connection: close');
		}
	}

	public function write(){
		if(!empty($this->messages)){
			$error = error_get_last();
			if(headers_sent() || isset($error)) echo "\x02";
			echo json_encode($this->messages);
		}
	}
}
