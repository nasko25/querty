extern crate config;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;

mod settings;
mod schema;
mod db;
mod solr;

use db::Website;
use db::User;
use db::DB;
use db::Metadata;
use db::ExternalLink;
use db::WebsiteRefExtLink;

use solr::WebsiteSolr;

use solr::req;
use solr::insert;
use solr::update_metadata;
use solr::update_ext_links;
// TODO tmp ----------------------------------------
use diesel::prelude::*;
use crate::schema::metadata;
use crate::schema::external_links;
use crate::schema::website_ref_ext_links;
// -------------------------------------------------

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

    if let DB::Website(mut website) = db::Database::insert(&w, &conn).unwrap() {
        println!("{:?}", website.id);
        println!("Inserted website: {:?}", website);
        website.rank = 7.0;
        println!("Website rank should be updated: {:?}", db::Database::update(&Some(website.clone()), &conn));
        let website_solr = WebsiteSolr {id: website.id, title: website.title, text: website.text, url: website.url, rank: website.rank, type_of_website: website.type_of_website, metadata: None, external_links: None};
        println!("Inserted in Solr: {:?}", insert(&settings, &website_solr));
    }
    // println!("{:?}", db::Database::insert(&u, &conn));
    // println!("{:?}", db::Database::insert(&w, &conn));

    println!("{:?}", req(&settings, "*:*".to_string()));

    // TODO extract as a select methods in DB
    // let mut website_ids = crate::schema::website::dsl::website.filter(crate::schema::website::dsl::id.eq(110)).load::<Website>(&conn).expect("Error loading website");
    let mut website_ids = db::Database::select_w(&Some(vec![ 110 ]), &conn);
    let md = db::Database::select_m(&Some(website_ids.clone()), &conn);
    println!("Metadata: {:?}", &md);

    println!("All websites: {:?}", db::Database::select_w(&None, &conn));

    let mut website_solr_vec = req(&settings, format!("id:{}", website_ids.get(0).unwrap().id.unwrap())).unwrap();
    let mut website_solr = website_solr_vec.get(0).unwrap();
    println!("\n\nUpdate metadata: {:?}", update_metadata(&settings, &md, &website_solr));

    website_ids = db::Database::select_w(&Some(vec![ 109 ]), &conn);

    let ext_links = db::Database::select_el(&website_ids.get(0), &conn);
    println!("External Links: {:?}", ext_links);
    website_solr_vec = req(&settings, format!("id:{}", website_ids.get(0).unwrap().id.unwrap())).unwrap();
    website_solr = website_solr_vec.get(0).unwrap();
    println!("\nUpdate external links: {:?}", update_ext_links(&settings, &ext_links, &website_solr));

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
