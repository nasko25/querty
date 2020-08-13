extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;

use diesel::{ insert_into, sql_query };
use crate::diesel::RunQueryDsl;

mod settings;
mod db;
mod schema;

use schema::website::dsl::*;
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

    println!("table website created:{:?}", creat_website);

    let w = Website { id: 1, title: "".to_string(), metadata: "".to_string(), url: "".to_string(), rank: 3, type_of_website: "".to_string() };
    println!("values inserted:{:?}", insert_into(website).values(w).execute(&conn));
}
