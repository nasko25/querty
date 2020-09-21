use reqwest;
use scraper::{Html, Selector};

use diesel::MysqlConnection;

use crate::solr::WebsiteSolr;
use crate::solr::update_metadata;
use crate::solr::req;
use crate::db::Website;
use crate::db::Metadata;
use crate::db::ExternalLink;
use crate::db::WebsiteRefExtLink;
use crate::settings::Settings;


pub fn analyse_website(url: &str, websites_saved: &Vec<WebsiteSolr>, conn: &MysqlConnection, settings: &Settings) -> Result<(), reqwest::Error> {
    let body = fetch_url(url, conn, settings).unwrap();

    // website_type(&body, meta);

    // TODO temporary for testing; remove when done
    let w = extract_website_info(&body, &url); // TODO could call this in the save_website_info function
    let website = save_website_info(w, &conn, &settings).unwrap();
    // should get the id from the save_website_info() function
    let website_id = website.id;
    let meta = extract_metadata_info(&body, website_id);
    let ext_links = extract_external_links(&body, website_id);
    let website_solr_vec = req(&settings, format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
    let website_solr = website_solr_vec.get(0).unwrap();
    save_metadata(meta, website_solr, &conn, &settings);

    // TODO if it is not empty, update the website(s) in it
    if websites_saved.is_empty() {
        // save_website_info(&body, &url, &conn, &settings);
    }
    Ok(())
}
// TODO conn and settings should probably not be passed here
// TODO return calculated rank of the website
#[tokio::main]
async fn fetch_url(url: &str, conn: &MysqlConnection, settings: &Settings) -> Result<String, reqwest::Error> {
    // await is not necessary
    let res = reqwest::get(url).await?;
    assert!(res.status().is_success());

    let body = res.text().await?;

    Ok(body)
}

fn extract_website_info(body: &str, url: &str) -> Website {
    let fragment = Html::parse_document(body);
    let mut selector = Selector::parse("title").unwrap();

    // if there are multiple titles, only the first is used
    let title_raw = fragment.select(&selector).next().unwrap().inner_html();
    let title = title_raw.trim();
    // println!("Website title: {:?}", title);

    // select all text
    selector = Selector::parse("body").unwrap();
    let text = fragment.select(&selector).next().unwrap().text().collect::<Vec<_>>();
    // println!("\nWebsite body text: {:?}", text);

    // trim the trailing and leading white spaces
    let mut trimmed_text = Vec::new();
    let mut trimmed_element;
    for element in text {
        trimmed_element = element.trim();
        if trimmed_element != "" {
            trimmed_text.push(trimmed_element);
        }
    }
    // println!("\nWebsite body text trimmed: {:?}", trimmed_text.join(", "));

    Website { id: None, title: title.to_string(), text: trimmed_text.join(", "), url: url.to_string(), rank: 1.0, type_of_website: "default".to_string() }
}

fn extract_metadata_info(body: &str, website_id: Option<u32>) -> Vec<Metadata> {
    let fragment = Html::parse_document(body);
    let selector = Selector::parse("meta").unwrap();
    // println!("Selected meta tags: {:?}", fragment.select(&selector));
    // meta tags can provide info for the type of website
    // TODO content of meta tag can have capital letters -> case insensitive search for "article"
    let mut metas = Vec::new();
    for element in fragment.select(&selector) {
        // iterate over each meta tag's attributes
        for attr in element.value().attrs() {
            // attr is a (&str, &str) tuple
            // the first string is the meta tag attribute and will be ignored, because it does not convey any useful information
            // (can later be saved in the db in a new "attribute" column, but it is not needed for now)
            // the second string is the actual value stored in the meta tag, and it will be saved
            metas.push(Metadata { id: None, metadata_text: attr.1.to_string(), website_id: website_id });
        }
        // println!("element.value(): {:?}, element charset: {:?}, element name: {:?}, element content: {:?}, element.value.name: {:?}", 
        //     element.value(), element.value().attr("charset"), element.value().attr("name"), element.value().attr("content"), element.value().name());
    }
    metas
}

fn extract_external_links(body: &str, website_id: Option<u32>) -> Vec< (ExternalLink, WebsiteRefExtLink) > {
    let fragment = Html::parse_document(body);
    let selector = Selector::parse("a").unwrap();

    let mut ext_links = Vec::new();
    let mut href;
    for element in fragment.select(&selector) {
        href = element.value().attr("href");
        match href {
            Some(l) => {
                                                                                                                // TODO change ext_link_id when the ExternalLink is inserter in the database
                ext_links.push( (ExternalLink { id: None, url: l.to_string() }, WebsiteRefExtLink { id: None, website_id: website_id, ext_link_id: None }) )
            },
            None => (),
        }
    }
    ext_links
}

// returns the Website saved to the database
fn save_website_info(website_to_insert: Website, conn: &MysqlConnection, settings: &Settings) -> Result<Website, throw::Error<&'static str>> {
    // TODO
    // first save the website info(meta tags, title, text, etc.) in the database, and if it is successful (check!) then add it to solr
    // (because the database should (eventually) have a unique constraint on url)
    // if the website cannot be inserted in the database, throw an error

    // TODO save metadata and external_links
    let w = crate::db::DB::Website (website_to_insert);
    if let crate::db::DB::Website(website) = crate::db::Database::insert(&w, conn).unwrap() {
        let w_solr = WebsiteSolr {id: website.id, title: website.title.clone(), text: website.text.clone(), url: website.url.clone(), rank: website.rank, type_of_website: website.type_of_website.clone(), metadata: None, external_links: None };
        crate::solr::insert(settings, &w_solr);
        println!("{:?}", website.id);
        Ok(website.clone())
    }
    else {
        throw_new!("Could not insert website in the database");
    }
}

fn save_metadata(metadata_vec: Vec<Metadata>, website_to_update: &WebsiteSolr, conn: &MysqlConnection, settings: &Settings) -> Result<Vec<Metadata>, throw::Error<&'static str>> {
    let mut m;
    let mut metadata_solr = Vec::new();
    for metadata in metadata_vec {
        m = crate::db::DB::Metadata (metadata);
        if let crate::db::DB::Metadata (meta) = crate::db::Database::insert(&m, conn).unwrap() {
            println!("meta id: {:?}", meta.id);
            metadata_solr.push(meta);
        }
        else {
            throw_new!("Could not insert metadata in the database");
        }
    }
    update_metadata(settings, &metadata_solr, website_to_update);
    Ok(metadata_solr)
}

// TODO javascript analysis -> execute javascript somehow? and check for popups, keywords that help determine website type, etc.
// TODO different languages?

// TODO train a model to guess the type of website (feed in the html document and classify its type)
//          - http://www.cse.lehigh.edu/~brian/pubs/2007/classification-survey/LU-CSE-07-010.pdf
//              -> this looks like a good source to use on web page classification
//              -> it also contains some optimization options that can help speed up the web page analysis
fn website_type<'a>(body: &str, meta: &'a Vec<&str>) -> &'a str {
    let body_lc = body.to_lowercase();
    // let mut meta_lc;

    // for m in meta {
    //     meta_lc = m.to_lowercase();
    //     if meta_lc.contains("article") {
    //         return "article";
    //     }
    // }
    // TODO also check meta tags for website type
    if (body_lc.contains("install") && body_lc.contains("version")) || body_lc.contains("maintained") || body_lc.contains("develop") {
        // product websites's rank should be mainly determined by users's reviews, users's interactions with the website and how many other websites link to this domain
        return "product";
    }
    else if body_lc.contains("author") || body_lc.contains("article") {
        // rank should additionally be determined by the quality of the article
        // (why was the article written -> are there too many ads and a short article
        //                              -> do reviews downvote it a lot
        //                              -> is there a "subscribe to our newsletter"
        //                              -> popups, etc.)
        return "article";
    }
    // TODO else if...
    return "default";
}