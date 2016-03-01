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
use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::Error;
use std::net::SocketAddr;
use std::path::{ PathBuf, Path };
use std::sync::Arc;
use std::thread;

use openssl::ssl::{ Ssl as SslImpl, SslContext, SslMethod, SSL_VERIFY_NONE };
use openssl::ssl::error::SslError;
use openssl::x509::X509FileType;
use openssl_sys;

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

pub struct TlsSniConfig {
    ssl_hosts: HashMap<String, SslContext>
}

impl TlsSniConfig {

    pub fn new() -> Self {
        TlsSniConfig {
            ssl_hosts: HashMap::new()
        }
    }

    pub fn context(&self) -> Result<SslContext, Error> {
        for ctx in self.ssl_hosts.values() {
            return Ok(ctx.clone());
        }

        Err(Error::new(ErrorKind::InvalidInput, "An SSL certificate was not configured"))
    }

    pub fn add_ssl_cert<C, K>(&mut self, hostname: String, crt: C, key: K) -> Result<(), SslError>
        where C: AsRef<Path>, K: AsRef<Path> {

        let mut ctx = try!(SslContext::new(SslMethod::Sslv23));
        try!(ctx.set_cipher_list("DEFAULT"));
        try!(ctx.set_certificate_file(crt.as_ref(), X509FileType::PEM));
        try!(ctx.set_private_key_file(key.as_ref(), X509FileType::PEM));
        ctx.set_verify(SSL_VERIFY_NONE, None);

        self.ssl_hosts.insert(hostname.to_owned(), ctx);

        Ok(())
    }

    #[allow(unused_variables)]
    fn servername_callback(ssl: &mut SslImpl, ad: &mut i32, configured_certs: &HashMap<String, SslContext>) -> i32 {
        let requested_hostname = ssl.get_servername();

        if requested_hostname.is_none() {
            return openssl_sys::SSL_TLSEXT_ERR_NOACK;
        }

        let requested_hostname = requested_hostname.unwrap();

        let ssl_context_for_hostname = configured_certs.get(&requested_hostname);

        if let Some(ctx)= ssl_context_for_hostname {
            ssl.set_ssl_context(ctx);
        }

        openssl_sys::SSL_TLSEXT_ERR_OK
    }


    // Stop clippy complaining in the case where we explicitly want a mutable value from the
    // hashmap, if we use the suggestion, then we can't borrow the mutable context.
    #[allow(for_kv_map)]
    pub fn init_ssl_context(&mut self) -> () {

        // TODO: Can we have the type system ensure all certs are added before running this.  Akin
        // to the way Hyper handles writes to the HTTP headers after they're sent
        // https://github.com/hyperium/hyper/blob/2b05fab85e8c1a25fd26cdb01552e69cfbfcd571/src/server/mod.rs#L89

        let configured_certs = self.ssl_hosts.clone();

        for (_, mut ctx) in &mut self.ssl_hosts {
            ctx.set_servername_callback_with_data(Self::servername_callback, configured_certs.clone());
        }
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
