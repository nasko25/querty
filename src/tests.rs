use crate::db;
use crate::settings;
use crate::solr;

use crate::db::Website;
use crate::db::User;
use crate::db::DB;
use crate::db::Metadata;
use crate::db::ExternalLink;
use crate::db::WebsiteRefExtLink;

use crate::solr::WebsiteSolr;

use crate::solr::req;
use crate::solr::insert;
use crate::solr::update_metadata;
use crate::solr::update_ext_links;

use std::error::Error;
// TODO tmp ----------------------------------------
use diesel::prelude::*;
// -------------------------------------------------

pub fn test_all(settings: &settings::Settings, conn: &MysqlConnection) -> Result<(), Box<dyn Error>> {
    let creat_website = db::Database::create_tables(conn);
    println!("table website created: {:?}", creat_website);

    let w = DB::Website(Website { id: None, title: "".to_string(), text: "This is a website for some things".to_string(), url: "".to_string(), base_url: "".to_string(), rank: 3.0, type_of_website: "".to_string() });
    // let mut vals_inserted = db::Database::insert_w(&w, conn);
    // println!("values inserted: {:?}", vals_inserted);

    // TODO handle duplicate usernames -> throw an error
    let u = DB::User(User {id: None, username: "asdf".to_string(), rank: 1.123123, country_iso_a2: "EN".to_string()});
    // vals_inserted = db::Database::insert_u(&u, conn);
    // println!("user values inseted: {:?}", vals_inserted);

    if let DB::Website(mut website) = db::Database::insert(&w, conn).unwrap() {
        println!("{:?}", website.id);
        println!("Inserted website: {:?}", website);
        website.rank = 7.0;
        println!("Website rank should be updated: {:?}", db::Database::update(&DB::Website(website.clone()), conn));
        let website_solr = WebsiteSolr {id: website.id, title: website.title, text: website.text, url: website.url, base_url: website.base_url, rank: website.rank, type_of_website: website.type_of_website, metadata: None, external_links: None};
        println!("Inserted in Solr: {:?}", insert(settings, &website_solr));
    }
    // println!("{:?}", db::Database::insert(&u, conn));
    // println!("{:?}", db::Database::insert(&w, conn));

    println!("{:?}", req(settings, "*:*".to_string()));

    // let mut website_ids = crate::schema::website::dsl::website.filter(crate::schema::website::dsl::id.eq(110)).load::<Website>(conn).expect("Error loading website");
    let mut website_ids = db::Database::select_w(&Some(vec![ 2 ]), conn);
    let md = db::Database::select_m(&Some(website_ids.clone()), conn);
    println!("Metadata: {:?}", &md);

    println!("All websites: {:?}", db::Database::select_w(&None, conn));

    let mut website_solr_vec = req(settings, format!("id:{}", website_ids.get(0).unwrap().id.unwrap())).unwrap();
    let mut website_solr = website_solr_vec.get(0).unwrap();
    println!("\n\nUpdate metadata: {:?}", update_metadata(settings, &md, &website_solr));

    website_ids = db::Database::select_w(&Some(vec![ 1 ]), conn);

    let ext_links = db::Database::select_el(&website_ids.get(0), conn);
    println!("External Links: {:?}", ext_links);
    website_solr_vec = req(settings, format!("id:{}", website_ids.get(0).unwrap().id.unwrap())).unwrap();
    website_solr = website_solr_vec.get(0).unwrap();
    println!("\nUpdate external links: {:?}", update_ext_links(settings, &ext_links, &website_solr));

    // ---------------------------------------------------------------------------------------------------------------------------------------------------------------
    // some insert tests
    let m = DB::Metadata(Metadata {id: None, metadata_text: "some metadata text".to_string(), website_id: website_ids.get(0).unwrap().id});
    println!("Metadata should be inserted: {:?}", db::Database::insert(&m, conn));

    let m_err = DB::Metadata(Metadata {id: None, metadata_text: "some metadata text".to_string(), website_id: Some(200)});
    println!("Metadata insert should trow a foreign key violation: {:?}", db::Database::insert(&m_err, conn));

    let e_l = DB::ExternalLink(ExternalLink {id: None, url: "http://example.com/asdf/@usr/$".to_string()});
    println!("External Link should be inserted: {:?}", db::Database::insert(&e_l, conn));

    let w_r_e_l = DB::WebsiteRefExtLink(WebsiteRefExtLink {id: None, website_id: Some(2), ext_link_id: Some(2)});
    println!("Website reference external link should be inserted: {:?}", db::Database::insert(&w_r_e_l, conn));

    let w_r_e_l_err = DB::WebsiteRefExtLink(WebsiteRefExtLink {id: None, website_id: Some(200), ext_link_id: Some(300)});
    println!("WebsiteRefExtLink insert should throw a foreign key violation: {:?}", db::Database::insert(&w_r_e_l_err, conn));

    Ok(())
}

pub fn reset_db_state(conn: &MysqlConnection, settings: &settings::Settings) -> Result<(), Box<dyn Error>> {
    // delete the databases
    solr::delete_collection(settings)?;
    db::Database::drop_tables(conn)?;

    // create the solr collection and db tables
    solr::create_collection(settings)?;
    db::Database::create_tables(conn)?;

    // import data from the database
    // right now this will do nothing, because the db was just created,
    // but if at some point we need to reindex solr, or reset only solr,
    // the dataimport function will import everything from the db to solr.
    solr::dataimport(settings)?;
    Ok(())
}
