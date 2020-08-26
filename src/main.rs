extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;

mod settings;
mod schema;
mod db;

use db::Website;
use db::User;
use db::DB;

// TODO separate file requests
use reqwest;

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

    let w = DB::Website(Website { id: None, title: "".to_string(), metadata: "".to_string(), text: "This is a website for some things".to_string(), url: "".to_string(), rank: 3, type_of_website: "".to_string() });
    // let mut vals_inserted = db::Database::insert_w(&w, &conn);
    // println!("values inserted: {:?}", vals_inserted);

    // TODO handle duplicate usernames -> throw an error
    let u = DB::User(User {id: None, username: "asdf".to_string(), rank: 1.123123, CountryISO_A2: "EN".to_string()});
    // vals_inserted = db::Database::insert_u(&u, &conn);
    // println!("user values inseted: {:?}", vals_inserted);

    if let DB::Website(website) = db::Database::insert(&w, &conn).unwrap() {
        println!("{:?}", website.id);
    }
    println!("{:?}", db::Database::insert(&u, &conn));
    println!("{:?}", db::Database::insert(&w, &conn));

    println!("{:?}", req());

}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    #[serde(rename = "responseHeader")]
    response_header: Header,
    response: ResponseBody
}

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    #[serde(rename = "zkConnected")]
    zk_connected: bool,
    status: i8,
    #[serde(rename = "QTime")]
    q_time: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseBody {
    docs: Vec<TempWebsite>
}

use std::str::FromStr;
use std::fmt::Display;
use serde::de::{self, Deserialize, Deserializer};
#[derive(Debug, Serialize, Deserialize)]
struct TempWebsite {
    // TODO since id is a string, the conversion to an object fails; maybe try casting?
    #[serde(deserialize_with = "from_str")]
    id: i32,
    title: String,
    metadata: String,
    text: String,
    url: String,
    rank: i32,
    type_of_website: String
}

// CITATION: https://github.com/serde-rs/json/issues/317#issuecomment-300251188
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}


#[tokio::main]
async fn req() -> Result<(), reqwest::Error> {
    println!("{}", reqwest::get("http://localhost:8983/solr/querty/select?q=*:*").await?.text().await?);
    let res: Response = reqwest::Client::new()
        .get("http://localhost:8983/solr/querty/select?q=*:*")
        .send()
        .await?
        .json()
        .await?;

    println!("Result: {:?}", res);

    Ok(())
}
