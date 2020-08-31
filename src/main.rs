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
use db::Metadata;
use db::ExternalLink;
use db::WebsiteRefExtLink;

// TODO tmp ----------------------------------------
use diesel::prelude::*;
use crate::schema::metadata;
use crate::schema::external_links;
use crate::schema::website_ref_ext_links;
// -------------------------------------------------

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

    let conn = db::Database::establish_connection(&url);

    // TODO foreign keys
    let creat_website = db::Database::create_tables(&conn);
    println!("table website created: {:?}", creat_website);

    let w = DB::Website(Website { id: None, title: "".to_string(), text: "This is a website for some things".to_string(), url: "".to_string(), rank: 3.0, type_of_website: "".to_string() });
    // let mut vals_inserted = db::Database::insert_w(&w, &conn);
    // println!("values inserted: {:?}", vals_inserted);

    // TODO handle duplicate usernames -> throw an error
    let u = DB::User(User {id: None, username: "asdf".to_string(), rank: 1.123123, country_iso_a2: "EN".to_string()});
    // vals_inserted = db::Database::insert_u(&u, &conn);
    // println!("user values inseted: {:?}", vals_inserted);

    // if let DB::Website(website) = db::Database::insert(&w, &conn).unwrap() {
    //     println!("{:?}", website.id);
    // }
    // println!("{:?}", db::Database::insert(&u, &conn));
    // println!("{:?}", db::Database::insert(&w, &conn));

    println!("{:?}", req(&settings));

    // TODO extract as a select methods in DB
    let mut website_ids = crate::schema::website::dsl::website.filter(crate::schema::website::dsl::id.eq(110)).load::<Website>(&conn).expect("Error loading website");
    let md = metadata::table.filter(metadata::website_id.eq(website_ids.get(0).unwrap().id)).load::<Metadata>(&conn).expect("Error loading metadata");
    println!("{:?}", &md);

    website_ids = crate::schema::website::dsl::website.filter(crate::schema::website::dsl::id.eq(109)).load::<Website>(&conn).expect("Error loading website");
    let link_ids = WebsiteRefExtLink::belonging_to(website_ids.get(0).unwrap()).select(website_ref_ext_links::ext_link_id).load::<Option<u32>>(&conn).expect("");

    let ext_links = external_links::table.filter(external_links::id.eq(link_ids.get(0).unwrap())).load::<ExternalLink>(&conn).expect("Error loading external links.");
    println!("External Links: {:?}", ext_links);

    // ---------------------------------------------------------------------------------------------------------------------------------------------------------------
    // some insert tests
    let m = DB::Metadata(Metadata {id: None, metadata_text: "some metadata text".to_string(), website_id: website_ids.get(0).unwrap().id});
    println!("Metadata should be inserted: {:?}", db::Database::insert(&m, &conn));

    let m_err = DB::Metadata(Metadata {id: None, metadata_text: "some metadata text".to_string(), website_id: Some(9)});
    println!("Metadata insert should trow a foreign key violation: {:?}", db::Database::insert(&m_err, &conn));

    let e_l = DB::ExternalLink(ExternalLink {id: None, url: "http://example.com/asdf/@usr/$".to_string()});
    println!("External Link should be inserted: {:?}", db::Database::insert(&e_l, &conn));

    let w_r_e_l = DB::WebsiteRefExtLink(WebsiteRefExtLink {id: None, website_id: Some(110), ext_link_id: Some(2)});
    println!("Website reference external link should be inserted: {:?}", db::Database::insert(&w_r_e_l, &conn));

    let w_r_e_l_err = DB::WebsiteRefExtLink(WebsiteRefExtLink {id: None, website_id: Some(200), ext_link_id: Some(300)});
    println!("WebsiteRefExtLink insert should throw a foreign key violation: {:?}", db::Database::insert(&w_r_e_l_err, &conn));
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
    #[serde(rename = "numFound")]
    num_found: i64,
    start: i32,
    #[serde(rename = "maxScore")]
    max_score: f32,
    #[serde(rename = "numFoundExact")]
    num_found_exact: bool,
    docs: Vec<Website>
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
