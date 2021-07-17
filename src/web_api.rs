use regex::Regex;

use rocket::State;
use rocket_contrib::json::JsonValue;
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

use std::path::PathBuf;
use urlencoding::{ encode, decode };

use crate::settings;
use crate::solr;

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

pub fn mount_web_api_endpoints(settings: settings::Settings) {
    rocket::ignite().attach(CORS).manage(settings).mount("/", routes![suggest, options_handler, query]).launch();
}

// TODO wouldn't post requests be better?
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

    // when sorting pharses are split by whitespace characters and sorted by the termfreq of each of the words
    //  for example when the search query is "example rust", the results are first sorted by the
    //  term frequency of "example" and after that sorted by the term frequency of "rust"

    let sanitized_query = sanitize_query(&query);
    let split_query: Vec<&str> = sanitized_query.split_whitespace().collect();
    // the sort terms are split by non-alphanumeric characters, while the search query is only
    //  split by whitespace characters
    //  (maybe add "" to the sort query; so `text_all:example` will become `text_all:"example"`;
    //  then maybe you don't have to split by non-alphanumeric characters ?)
    // TODO encode(build_search_query(...))? because
    //  http://localhost:8080/results?q=spacex%20rust-lang%26asdasd still throws an error
    let matched_websites = solr::req(&settings, format!("{}&sort={}", &build_search_query(&split_query), &build_sort_query(sanitized_query)));

    if matched_websites.is_ok() {
        return json!(matched_websites.unwrap());
    }
    colour::red!("[ERR]"); println!(" query() returned an error!");
    // if there is something wrong with the suggester just return an empty list as suggestions
    json!([])
}

// helper function that sanitizes the query string
//  characters taken from:
//  https://github.com/apache/solr/blob/9903d00b0fb6216f836bb580f42d0081b7b41584/solr/solrj/src/java/org/apache/solr/client/solrj/util/ClientUtils.java#L159
pub fn sanitize_query(query: &String) -> String {
    return query.replace("\\", "\\\\")
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
}

// helper function that will build a string given an array of strings extracted from the query that will be used in the solr select queries
// for example if the query is "example rust", and this function is called with ["example", "rust"], it will return this string url encoded:
//  termfreq(url,example) desc,termfreq(url,rust) desc,termfreq(text_all,example) desc,termfreq(text_all,rust) desc
pub fn build_sort_query(sanitized_query: String) -> String {
    let words = Regex::new(r"[^a-zA-Z\d]").unwrap().split(&sanitized_query).collect::<Vec<&str>>().into_iter().filter(|word| word.to_string() != "").collect::<Vec<&str>>();
    // construct two vectors of words; one sorting by url and the other by text_all
    let mut st1: Vec<String> = Vec::new();
    let mut st2: Vec<String> = Vec::new();
    words.iter().for_each(|word| {
        let encoded_word: String = encode(word);
        st1.push("termfreq%28url%2C".to_string());
        st1.push(encoded_word.clone());
        st1.push("%29%20desc".to_string());
        st1.push("%2C".to_string());

        st2.push("termfreq%28text_all%2C".to_string());
        st2.push(encoded_word);
        st2.push("%29%20desc".to_string());
        st2.push("%2C".to_string());
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
    let mut query: Vec<String> = Vec::new();
    words.iter().for_each(|word| {
        let encoded_word: String = encode(word);
        query.push("text_all%3A".to_string());
        query.push(encoded_word);
        query.push("%20".to_string());  // space
    });
    // remove the last <space> character
    query.pop();
    return query.concat();
}
