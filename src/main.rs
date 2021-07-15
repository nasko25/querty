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
use regex::Regex;

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
    // tests::reindex_solr(&settings);

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
    // TODO sort by term frequency
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

    // when sorting pharses are split by whitespace characters and sorted by the termfreq of each of the words
    //  for example when the search query is "example rust", the results are first sorted by the
    //  term frequency of "example" and after that sorted by the term frequency of "rust"

    // sanitize the query string
    //  characters taken from:
    //  https://github.com/apache/solr/blob/9903d00b0fb6216f836bb580f42d0081b7b41584/solr/solrj/src/java/org/apache/solr/client/solrj/util/ClientUtils.java#L159
    let sanitized_query = query.replace("\\", "\\\\")
                                    .replace("+", "\\+")
                                    .replace("-", "\\-")
                                    .replace("!", "\\!")
                                    .replace("(", "\\(")
                                    .replace(")", "\\)")
                                    .replace(":", "\\:")
                                    .replace("^", "\\^")
                                    .replace("[", "\\[")
                                    .replace("]", "\\]")
                                    .replace("\"", "\\\"")
                                    .replace("{", "\\{")
                                    .replace("}", "\\}")
                                    .replace("~", "\\~")
                                    .replace("*", "\\*")
                                    .replace("?", "\\?")
                                    .replace("|", "\\|")
                                    .replace("&", "\\&")
                                    .replace(";", "\\;")
                                    .replace("/", "\\/");

    let split_query: Vec<&str> = sanitized_query.split_whitespace().collect();
    // the sort terms are split by non-alphanumeric characters, while the search query is only
    //  split by whitespace characters
    //  (maybe add "" to the sort query; so `text_all:example` will become `text_all:"example"`;
    //  then maybe you don't have to split by non-alphanumeric characters ?)
    let matched_websites = solr::req(&settings, format!("{}&sort={}", &build_search_query(&split_query), &build_sort_query(Regex::new(r"[^a-zA-Z\d]").unwrap().split(&sanitized_query).collect::<Vec<&str>>().into_iter().filter(|word| word.to_string() != "").collect::<Vec<&str>>())));

    if matched_websites.is_ok() {
        return json!(matched_websites.unwrap());
    }
    colour::red!("[ERR]"); println!(" query() returned an error!");
    // if there is something wrong with the suggester just return an empty list as suggestions
    json!([])
}

// helper function that will build a string given an array of strings extracted from the query that will be used in the solr select queries
// for example if the query is "example rust", and this function is called with ["example", "rust"], it will return this string url encoded:
//  termfreq(url,example) desc,termfreq(url,rust) desc,termfreq(text_all,example) desc,termfreq(text_all,rust) desc
fn build_sort_query(words: Vec<&str>) -> String {
    // construct two vectors of words; one sorting by url and the other by text_all
    let mut st1: Vec<&str> = Vec::new();
    let mut st2: Vec<&str> = Vec::new();
    words.iter().for_each(|word| {
        st1.push("termfreq%28url%2C");
        st1.push(word);
        st1.push("%29%20desc");
        st1.push("%2C");

        st2.push("termfreq%28text_all%2C");
        st2.push(word);
        st2.push("%29%20desc");
        st2.push("%2C");
    });
    // remove the last %2C
    st2.pop();

    st1.append(&mut st2);
    return st1.concat();
}

// helper function that builds a string given an array of strings extracted from the query that
//  will be used in the solr select queries
// it is similar to build_sort_query(), but it builds the main query (by splitting the original query string
//  into words) by appending "text_all:" to each word in the original query string
// for example if the query is "example rust", then this function will be called with
//  ["example", "rust"], it will return this string url encoded:
//  text_all:example text_all:rust
fn build_search_query(words: &Vec<&str>) -> String {
    // construct a vector of strings that will be concatinated in the end
    //  and returned by this function
    let mut query: Vec<&str> = Vec::new();
    words.iter().for_each(|word| {
        query.push("text_all%3A");
        query.push(word);
        query.push("%20");  // space
    });
    // remove the last <space> character
    query.pop();
    return query.concat();
}
