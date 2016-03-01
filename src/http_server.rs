/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use controller::Controller;
use service_router;
use foxbox_users::users_router::UsersRouter;
use iron::Iron;
use mount::Mount;
use staticfile::Static;
use std::path::{ PathBuf, Path };
use std::thread;

pub struct HttpServer<T: Controller> {
    controller: T
}

impl<T: Controller> HttpServer<T> {
    pub fn new(controller: T) -> Self {
        HttpServer { controller: controller }
    }

    pub fn start(&mut self, cert: PathBuf, key: PathBuf) {
        let router = service_router::create(self.controller.clone());

        let mut mount = Mount::new();
        mount.mount("/", Static::new(Path::new("static")))
             .mount("/services", router)
             .mount("/users", UsersRouter::init());

        let addrs: Vec<_> = self.controller.http_as_addrs().unwrap().collect();

        thread::Builder::new().name("LocalHttpServer".to_owned())
                              .spawn(move || {
            Iron::new(mount).https(addrs[0], cert, key).unwrap();
        }).unwrap();
    }
}
