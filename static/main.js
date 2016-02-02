/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

var socket = new WebSocket("ws://localhost:4000/foo/bar");
socket.onmessage = function(event) {
  var received = document.body;
  var br = document.createElement("br");
  var text = document.createTextNode(event.data);
  received.appendChild(br);
  received.appendChild(text);
};
