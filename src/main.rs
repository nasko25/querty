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
use diesel::MysqlConnection;
use std::fmt;

// TODO move all the tests from main to tests.rs
// TODO add a testing database
fn main() {
    // TODO https://lucene.apache.org/solr instead of mysql
    let settings = settings::Settings::new(false).unwrap();
    let db = &settings.database;
    println!("{:?}", db);
    println!("{:?}", settings.get_serv());

    let url_mysql = format!("mysql://{}:{}@{}:{}/{}", &db.user, &db.pass, &db.server, &db.port, &db.db_name);
    println!("{:?}", url_mysql);

    let conn = db::Database::establish_connection(&url_mysql);

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
    let updated_rank = user_react(url, React::Upvote { val: 0.0 }, &settings, &conn);

    match updated_rank {
        Ok(new_rank) => println!("Rank updated successfully. New rank: {}", new_rank),
        Err(err) => println!("Rank was not updated successfully: Err({})", err),
    }
}

// _________________________________________ TODO add new file?__________________________________________

// for now all users reacts will change the website's rank with +/-1.0
// later this could depend on user's ranks
// TODO more sensible name than "val"
#[derive(PartialEq)]
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

    // TODO shouldn't give val when checking if react_type matches React::Upvode/Downvote
	else if websites_saved.len() == 1 && ((websites_saved[0].rank <= 9.0 && react_type == React::Upvote {val: 0.0}) || (websites_saved[0].rank >= -9.0 && react_type == React::Downvote {val: 0.0})) {
        println!("{:?}'s old rank: {}", websites_saved[0].id, websites_saved[0].rank);
        websites_saved[0].rank += match react_type {
            React::Upvote { val } => 1.0,
            React::Downvote { val } => -1.0,
        };
    }
    else if websites_saved.len() != 1 {
        return Err(ReactError::InvalidArgument { mes: "Vector is not empty and has a size != 1.".to_string() });
    }

    else {
        return Err(ReactError::GenericError);
    }
    crawler::analyse_website(&url, &websites_saved, &conn, &settings);

    if websites_saved.is_empty() {
        return Err(ReactError::RankNotUpdated { mes: "Url has not been analysed previously, so its rank was set to 0.".to_string() });
    }
    Ok(websites_saved[0].rank)
    // TODO test errors
}
