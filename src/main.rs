extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;

mod settings;
mod db;

fn main() {
    let settings = settings::Settings::new(false);
    println!("{:?}", settings);
    println!("{:?}", settings.unwrap().get_serv());

    // TODO get url from config
    db::Database::establish_connection(&"mysql://asdf:asdf@localhost:3306/querty");
}
