/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use context::SharedContext;
use hyper::uri::RequestUri;
use std::thread;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;

pub struct WebsocketServer {
    context: SharedContext
}

impl WebsocketServer {
    pub fn new(context: SharedContext) -> WebsocketServer {
        WebsocketServer { context: context }
    }

    pub fn start(&self) {
        let addrs: Vec<_> = self.context.lock().unwrap().ws_as_addrs().unwrap().collect();
        let server = Server::bind(addrs[0]).unwrap();

        let context = self.context.clone();
        thread::spawn(move || {
            for connection in server {
                let ctxt1 = context.clone();
                // Spawn a new thread for each connection.
                thread::spawn(move || {
                    let request = connection.unwrap().read_request().unwrap();
                    let url = request.url.clone();

                    // Extract the service id from the url path.
                    let service_id = match url {
                        RequestUri::AbsolutePath(path) => {
                            println!("AbsolutePath {}", path);
                            path[1..].to_string()
                        },
                        _ => {
                            println!("Unsupported url");
                            "".to_string()
                        }
                    };

                    // Incorrect url, bail out.
                    if service_id.is_empty() {
                        request.fail();
                        return;
                    }

                    let ctxt2 = ctxt1.clone();
                    let ctxt3 = ctxt2.lock().unwrap();
                    match ctxt3.get_service(&service_id) {
                        Some(service) => {
                            let response = request.accept();
                            // Send the response
                            let mut client = response.send().unwrap_or_else(|err| {
                               println!("Unable to send response: {}", err);
                               panic!("oops");
                            });

                            let ip = client.get_mut_sender()
                                .get_mut()
                                .peer_addr()
                                .unwrap();

                            println!("WebSocket Connection from {}", ip);
                        },
                        None => {
                            println!("No such service: {}", service_id);
                            request.fail();
                            return;
                        }
                    }

                    /*let message: Message = Message::text(format!("Hello from {}", service_id));
                    client.send_message(&message).unwrap_or_else(|err| {
                       println!("Unable to send message: {}", err);
                       panic!("oops");
                    });

                    let (mut sender, mut receiver) = client.split();

                    for message in receiver.incoming_messages() {
                        let message: Message = message.unwrap();

                        match message.opcode {
                            Type::Close => {
                                let message = Message::close();
                                sender.send_message(&message).unwrap();
                                println!("Client {} disconnected", ip);
                                return;
                            },
                            Type::Ping => {
                                let message = Message::pong(message.payload);
                                sender.send_message(&message).unwrap();
                            }
                            _ => sender.send_message(&message).unwrap(),
                        }
                    }*/
                });
            }
        });
    }
}
