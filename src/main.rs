// necessary for rocket
#![feature(proc_macro_hygiene, decl_macro)]

extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;
#[macro_use] extern crate throw;
#[macro_use] extern crate simple_error;
extern crate dotenv;

// used for colorful output
extern crate colour;

// used for a web API
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;


#[macro_use] mod crawl;
mod settings;
mod schema;
mod db;
mod solr;
mod tests;
mod crawler;
mod web_api;
mod react;

// used to load .env
use dotenv::dotenv;

use tests::{ test_all, test_analyse_website };
use std::thread;
use std::env;

fn init() {
    // load the environment variables from the .env file
    dotenv().ok();

    // reset the state of the db and solr
    // tests::reset_db_state();

    // reindex solr
    // tests::reindex_solr();

    let url = "https://www.rust-lang.org";

    // this url has a weird <a> href (it does not have a host_str()) that should not throw an exception when parsed
    // it also does not have external links, so tests checking that will fail
    // let url = "https://doc.rust-lang.org/std/macro.assert_ne.html";

    // load the website with this url from solr to see if it is in the database
    let websites_saved = solr::req(format!("url:\"{}\"", url)).unwrap();
    println!("web saved: {:?}", websites_saved);

    match env::var("RUN_TESTS") {
        Ok(ref var) if var == "True" => {
            // run tests
            println!("Tests should be Ok: {:?}", test_all(url));

            println!("Test website analysis and user reactions\nShould be Ok: {:?}", test_analyse_website());
        },
        Ok(_) => colour::yellow!("Set RUN_TESTS to \"True\" to run the tests."),
        Err(_) => colour::red!("Environment variable RUN_TESTS is not set.")
    }
}

fn main() {
    let init_thread_handle = thread::spawn(move || { init() });

    // thread blocking!
    web_api::mount_web_api_endpoints();

    // if mount_web_api_endpoints() fails for some reason, wait for init() to finish
    init_thread_handle.join().unwrap();
}
