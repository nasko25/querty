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
    let settings = settings::Settings::new(false).unwrap();
    let db = &settings.database;
    println!("{:?}", db);
    println!("{:?}", settings.get_serv());

    let url = format!("mysql://{}:{}@{}:{}/{}", &db.user, &db.pass, &db.server, &db.port, &db.db_name);
    println!("{:?}", url);

    // TODO get url from config
    let conn = db::Database::establish_connection(&url);

    // TODO foreign keys
    let creat_website = db::Database::create_tables(&conn);
    println!("table website created: {:?}", creat_website);

    let w = Website { id: 2, title: "".to_string(), metadata: "".to_string(), url: "".to_string(), rank: 3, type_of_website: "".to_string() };
    let vals_inserted = db::Database::insert_w(&w, &conn);
    println!("values inserted: {:?}", vals_inserted);
}
