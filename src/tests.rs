use crate::db;
use crate::settings;
use crate::solr;

use crate::crawler::test_crawler;
use crate::crawler::Crawler;

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

// TODO split the function into multiple smaller functions
pub fn test_all(url: &str, settings: &settings::Settings, conn: &MysqlConnection) -> Result<(), Box<dyn Error>> {
    // reset the state of the database before executing the tests
    assert!(reset_db_state(&conn, &settings).is_ok(), "The detabase cannot be reset. Try resetting it manually.");

    assert!(test_crawler(url, conn, settings).is_ok(), "The crawler tests failed.");

    let create_website = db::Database::create_tables(conn);
    println!("table website created: {:?}", create_website);
    assert!(create_website.is_ok(), "Could not create tables in the db.");

    let w = DB::Website(Website { id: None, title: "".to_string(), text: "This is a website for some things".to_string(), url: "".to_string(), base_url: "".to_string(), rank: 3.0, type_of_website: "".to_string() });
    // let mut vals_inserted = db::Database::insert_w(&w, conn);
    // println!("values inserted: {:?}", vals_inserted);

    // TODO handle duplicate usernames -> throw an error
    let u = DB::User(User {id: None, username: "asdf".to_string(), rank: 1.123123, country_iso_a2: "EN".to_string()});
    // vals_inserted = db::Database::insert_u(&u, conn);
    // println!("user values inseted: {:?}", vals_inserted);
    let user = db::Database::insert(&u, conn);
    assert!(user.is_ok(), "User could not be inserted in the database.");
    // TODO add a select_u() function in the database and ensure that the user is there.

    if let DB::Website(mut website) = db::Database::insert(&w, conn).unwrap() {
        println!("{:?}", website.id);
        println!("Inserted website: {:?}", website);
        // assert!(website.is_ok(), "Could not insert website with id {:?} in the database.", website.id);
        website.rank = 7.0;
        let updated_website = db::Database::update(&DB::Website(website.clone()), conn);
        println!("Website rank should be updated: {:?}", updated_website);
        assert!(updated_website.is_ok(), "Could not update website with id: {:?}", website.id);
        let website_solr = WebsiteSolr {id: website.id, title: website.title, text: website.text, url: website.url, base_url: website.base_url, rank: website.rank, type_of_website: website.type_of_website, metadata: None, external_links: Some(vec!["spacex.com".to_string(), "rust-lang.org".to_string()])};
        let solr_inserted = insert(settings, &website_solr);
        println!("Inserted in Solr: {:?}", solr_inserted);
        assert!(solr_inserted.is_ok(), "Could not insert website with id {:?} into solr.", website.id);
    }
    // println!("{:?}", db::Database::insert(&u, conn));
    // println!("{:?}", db::Database::insert(&w, conn));

    let all_websites_solr = req(settings, "*:*".to_string());
    println!("ALL WEBSITES FROM SOLR:\n\n{:?}", all_websites_solr);
    assert!(all_websites_solr.is_ok(), "Could not fetch all websites from solr.");

    // let mut website_ids = crate::schema::website::dsl::website.filter(crate::schema::website::dsl::id.eq(110)).load::<Website>(conn).expect("Error loading website");
    let mut website_ids = db::Database::select_w(&Some(vec![ 2 ]), conn);
    assert!(!website_ids.is_empty(), "No websites in the database with the given ids.");
    let md = db::Database::select_m(&Some(website_ids.clone()), conn);
    println!("Metadata: {:?}", &md);
    assert!(md.is_empty(), "Vector with metadata of websites with ids: {:?} is not empty.", website_ids);

    let all_websites = db::Database::select_w(&None, conn);
    println!("All websites: {:?}", all_websites);
    assert!(!all_websites.is_empty(), "No websites were retrieved from the database.");

    let mut website_solr_vec = req(settings, format!("id:{}", website_ids.get(0).unwrap().id.unwrap())).unwrap();
    let mut website_solr = website_solr_vec.get(0).unwrap();
    let updated_metadata = update_metadata(settings, &md, &website_solr);
    println!("\n\nUpdate metadata: {:?}", updated_metadata);
    assert!(updated_metadata.is_ok(), "Could not update metadata of website with id: {:?}", website_ids.get(0));

    website_ids = db::Database::select_w(&Some(vec![ 1 ]), conn);
    assert!(!website_ids.is_empty(), "No websites in the database with the given ids.");

    let ext_links = db::Database::select_el(&website_ids.get(0), conn);
    println!("External Links: {:?}", ext_links);
    assert!(!ext_links.is_empty(), "Websites with ids: {:?} have no external links.", website_ids.get(0));
    website_solr_vec = req(settings, format!("id:{}", website_ids.get(0).unwrap().id.unwrap())).unwrap();
    website_solr = website_solr_vec.get(0).unwrap();
    let updated_ext_links = update_ext_links(settings, &ext_links, &website_solr);
    println!("\nUpdate external links: {:?}", updated_ext_links);
    assert!(updated_ext_links.is_ok(), "Could not update external links of website with id: {}", website_ids.get(0).unwrap().id.unwrap());

    // ---------------------------------------------------------------------------------------------------------------------------------------------------------------
    // some insert tests
    let m = DB::Metadata(Metadata {id: None, metadata_text: "some metadata text".to_string(), website_id: website_ids.get(0).unwrap().id});
    let metadata_inserted = db::Database::insert(&m, conn);
    println!("Metadata should be inserted: {:?}", metadata_inserted);
    assert!(metadata_inserted.is_ok(), "Metadata of website with id {:?} was not inserted.", website_ids.get(0).unwrap().id);

    let m_err = DB::Metadata(Metadata {id: None, metadata_text: "some metadata text".to_string(), website_id: Some(200)});
    let metadata_inserted_err = db::Database::insert(&m_err, conn);
    println!("Metadata insert should trow a foreign key violation: {:?}", metadata_inserted_err);
    assert!(metadata_inserted_err.is_err(), "Metadata of website with non-existent id did not throw a foreign key violation.");

    let e_l = DB::ExternalLink(ExternalLink {id: None, url: "http://example.com/asdf/@usr/$".to_string()});
    let e_l_inserted = db::Database::insert(&e_l, conn);
    println!("External Link should be inserted: {:?}", e_l_inserted);
    assert!(e_l_inserted.is_ok(), "External links were not inserted in the database.");

    let e_l_id = match e_l_inserted {
        Ok(DB::ExternalLink(el)) => el.id,
        _ => panic!("e_l_inserted has an unexpected type")
    };

    assert_eq!(e_l_id.unwrap(), 9, "This was just for testing. There is no need the id of the inserted external link should be 9. This assertion can be safely removed.");

    let w_r_e_l = DB::WebsiteRefExtLink(WebsiteRefExtLink {id: None, website_id: Some(2), ext_link_id: e_l_id});
    let w_r_e_l_inserted = db::Database::insert(&w_r_e_l, conn);
    println!("Website reference external link should be inserted: {:?}", w_r_e_l_inserted);
    match w_r_e_l {
        DB::WebsiteRefExtLink(wrel) => assert!(w_r_e_l_inserted.is_ok(), "Website ref external link of website with id: {:?} and External links with id: {:?} was not inserted in the database.", wrel.website_id, wrel.ext_link_id),
        _ => panic!("w_r_e_l has wrong enum type")
    };

    let w_r_e_l_err = DB::WebsiteRefExtLink(WebsiteRefExtLink {id: None, website_id: Some(200), ext_link_id: Some(300)});
    let w_r_e_l_inserted_err = db::Database::insert(&w_r_e_l_err, conn);
    println!("WebsiteRefExtLink insert should throw a foreign key violation: {:?}", w_r_e_l_inserted_err);
    assert!(w_r_e_l_inserted_err.is_err(), "Website ref external link of a website with non-existent id and external link with non-existent id did not throw a foreign key violation.");

    // reset the state of the db and solr after the tests are done
    // reset_db_state(&conn, &settings);

    // delete metatags from the database
    let mut del_result = db::Database::delete_m_by_id(&vec![1, 2, 3], conn);
    assert!(del_result.is_ok());
    // 3 entries should be deleted from the database
    assert_eq!(del_result.unwrap(), 3);

    // assert that the meta tags with ids 1, 2, 3 were deleted from the database
    assert_eq!(db::Database::select_m_by_id(&Some(vec![ 1, 2, 3 ]), conn).len(), 0);
    assert_eq!(db::Database::select_m_by_id(&Some(vec![ 1, 2, 3, 4 ]), conn).len(), 1);

    del_result = db::Database::delete_m_by_id(&vec![0, 1, 4], conn);
    assert!(del_result.is_ok());
    // only 1 entry should be deleted from the database
    assert_eq!(del_result.unwrap(), 1);
    // assert that the meta tag with id 4 is no longer present in the database
    assert_eq!(db::Database::select_m_by_id(&Some(vec![ 1, 2, 3, 4 ]), conn).len(), 0);


    // db::Database::select_w(&None, conn).get(0).unwrap().id       // it is = 1
    // print!("{:?}", db::Database::select_m(&Some(vec![ Website{ id: Some(1), base_url: "".to_string(), rank: 0.0, text: "".to_string(), title: "".to_string(), type_of_website: "".to_string(), url: "".to_string() } ]), conn));

    // create a Crawler struct
    let crawler = Crawler {
        conn: &conn,
        settings: &settings
    };
    // test the update website functionality (update metadata and external links as well)
    assert!(crawler.test_website_update(&solr::WebsiteSolr { id: Some(1), base_url: "updated url".to_string(), external_links: Some(vec!["example.com".to_string(), "updated_url.asdf".to_string()]), metadata: Some(vec!["asdf".to_string(), "updated meta".to_string(), "asdfadsf".to_string()]), rank: -2.0_f64, text: "this is the updated website text".to_string(), title: "Updated title 2.0".to_string(), type_of_website: "updated".to_string(), url: "updated_url.new".to_string()}).is_ok(), "crawler.test_website_update() for a website with a valid id should return Ok");

    // there should be only 3 metadata entries after the update
    assert_eq!(db::Database::select_m(&Some(vec![ Website{ id: Some(1), base_url: "".to_string(), rank: 0.0, text: "".to_string(), title: "".to_string(), type_of_website: "".to_string(), url: "".to_string() } ]), conn).len(), 3, "Number of metadata entries in the database is wrong after the update.");

    // there should be only 2 external link entries after the update
    assert_eq!(db::Database::select_el(&Some( &Website{ id: Some(1), base_url: "".to_string(), rank: 0.0, text: "".to_string(), title: "".to_string(), type_of_website: "".to_string(), url: "".to_string() } ), conn).len(), 2, "Number of external link entries in the database is wrong after the update.");

    //std::process::exit(1);

    // test delete_m()
    // first try to delete metadatas that are linked to website with id equal to 2 (there are no
    // such meta tags in the database)
    del_result = db::Database::delete_m(&vec![ 2 ], conn);
    assert!(del_result.is_ok());
    assert_eq!(del_result.unwrap(), 0, "There should be no metadata associated with the website with id = 2.");

    // delete all metadatas linked to the website with id equal to 1
    // first get them from the db to assert they were deleted:
    let metadatas_in_db = db::Database::select_m(&Some(vec![ Website{ id: Some(1), base_url: "".to_string(), rank: 0.0, text: "".to_string(), title: "".to_string(), type_of_website: "".to_string(), url: "".to_string() } ]), conn);

    // this is not necessarily true; it depends on the website with id 1;
    // the meta tags for that website are updated somewhere above, so it should for now always have 3 meta tag entries
    assert!(metadatas_in_db.len() > 0, "There should be some metadata associated with the website with id equal to 1.");
    // actually delete them
    del_result = db::Database::delete_m(&vec![ 1 ], conn);
    assert!(del_result.is_ok());
    assert_eq!(del_result.unwrap(), metadatas_in_db.len(), "The number of deleted metadata entries should be the same as the number of metadata entries that were associated with the website with id equal to 1 before.");

    assert!(test_suggester(&settings).is_ok(), "Suggester tests failed.");

    Ok(())
}

fn test_suggester(settings: &settings::Settings) -> Result<(), Box<dyn Error>> {
    assert!(solr::suggest("a".to_string(), &settings).is_err(), "Letters should be longer than 2 characters");

    match solr::suggest("random string word asdfsdfjkn 0".to_string(), &settings) {
        Ok(terms) => assert!(terms.is_empty(), "There should be no suggestion for the random string above"),
        Err(err) => return Err(err)
    };
    let suggestion = solr::suggest("thin".to_string(), &settings);

    match suggestion {
        Ok(terms) => {
            // check if the term "things" is returned by the suggester when searching for "thin"
            // this could fail if the test website is removed from solr
            assert!(terms.iter().any(| term | term.term == "things"), "The term \"things\" is not returned as a suggestion. This could have happened if you have changed what websites have been saved to solr.");
            Ok(())
        },
        Err(err) => Err(err)
    }
}

pub fn reset_db_state(conn: &MysqlConnection, settings: &settings::Settings) -> Result<(), Box<dyn Error>> {
    // delete the databases
    solr::delete_collection(settings)?;
    db::Database::drop_tables(conn)?;

    // create the solr collection and db tables
    solr::create_collection(settings)?;
    db::Database::create_tables(conn)?;

    Ok(())
}

pub fn reindex_solr(settings: &settings::Settings) -> Result<(), Box<dyn Error>> {
    // delete the querty collection
    solr::delete_collection(settings)?;

    // create a new solr collection
    solr::create_collection(settings)?;

    // import the data from the mysql database
    solr::dataimport(settings)?;
    Ok(())
}
