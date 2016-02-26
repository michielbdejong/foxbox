/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use controller::Controller;
use service_router;
use foxbox_users::users_router::UsersRouter;
use iron::Iron;
use mount::Mount;
use staticfile::Static;
use std::collections::HashMap;
use std::path::{ PathBuf, Path };
use std::thread;

use openssl::ssl::{ Ssl, SslContext, SslMethod, SSL_VERIFY_NONE };
use openssl::ssl::error::SslError;
use openssl::x509::X509FileType;
use openssl_sys;

pub struct HttpServer<T: Controller> {
    controller: T,
    ssl_hosts:  HashMap<String, SslContext>
}

impl<T: Controller> HttpServer<T> {
    pub fn new(controller: T) -> Self {
        HttpServer {
            controller: controller,
            ssl_hosts: HashMap::new(),
        }
    }

    fn add_ssl_cert<C, K>(&mut self, hostname: String, crt: C, key: K) -> Result<(), SslError>
        where C: AsRef<Path>, K: AsRef<Path> {

        let mut ctx = try!(SslContext::new(SslMethod::Sslv23));
        try!(ctx.set_cipher_list("DEFAULT"));
        try!(ctx.set_certificate_file(crt.as_ref(), X509FileType::PEM));
        try!(ctx.set_private_key_file(key.as_ref(), X509FileType::PEM));
        ctx.set_verify(SSL_VERIFY_NONE, None);

        self.ssl_hosts.insert(hostname.to_owned(), ctx);

        Ok(())
    }

    fn servername_callback(ssl: &mut Ssl, ad: &mut i32, configured_certs: &HashMap<String, SslContext>) -> i32 {
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
    fn init_ssl_context(&mut self) -> () {

        // TODO: Can we have the type system ensure all certs are added before running this.  Akin
        // to the way Hyper handles writes to the HTTP headers after they're sent
        // https://github.com/hyperium/hyper/blob/2b05fab85e8c1a25fd26cdb01552e69cfbfcd571/src/server/mod.rs#L89

        let configured_certs = self.ssl_hosts.clone();

        for (_, mut ctx) in &mut self.ssl_hosts {
            ctx.set_servername_callback_with_data(Self::servername_callback, configured_certs.clone());
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

        thread::Builder::new().name("LocalHttpServer".to_owned())
                              .spawn(move || {
            Iron::new(mount).https(addrs[0], cert, key).unwrap();
        }).unwrap();
    }
}
