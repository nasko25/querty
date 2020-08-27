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

    println!("{:?}", req(&settings));

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
    // TODO the TempWebsite struct is no longer needed; just add #[serde(...)] to the Website struct
    #[serde(deserialize_with = "from_str")]
    id: Option<i32>,
    title: String,
    metadata: String,
    text: String,
    url: String,
    rank: i32,
    type_of_website: String
}

// CITATION: https://github.com/serde-rs/json/issues/317#issuecomment-300251188
fn from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    match T::from_str(&s).map_err(de::Error::custom) {
        Ok(r) => return Ok(Some(r)),
        Err(err) => return Err(err)
    }
}


#[tokio::main]
async fn req(settings: &settings::Settings) -> Result<(), reqwest::Error> {
    let solr = &settings.solr;
    println!("Solr config: {:?}", solr);

    let method = "select";
    let query = "*:*";
    // TODO more options
    let url =  format!("http://{}:{}/solr/{}/{}?q={}", &solr.server, &solr.port, &solr.collection, &method, &query);

    println!("{}", reqwest::get(&url).await?.text().await?);
    let res: Response = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    println!("Result: {:?}", res);

    Ok(())
}
