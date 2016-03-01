/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use controller::Controller;
use service_router;
use foxbox_users::users_router::UsersRouter;
use hyper::error::Error as HyperError;
use hyper::server::Server;
use hyper::net::{ HttpsListener, Ssl, Openssl };
use iron::{ Iron, Protocol, ServerFactory };
use mount::Mount;
use staticfile::Static;
use std::net::SocketAddr;
use std::path::{ PathBuf, Path };
use std::sync::Arc;
use std::thread;
use tls_config::TlsSniConfig;

use openssl::ssl::SslContext;

pub struct HttpServer<T: Controller> {
    controller: T,
}

/// HttpsServerFactory
pub struct HttpsServerFactory<S: Ssl + Clone + Send> {
    ssl: S
}

impl HttpsServerFactory<Openssl> {
    /// Create a new HttpsServerFactory
    pub fn new(ssl: SslContext) -> HttpsServerFactory<Openssl> {
        HttpsServerFactory {
            ssl: Openssl {
                context: Arc::new(ssl)
            }
        }
    }
}

impl ServerFactory<HttpsListener<Openssl>> for HttpsServerFactory<Openssl> {

    fn protocol(&self) -> Protocol {
        Protocol::Https
    }

    fn create_server(&self, sock_addr: SocketAddr) -> Result<Server<HttpsListener<Openssl>>, HyperError> {
        Server::https(sock_addr, self.ssl.clone())
    }
}


impl<T: Controller> HttpServer<T> {
    pub fn new(controller: T) -> Self {
        HttpServer {
            controller: controller,
        }
    }

    pub fn start(&mut self, cert: PathBuf, key: PathBuf) {
        let router = service_router::create(self.controller.clone());

        let mut mount = Mount::new();
        mount.mount("/", Static::new(Path::new("static")))
             .mount("/services", router)
             .mount("/users", UsersRouter::init());

        let addrs: Vec<_> = self.controller.http_as_addrs().unwrap().collect();

        debug!("Starting server with cert: {:?} and key: {:?}", cert, key);

        let mut config = TlsSniConfig::new();
        config.add_ssl_cert("foxbox.local".to_owned(), cert, key).unwrap();
        config.init_ssl_context();


        let server_factory = HttpsServerFactory::new(config.context().unwrap());

        thread::Builder::new().name("LocalHttpServer".to_owned())
                              .spawn(move || {
            Iron::new(mount).listen_with(addrs[0], 8, &server_factory, None).unwrap();
            // Iron::new(mount).https(addrs[0], cert, key).unwrap();
        }).unwrap();
    }
}
