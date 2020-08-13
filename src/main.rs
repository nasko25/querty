extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;

mod settings;
mod schema;
mod db;

use db::Website;

fn main() {
    // TODO https://lucene.apache.org/solr instead of mysql
    let settings = settings::Settings::new(false);
    println!("{:?}", settings);
    println!("{:?}", settings.unwrap().get_serv());

    // TODO get url from config
    let conn = db::Database::establish_connection(&"mysql://asdf:asdf@localhost:3306/querty");

    // TODO foreign keys
    let creat_website = db::Database::create_tables(&conn);
    println!("table website created: {:?}", creat_website);

    let w = Website { id: 2, title: "".to_string(), metadata: "".to_string(), url: "".to_string(), rank: 3, type_of_website: "".to_string() };
    let vals_inserted = db::Database::insert_w(&w, &conn);
    println!("values inserted: {:?}", vals_inserted);
}
