extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod settings;

fn main() {
    let settings = settings::Settings::new(false);
    println!("{:?}", settings);
    println!("{:?}", settings.unwrap().get_serv());
}
