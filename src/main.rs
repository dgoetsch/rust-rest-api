mod api;
mod server;

#[cfg(test)]
mod test;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;


struct RestApp {
    storage_dir: String
}

impl Clone for RestApp {
    fn clone(&self) -> RestApp {
        RestApp { storage_dir: self.storage_dir.clone() }
    }
}
 

fn main() {
    simple_logger::init().unwrap();

    server::http::hyper::server();
}


