extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;
#[macro_use] extern crate throw;
#[macro_use] extern crate simple_error;

mod settings;
mod schema;
mod db;
mod solr;
mod tests;
mod crawler;

use tests::test_all;

// TODO move all the tests from main to tests.rs
// TODO add a testing database
fn main() {
    // TODO https://lucene.apache.org/solr instead of mysql
    let settings = settings::Settings::new(false).unwrap();
    let db = &settings.database;
    println!("{:?}", db);
    println!("{:?}", settings.get_serv());

    let url = format!("mysql://{}:{}@{}:{}/{}", &db.user, &db.pass, &db.server, &db.port, &db.db_name);
    println!("{:?}", url);

    let conn = db::Database::establish_connection(&url);

    // reset the state of the db and solr
    // tests::reset_db_state(&conn, &settings);

    let url = "https://www.rust-lang.org";
    // load the website with this url from solr to see if it is in the database
    let websites_saved = crate::solr::req(&settings, format!("url:\"{}\"", url)).unwrap();
    println!("web saved: {:?}", websites_saved);
    crawler::analyse_website(&url, &websites_saved, &conn, &settings);
    // TODO save_website_info(...)
    // get rank from analyse_website

    println!("Tests should be Ok: {:?}", test_all(&settings, &conn));
}
