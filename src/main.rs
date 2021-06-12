// necessary for rocket
#![feature(proc_macro_hygiene, decl_macro)]

extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;
#[macro_use] extern crate throw;
#[macro_use] extern crate simple_error;

// used for colorful output
extern crate colour;

// used for a web API
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
use rocket::State;
use rocket_contrib::json::JsonValue;
use std::path::PathBuf;


mod settings;
mod schema;
mod db;
mod solr;
mod tests;
mod crawler;

use tests::test_all;
use diesel::MysqlConnection;
use std::fmt;
use std::mem::discriminant;
use urlencoding::decode;

fn main() {
    let settings = settings::Settings::new(false).unwrap();
    let db = &settings.database;
    println!("{:?}", db);
    println!("{:?}", settings.get_serv());

    let url_mysql = format!("mysql://{}:{}@{}:{}/{}", &db.user, &db.pass, &db.server, &db.port, &db.db_name);
    println!("{:?}", url_mysql);

    let conn = db::Database::establish_connection(&url_mysql);

    // reset the state of the db and solr
    // tests::reset_db_state(&conn, &settings);

    // reindex solr
    tests::reindex_solr(&settings);

    let mut url = "https://www.rust-lang.org";

    // this url has a weird <a> href (it does not have a host_str()) that should not throw an exception when parsed
    // it also does not have external links, so tests checking that will fail
    // let url = "https://doc.rust-lang.org/std/macro.assert_ne.html"; 

    // load the website with this url from solr to see if it is in the database
    let mut websites_saved = crate::solr::req(&settings, format!("url:\"{}\"", url)).unwrap();
    println!("web saved: {:?}", websites_saved);

    // run tests
    //println!("Tests should be Ok: {:?}", test_all(url, &settings, &conn));

    url = "https://www.spacex.com/";

    websites_saved = crate::solr::req(&settings, format!("url:\"{}\"", url)).unwrap();
    println!("web saved: {:?}", websites_saved);

    // analyse a website and update its rank
    //let crawler = crawler::Crawler {
    //    conn: &conn,
    //    settings: &settings
    //};
    //crawler.analyse_website(&url, &websites_saved).unwrap();
    //let updated_rank = user_react(url, React::Upvote { val: 0.0 }, &settings, &conn);

    //match updated_rank {
    //    Ok(new_rank) => println!("Rank updated successfully. New rank: {}", new_rank),
    //    Err(err) => println!("Rank was not updated successfully: Err({})", err),
    //}

    // TODO this can be async
    // TODO it can be a new function that mounts all necessary endpoints
    // mount the web API endpoints
    rocket::ignite().attach(CORS).manage(settings).mount("/", routes![suggest, options_handler, query]).launch();
}

// _________________________________________ TODO add new file?__________________________________________

// for now all users reacts will change the website's rank with +/-1.0
// later this could depend on user's ranks
// TODO more sensible name than "val"
// #[derive(PartialEq)]
enum React {
    Upvote { val: f64 },
    Downvote { val: f64 },
}

enum ReactError {
    InvalidArgument { mes: String },
    RankNotUpdated { mes: String },
    GenericError
}

impl fmt::Display for ReactError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReactError::InvalidArgument { mes } => write!(f, "{}", mes),
            ReactError::RankNotUpdated { mes } => write!(f, "{}", mes),
            ReactError::GenericError => write!(f, "An error occured in user_react()") // TODO more sensible error message
        }
    }
}
// TODO passing settings and MysqlConnection everywhere is probably not a good idea
// refactor?
fn user_react(url: &str, react_type: React, settings: &settings::Settings, conn: &MysqlConnection) -> Result<f64, ReactError> {
    println!("Updating the website with url {} after user react.", url);
    let mut websites_saved = crate::solr::req(&settings, format!("url:\"{}\"", url)).unwrap();
    // websites_saved should either be empty (if there are no websites with that url in solr)
    //      in which case the website should just be analysed and its rank should be set to 0.0
    //
    // or websites_saved should have a length of 1 (because olny 1 website should have been fetched from solr
    // because url should be unique)
    if websites_saved.is_empty() {}
    // since website ranks should be between -10 and 10 and user react FOR NOW will only update it
    // with +/-1, I can do this ugly check
    else if websites_saved.len() == 1 && ((websites_saved[0].rank <= 9.0 && discriminant(&react_type) == discriminant(&React::Upvote{ val: 0.0 })) || (websites_saved[0].rank >= -9.0 && discriminant(&react_type) == discriminant(&React::Downvote {val: 0.0}))) {
        println!("{:?}'s old rank: {}", websites_saved[0].id, websites_saved[0].rank);
        websites_saved[0].rank += match react_type {
            React::Upvote { val } => {
                println!("Upvote val: {}", val);
                1.0
            },
            React::Downvote { val } => {
                println!("Downvote val: {}", val);
                -1.0
            },
        };
    }
    else if websites_saved.len() != 1 {
        return Err(ReactError::InvalidArgument { mes: "Vector is not empty and has a size != 1.".to_string() });
    }

    else {
        return Err(ReactError::GenericError);
    }
    let crawler = crawler::Crawler {
        conn,
        settings
    };
    crawler.analyse_website(&url, &websites_saved).unwrap();

    if websites_saved.is_empty() {
        return Err(ReactError::RankNotUpdated { mes: "Url has not been analysed previously, so its rank was set to 0.".to_string() });
    }

    Ok(websites_saved[0].rank)
}


// _________________________________________ TODO add new file?__________________________________________
//                                              Web API

// TODO wouldn't post requests be better?
// TODO take into account / and whitespace characters

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

pub struct CORS;

impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "CORS headers",
            kind: Kind::Response
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "http://localhost:8080")); // could also be Access-Control-Allow-Origin: *
        response.set_header(Header::new("Access-Control-Allow-Methods", "GET, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

// Source for cors: https://stackoverflow.com/questions/62412361/how-to-set-up-cors-or-options-for-rocket-rs
#[get("/suggest/<query>")]
fn suggest(query: String, settings: State<settings::Settings>) -> JsonValue {
    let suggestions = solr::suggest(decode(&query).expect("Cannot url decode the query."), &settings);
    println!("suggestions: {:?}", suggestions);
    // parse the suggestion
    if (suggestions.is_ok()) {
        return json!(suggestions.unwrap().iter().map(|suggestion| &suggestion.term).collect::<Vec<&String>>());
    }
    colour::red!("[ERR]"); println!(" suggest() returned an error!");
    // if there is something wrong with the suggester just return an empty list as suggestions
    json!([])
}

#[options("/suggest/<path..>")]
fn options_handler<'a>(path: PathBuf) -> Response<'a> {
    Response::build()
        //.raw_header("Access-Control-Allow-Origin", "http://localhost:8080")
        //.raw_header("Access-Control-Allow-Methods", "OPTIONS, GET")
        //.raw_header("Access-Control-Allow-Headers", "Content-Type")
        .finalize()
}

// TODO returning 404 might be better if solr has no response ?
//  (although this is just an API, so an empty array should also be acceptable?)
#[get("/query/<query>")]
fn query(query: String, settings: State<settings::Settings>) -> JsonValue {
    // TODO maybe add an endpoint that only returns the important fields of the websites (title,
    //  url and the relevant part of the text)
    //  also sort by term frequency and setup spellchecker (check the TODO file)
    let matched_websites = solr::req(&settings, format!("text_all:{q}&sort=termfreq(text_all:{q}) desc", q = decode(&query).expect("Cannot url decode the query")));

    if matched_websites.is_ok() {
        return json!(matched_websites.unwrap());
    }
    colour::red!("[ERR]"); println!(" query() returned an error!");
    // if there is something wrong with the suggester just return an empty list as suggestions
    json!([])
}
