/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use context::SharedContext;
use std::thread;
use ws::{listen, Handler, Sender, Result, Message, CloseCode};

pub struct WebsocketServer {
    context: SharedContext
}

struct WebsocketHandler {
    out: Sender,
}

impl Handler for WebsocketHandler {

    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Echo the message back
        self.out.send(msg)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        // The WebSocket protocol allows for a utf8 reason for the closing state after the
        // close code. WS-RS will attempt to interpret this data as a utf8 description of the
        // reason for closing the connection. I many cases, `reason` will be an empty string.
        // So, you may not normally want to display `reason` to the user,
        // but let's assume that we know that `reason` is human-readable.
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

impl WebsocketServer {
    pub fn new(context: SharedContext) -> WebsocketServer {
        WebsocketServer { context: context }
    }

    pub fn start(&self) {
        let addrs: Vec<_> = self.context.lock().unwrap().ws_as_addrs().unwrap().collect();

        thread::Builder::new().name("WebsocketServer".to_string())
                              .spawn(move || {
            listen(addrs[0], |out| {
                WebsocketHandler { out: out }
            }).unwrap()
        }).unwrap();
    }
}
