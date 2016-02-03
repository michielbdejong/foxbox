/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use context::SharedContext;
use std::thread;
use ws::{Builder, Error, ErrorKind, Factory, Handler, Handshake, Sender, Request, Response, Result, Message, CloseCode};

pub struct WebsocketServer {
    context: SharedContext
}

struct WebsocketHandler {
    out: Sender,
    context: SharedContext
}

impl Handler for WebsocketHandler {
    // Check if the path matches a service id, and close the connection
    // if not.
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        println!("on_request {}", req.resource());
        let service_id = &req.resource()[1..];
        println!(" service id is {}", service_id);

        // Hardcoded endpoint where clients can listen to general notifications,
        // like device joining and departing.
        if service_id == "services" {
            let res = try!(Response::from_request(req));
            return Ok(res);
        }

        // Look for a service.
        match self.context.lock().unwrap().get_service(service_id) {
            None => Err(Error::new(ErrorKind::Internal, "No such service")),
            Some(_) => {
                let res = try!(Response::from_request(req));
                Ok(res)
            }
        }
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("on_open {}", shake.request.resource());
        Ok(())
    }

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

struct WebsocketFactory {
    context: SharedContext
}

impl WebsocketFactory {
    fn new(context: SharedContext) -> WebsocketFactory {
        WebsocketFactory {
            context: context
        }
    }
}

impl Factory for WebsocketFactory {
    type Handler = WebsocketHandler;

    fn connection_made(&mut self, sender: Sender) -> Self::Handler {
        let ctx = self.context.clone();
        WebsocketHandler {
            out: sender,
            context: ctx
        }
    }
}

impl WebsocketServer {
    pub fn new(context: SharedContext) -> WebsocketServer {
        WebsocketServer { context: context }
    }

    pub fn start(&self) {
        let addrs: Vec<_> =
            self.context.lock().unwrap().ws_as_addrs().unwrap().collect();

        let context = self.context.clone();
        thread::Builder::new().name("WebsocketServer".to_owned())
                              .spawn(move || {
            let factory = WebsocketFactory::new(context);
            Builder::new().build(factory).unwrap().listen(addrs[0]).unwrap();
        }).unwrap();
    }
}
