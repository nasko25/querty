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

use tests::test_all;
use std::thread;
use std::env;

fn init() {
    // load the environment variables from the .env file
    dotenv().ok();

    // reset the state of the db and solr
    // tests::reset_db_state();

    // reindex solr
    // tests::reindex_solr();

    let mut url = "https://www.rust-lang.org";

    // this url has a weird <a> href (it does not have a host_str()) that should not throw an exception when parsed
    // it also does not have external links, so tests checking that will fail
    // let url = "https://doc.rust-lang.org/std/macro.assert_ne.html";

    // load the website with this url from solr to see if it is in the database
    let mut websites_saved = solr::req(format!("url:\"{}\"", url)).unwrap();
    println!("web saved: {:?}", websites_saved);

    match env::var("RUN_TESTS") {
        Ok(ref var) if var == "True" => {
            // TODO move the code here to tests.rs
            // run tests
            println!("Tests should be Ok: {:?}", test_all(url));

            url = "https://www.spacex.com/";

            websites_saved = solr::req(format!("url:\"{}\"", url)).unwrap();
            println!("web saved: {:?}", websites_saved);

            // analyse a website and update its rank
            let crawler = crawler::Crawler {
                // conn: &*DB_CONN.lock().unwrap()
            };
            crawler.analyse_website(&url, &websites_saved).unwrap();

            // get the id of the website that was just analysed
            websites_saved = solr::req(format!("url:\"{}\"", url)).unwrap();
            assert!(websites_saved.len() == 1, "Solr contains unexpected number of websites with url {}: {}", url, websites_saved.len());
            let website_id = websites_saved[0].id;
            assert!(website_id.is_some(), "The id of the website does not have an id in solr.");

            let updated_rank = react::user_react(&website_id.unwrap().to_string(), react::React::Upvote { var: 0.0 });

            match updated_rank {
                Ok(new_rank) => println!("Rank updated successfully. New rank: {}", new_rank),
                Err(err) => println!("Rank was not updated successfully: Err({})", err),
            }
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
