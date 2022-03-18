/*
UfoAjax is licensed under the Apache License 2.0 license
https://github.com/TRP-Solutions/ufo-ajax/blob/master/LICENSE
*/
var Ufo = (function(){
	var status_callback_points = {
		403:'forbidden'
	};
	var url_root = '';
	function modify_url(url){
		return url_root + url + (url.indexOf('?')==-1 ? "?" : "&") + "ufo=" + Math.floor((Math.random() * 8999999) + 1000000);
	}

	function set_root(new_root){
		url_root = new_root;
	}

	function Connection(id){
		this.id = id;
		this.callback = {};
		this.http = new XMLHttpRequest();
		this.http.onreadystatechange = function(){
			if(this.readyState==4){
				if(this.status==200){
					callback(id, 'reply');
					parse_reply(id, this.responseText);
				} else if(status_callback_points[this.status]){
					callback(id, status_callback_points[this.status]);
				}
			}
		}
	}

	Connection.prototype.get = function(){
		if(this.url){
			this.http.open('GET',modify_url(this.url),true);
			this.http.send();
		}
	}

	Connection.prototype.post = function(data){
		if(this.url){
			this.http.open('POST',modify_url(this.url),true);
			this.http.send(data);
		}
	}

	Connection.prototype.abort = function(){
		if(this.http){
			this.http.abort();
		}
	}


	/* Client-Server */
	/* Client-Server helpers */
	var connections = {};
	function init(id, dont_create){
		if(!connections[id] && !dont_create) connections[id] = new Connection(id);
		return connections[id];
	}

	/* Client-Server interface  */
	function post(id, url, form, upload_callback){
		var con = init(id);
		con.url = url;
		if(typeof upload_callback == 'function' && typeof con.http.upload == 'object') upload_callback(con.http.upload);
		if(typeof form != 'object') form = document.getElementById(form);
		callback(id,'post',[form]);
		if(!(form instanceof FormData)) form = new FormData(form);
		if(typeof form.entries == 'function' && typeof form.delete == 'function'){
			for(var pair of form.entries()){
				if(typeof pair[1] == 'object' && pair[1].size == 0) form.delete(pair[0]); // Some versions of Safari can't handle empty file inputs in a FormData object
			}
		}
		con.post(form);
	}

	function get(id, url){
		var con = init(id);
		con.url = url;
		callback(id,'get');
		update(id);
	}

	function update(id){
		var con = init(id);
		callback(id,'update');
		if(!con.url){
			log('ufo.js | update - no url: '+id);
			return;
		}
		con.get();
		delay(id);
	}

	function interval(id,interval){
		// if an interval is to be set, we allow creation of a new connection object
		// this allows interval() to be called before get()
		var con = init(id,!interval);
		if(!con) return;
		if(interval) con.interval = interval;
		else delete con.interval;
	}

	function delay(id){
		var con = init(id,true);
		if(!con) return;
		if(con.interval){
			var timeout = con.interval*Math.round(500+Math.random()*750);
			clearTimeout(con.timer);
			con.timer = setTimeout(update,timeout,id);
		}
	}

	function stop(id){
		var con = init(id,true);
		if(!con) return;
		if(con.timer) clearTimeout(con.timer);
		abort();
	}

	function unset(id){
		stop(id);
		delete connections[id];
	}

	function abort(id){
		var con = init(id,true);
		if(!con) return;
		callback(id,'abort');
		con.abort();
	}

	function url(id){
		var con = init(id,true);
		return con ? con.url : undefined;
	}

	function list_connections(){
		return Object.keys(connections);
	}

	/* Server-Client */
	/* Server-Client helpers */
	function parse_reply(id, content){
		try {
			content = content.split("\x02");
			if(content.length > 1){
				if(content[0]) log("UFO ignored content:\n",content[0]);
				content = content[1];
			} else content = content[0];
			var output_array = JSON.parse(content);
		}
		catch(e) {
			alert("ufoReturn - ParseError (log): "+e);
			log(content);
			return;
		}
		var tailcalls = [];
		for(key in output_array){
			var inst = output_array[key];
			switch(inst['type']){
				case 'post': get(inst['id'],inst['url'],inst['form']); break;
				case 'get': get(inst['id'],inst['url']); break;
				case 'interval': interval(inst['id'],inst['interval']); break;
				case 'update': update(inst['id']); break;
				case 'stop': stop(inst['id']); break;
				case 'unset': unset(inst['id']); break;
				case 'abort': abort(inst['id']); break;

				case 'log': reply_log(inst); break;
				case 'output': reply_output(inst, id); break;
				case 'attribute': reply_attribute(inst); break;
				case 'close': reply_close(inst); break;

				case 'callbackadd': callback_add(inst['id'],inst['point'],inst['func'],inst['args']); break;
				case 'callbackremove': callback_remove(inst['id'],inst['point'],inst['func']); break;
				case 'callbackclear': callback_clear(inst['id']); break;
				case 'call': tailcalls.push(inst); break;

				case 'dataset': exportobj.data.set(inst['key'],inst['value']); break;

				case 'nop': break;
			}
		}
		for(var i=0;i<tailcalls.length;i++){
			callback_call(tailcalls[i]['func'],tailcalls[i]['args']);
		}
	}

	function reply_log(inst){
		if(Array.isArray(inst['args'])){
			log.apply(this,inst['args']);
		} else if(typeof inst['text'] != 'undefined'){
			log(inst['text']);
		} else {
			log(inst);
		}
	}

	function reply_output(inst, id){
		var target = inst['target'];
		var content = inst['content'];
		var original = document.getElementById(target);

		if(!original){
			setTimeout(function(){
				original = document.getElementById(target);
				replace();
			},100);
		} else {
			replace();
		}

		function replace(){
			if(original) {
				var clone = original.cloneNode(true);
				clone.innerHTML=content;
				if(original.innerHTML != clone.innerHTML) {
					var top = original.scrollTop;
					var left = original.scrollLeft;
					original.parentNode.replaceChild(clone, original);
					clone.scrollTop = top;
					clone.scrollLeft = left;
					callback(id,'inner'); // CALL back only run if(original.innerHTML!=clone.innerHTML)
				}
			}
			else {
				alert("ufoInner: "+target);
			}
		}
	}

	function reply_attribute(inst){
		var elem = document.getElementById(inst['target']);
		if(elem){
			if(inst['name']=="value") { // SUPPORT FOR SELECT
				elem.value = inst['content'];
				elem.dispatchEvent(new Event('change'));
			} else if(elem.nodeName=='INPUT' && inst['name']=="checked") { // SUPPORT FOR CHECKBOX
				elem.checked = inst['content'];
				elem.dispatchEvent(new Event('change'));
			} else {
				if(inst['content']!==null) elem.setAttribute(inst['name'],inst['content']);
				else elem.removeAttribute(inst['name']);
			}
		} else {
			log('Ufo missing attribute target: '+inst['target']);
		}
	}

	function reply_close(inst){
		var elem = document.getElementById(inst['target']);
		elem.style.display = 'none';
		elem.innerHTML = '';
	}
	
	/* Callback */
	/* Callback helpers*/
	function callback(id, point, additional_args){
		var con = init(id,true);
		if(!con) return;
			
		if(Array.isArray(con.callback[point])){
			var callbacks = con.callback[point]
			for(var i=0; i < callbacks.length; i++){
				if(Array.isArray(callbacks[i].args)){
					var args = callbacks[i].args.concat(additional_args);
				} else {
					args = additional_args;
				}
				callback_call(callbacks[i].func, args);
			}
		}
	}

	/* Callback interface */
	var callback_functions = {};

	function callback_add(id,point,func_name,args){
		var con = init(id);
		if(typeof callback_functions[func_name] != 'function'){
			log('ufo.js | unknown callback function: '+func_name);
			return;
		}
		if(!Array.isArray(con.callback[point])) con.callback[point] = [];
		con.callback[point].push({func:func_name,args:args});
	}

	function callback_remove(id,point,func_name){
		var con = init(id,true);
		if(!con) return;
		if(!Array.isArray(con.callback[point])) return;
		for(var i=0;i<con.callback[point].length;i++){
			if(con.callback[point][i].name == func_name){
				var index = i;
				break;
			}
		}
		delete con.callback[point][index];
	}

	function callback_clear(id){
		var con = init(id,true);
		if(!con) return;
		con.callback = {};
	}

	function callback_call(func_name, args){
		var func = callback_functions[func_name];
		if(typeof func=='function') func.apply(null,args);
	}

	/* Misc helpers */

	function log(){
		console.log.apply(console,arguments);
	}

	var exportobj = {
		set_root: set_root,
		post: post,
		get: get,
		update: update,
		interval: interval,
		delay: delay,
		stop: stop,
		unset: unset,
		abort: abort,
		url: url,
		connections: list_connections,
		callback_functions: callback_functions,
		callback_add: callback_add,
		callback_remove: callback_remove,
		callback_clear: callback_clear
	};

	/* Data Store */
	exportobj.data = (function(){
		var data = {};
		var callbacks = {};
		var ln = {};
		function ln_add(values){
			for(var key in values){
				if(typeof values[key] == 'string') ln[key] = values[key];
			}
		}

		var exportobj = {};
		exportobj.set = function(key, value){
			if(key == 'ln') return ln_add(value);
			data[key] = value;
			if(!callbacks[key]) return;
			for(var i = 0; i < callbacks[key].length; i++){
				if(typeof callbacks[key][i] == 'function') callbacks[key][i](value);
			}
		}
		exportobj.get = function(key){
			return data[key];
		}
		exportobj.listen = function(key, callback){
			if(!callbacks[key]) callbacks[key] = [];
			else if(callbacks[key].indexOf(callback) == -1) return;
			callbacks[key].push(callback);
		}
		exportobj.unlisten = function(key, callback){
			if(!callbacks[key]) return;
			var index = callbacks[key].indexOf(callback);
			if(index == -1) return;
			callbacks[key] = callbacks[key].splice(index, 1);
		}
		exportobj.ln = function(key){
			return ln[key] ? ln[key] : key;
		}
		return exportobj;
	})();


	return exportobj;
})();