function loadRom(url, callback) {
	var xhr = new XMLHttpRequest();
	xhr.open('GET', url);
	xhr.responseType = 'arraybuffer';

	xhr.onload = function() { callback(new Blob([xhr.response])) };
	xhr.send();
}
