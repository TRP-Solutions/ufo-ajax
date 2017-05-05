var Ufo = (function(){
	status_callback_points = {
		403:'forbidden'
	};

	function Connection(id){
		this.id = id;
		this.callback = {};
		this.http = new XMLHttpRequest();
		this.http.onreadystatechange = function(){
			if(this.readyState==4){
				if(this.status==200){
					parse_reply(id, this.responseText);
				} else if(status_callback_points[this.status]){
					callback(id, status_callback_points[this.status]);
				}
			}
		}
	}

	Connection.prototype.get = function(){
		if(this.url){
			this.http.open('GET',this.url,true);
			this.http.send();
		}
	}

	Connection.prototype.post = function(data){
		if(this.url){
			this.http.open('POST',this.url,true);
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
	function post(id, url, form){
		var con = init(id);
		con.url = url;
		callback(id,'post');
		if(typeof form != 'object') form = document.getElementById(form);
		con.post(new FormData(form));
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
		if(con.interval){
			var timeout = con.interval*Math.round(500+Math.random()*750);
			clearTimeout(con.timer);
			con.timer = setTimeout(update,timeout,id);
		}
	}

	function interval(id,interval){
		// if an interval is to be set, we allow creation of a new connection object
		// this allows interval() to be called before get()
		var con = init(id,!interval);
		if(!con) return;
		if(interval) con.interval = interval;
		else delete con.interval;
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

				case 'callbackadd': callback_add(inst['id'],inst['point'],inst['func']); break;
				case 'callbackremove': callback_remove(inst['id'],inst['point'],inst['funct']); break;
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
		// TODO: This function might need refactoring.
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
				clone.style.display = 'none';
				
				if(original.innerHTML!=clone.innerHTML) {
					original.parentNode.insertBefore(clone, original);
					
					setTimeout(function() {
						clone.style.display = '';
					
						original.style.display = 'none';
						original.parentNode.removeChild(original);
						callback(id,'inner'); // CALL back only run if(original.innerHTML!=clone.innerHTML)
					},1,id);
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
			} else {
				if(content!==null) elem.setAttribute(inst['name'],inst['content']);
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
	function callback(id, point){
		var con = init(id,true);
		if(!con) return;
		if(Array.isArray(con.callback[point])){
			for(var i=0; i < con.callback[point].length; i++){
				callback_call(con.callback[point][i]);
			}
		}
	}

	/* Callback interface */
	var callback_functions = {};

	function callback_add(id,point,func_name){
		var con = init(id);
		if(typeof callback_functions[func_name] != 'function'){
			log('ufo.js | unknown callback function: '+func_name);
			return;
		}
		if(!Array.isArray(con.callback[point])) con.callback[point] = [];
		con.callback[point].push(func_name);
	}

	function callback_remove(id,point,func_name){
		var con = init(id,true);
		if(!con) return;
		if(!Array.isArray(con.callback[point])) return;
		var index = con.callback[point].indexOf(func_name);
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
		post: post,
		get: get,
		update: update,
		interval: interval,
		stop: stop,
		unset: unset,
		abort: abort,
		callback_functions: callback_functions
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