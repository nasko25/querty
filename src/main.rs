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

    let mut url = "https://www.rust-lang.org";

    // this url has a weird <a> href (it does not have a host_str()) that should not throw an exception when parsed
    // it also does not have external links, so tests checking that will fail
    // let url = "https://doc.rust-lang.org/std/macro.assert_ne.html"; 

    // load the website with this url from solr to see if it is in the database
    let mut websites_saved = crate::solr::req(&settings, format!("url:\"{}\"", url)).unwrap();
    println!("web saved: {:?}", websites_saved);
    // TODO save_website_info(...)
    // get rank from analyse_website

    println!("Tests should be Ok: {:?}", test_all(url, &settings, &conn));

    url = "https://www.spacex.com/";

    websites_saved = crate::solr::req(&settings, format!("url:\"{}\"", url)).unwrap();
    println!("web saved: {:?}", websites_saved);

    crawler::analyse_website(&url, &websites_saved, &conn, &settings);

    println!("Updating the website with url {}.", url);
    websites_saved = crate::solr::req(&settings, format!("url:\"{}\"", url)).unwrap();
    println!("{:?}", websites_saved[0].id);
    crawler::analyse_website(&url, &websites_saved, &conn, &settings);
}
