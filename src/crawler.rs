use reqwest;
use scraper::{Html, Selector};

use std::collections::HashSet;

use crate::solr::WebsiteSolr;
use crate::solr::update_metadata;
use crate::solr::update_ext_links;
use crate::solr::req;
use crate::db::DB;
use crate::db::Website;
use crate::db::Metadata;
use crate::db::ExternalLink;
use crate::db::WebsiteRefExtLink;
// use crate::settings::Settings;

extern crate xmlrpc;
use xmlrpc::Request;
use std::error::Error;

use url::Url;
use publicsuffix::List;

use crate::crawl::add_next_crawl_url;
use std::env;

use futures::executor::block_on;

pub struct Crawler/* <'a> */ {
    // pub conn: &'a MysqlConnection,
    // pub settings: &'a Settings
}

impl /* <'a> */  Crawler /* <'a> */ {

    pub fn analyse_website(&self, url: &str, websites_saved: &Vec<WebsiteSolr>) -> Result<(), Box<dyn Error>> {
        // let body = fetch_url(url).unwrap();
        if let Err(err) = self.rank_from_links(url) {
            return Err(err);
        }

        // if it is not empty, update the website(s) in it
        if websites_saved.is_empty() {
            // save_website_info(&body, &url, &conn, &settings);
            if let Err(err) = self.save_website(&url) {
                return Err(err);
            }
        }
        else {
            // there should be only one website in the vector, because it should have been
            // gotten from its url, and url should be unique
            assert_eq!(websites_saved.len(), 1, "There are {} websites returned by req()", websites_saved.len());
            // select_w first to get a Website, and then db::update
            // also update metadata and external links connected to that website
            if let Err(err) = self.update_website(&websites_saved[0]) {
                return Err(err);
            }
        }

        Ok(())
    }

    // TODO return calculated rank of the website
    #[tokio::main]
    async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
        // await is not necessary
        let res = reqwest::get(url).await?;
        assert!(res.status().is_success());

        let body = res.text().await?;

        Ok(body)
    }

    // returns the rank calculated from all websites that link to the given url
    fn rank_from_links(&self, url: &str) -> Result<f64, Box<dyn Error>> {
        let base_url = match Crawler/* ::<'a>*/::extract_base_url(url) {
            Ok(base_url) => base_url,
            Err(err) => return Err(Box::new(err))
        };

        // get all websites that link to this and have rank >= 0
        // TODO &rows=0 to not return the actual websites, but only get the numFound
        let websites = match req(format!("external_links:\"{}\" AND rank:[0.0 TO *]", base_url)) {
            Ok(websites) => websites,
            Err(err) => return Err(Box::new(err))
        };
        println!("Should return only one website: {:?}", websites);

        // retrun number of websites * a constant
        Ok(1.0)
    }

    fn extract_base_url(url: &str) -> Result<String, publicsuffix::errors::Error> {
        let list = List::fetch().unwrap();  // TODO get public suffix list from path https://docs.rs/publicsuffix/1.5.4/publicsuffix/
        let parsed_url = Url::parse(url)?;
        let host = match parsed_url.host_str() {
            Some(val) => val,
            None => panic!("Problem parsing the url: {:}", url)
        };
        let parsed_domain = list.parse_domain(host)?;
        let parsed_url = match parsed_domain.root() {
            Some(parsed_url) => parsed_url,
            None => panic!("Problem extracting the root of the parsed domain: {:}", parsed_domain)
        };

        Ok(parsed_url.to_string())
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

        // TODO maybe always url encode the url and base_url to ensure that
        // fetching "http://example.com/asd asd" and "http://example.com/asd%20asd" from solr will return
        // the same website
        // also what about urls in different languages: "example.com" and "ехампле.ком"
        // should they be the same url ?
        Website { id: None, title: title.to_string(), text: trimmed_text.join(", "), url: url.to_string(), base_url: Crawler::extract_base_url(url).unwrap(), rank: 0.0, type_of_website: "default".to_string() }
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

    fn extract_external_links(body: &str, website_id: Option<u32>, url: &str) -> Vec< (ExternalLink, WebsiteRefExtLink) > {
        let fragment = Html::parse_document(body);
        // TODO check also <link> html tags
        let selector = Selector::parse("a").unwrap();

        let list = List::fetch().unwrap();  // TODO get public suffix list from path https://docs.rs/publicsuffix/1.5.4/publicsuffix/

        // use a hashset to only save unique domains
        let mut ext_links = HashSet::new();
        let mut href;
        for element in fragment.select(&selector) {
            href = element.value().attr("href");
            match href {
                Some(l) => {
                    let parsed_link = Url::parse(l);
                    let parsed_url = list.parse_domain(Url::parse(url).unwrap().host_str().unwrap()).unwrap();      // TODO should check somewhere if the given url is valid. Probably in fetch_url()
                    match parsed_link.clone() {
                        Ok(val) => {
                            match val.host_str() {
                                Some(host_str) => {
                                    if list.parse_domain(host_str).unwrap().root() != parsed_url.root() {
                                        // only save the base_url, no need to save the whole url
                                        ext_links.insert( (ExternalLink { id: None, url: list.parse_domain(host_str).unwrap().root().unwrap().to_string() }, WebsiteRefExtLink { id: None, website_id: website_id, ext_link_id: None }) );

                                        // save the external links extracted from the website to be
                                        // crawled next
                                        // TODO
                                        match env::var("ADD_EXTERNAL_LINKS_TO_BE_CRAWLED") {
                                            Ok(ref var) if var == "True" => { block_on(add_next_crawl_url(vec![parsed_link.unwrap().into()])).unwrap(); },
                                            Ok(_) => colour::yellow!("Set ADD_EXTERNAL_LINKS_TO_BE_CRAWLED to \"True\" to add extracted external links to be crawled next."),
                                            Err(_) => colour::red!("Environment variable ADD_EXTERNAL_LINKS_TO_BE_CRAWLED is not set. External links extracted from websites will not be crawled."),
                                        }
                                    }
                                    else {
                                        println!("Urls are not equal: {:?} != {:?}", list.parse_domain(val.host_str().unwrap()).unwrap().root(), parsed_url.root());
                                    }
                                },
                                None => println!("Eror: Url \"{}\" does not have a host string.", val),
                            }

                        },
                        Err(err) => { println!("Error when parsing the url {:?}: {:?}", l, err); }
                    };
                },
                None => (),
            }
        }
        ext_links.into_iter().collect()
    }

    // this is a wrapper around the functions that extract and save website info, metadata, and
    // external links
    fn save_website(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let body = Crawler::fetch_url(url).unwrap();
        // let settings = self.settings;

        let mut w = Crawler::extract_website_info(&body, &url);
        let mut meta = Crawler::extract_metadata_info(&body, None); // website id should not matter here, because it is not needed for website_genre_offline_classification() and is later fetched again

        match Crawler/*::<'a>*/::website_genre(&url) {
            Ok(genre) => w.type_of_website = genre,
            Err(err) => {
                println!("Encountered an error while trying to classify the website: {:?}", err);
                println!("Attempting offline classification.");
                w.type_of_website = Crawler::website_genre_offline_classification(&w.text, &meta); // use the extracted text that is saved in solr and the db instead of the raw, unprocessed website body
            }
        }

        let website = self.save_website_info(w).unwrap();

        // should get the id from the save_website_info() function
        let website_id = website.id;

        // meta = self.extract_metadata_info(&body, website_id);

        // after saving the website, the db will generate an id,
        // set this id to every metadata entry before saving it to the database and solr
        // (because the metadata was already extracted above, but the website id was set to None,
        // because it was not yet saved to the db, so it did not have an id)
        meta.iter_mut().for_each(|m| m.website_id = website_id);
        let ext_links = Crawler::extract_external_links(&body, website_id, &url);
        let mut website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        let mut website_solr = website_solr_vec.get(0).unwrap();
        /* let metadata = */ self.save_metadata(&meta, website_solr).unwrap();
        // need to fetch the updated website from solr before updating the external_links,
        // otherwise it would set the metadata that was just updated to null
        website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        website_solr = website_solr_vec.get(0).unwrap();
        /* let external_links = */ self.save_external_links(ext_links, website_solr).unwrap();

        // for now updating the website info will remove the metadata and external links stored in solr
        // maybe don't overwrite them to null?
        // (or fetch the old metadata and external_links and include them when updating the website in
        // solr
        // update_website_info(website, &conn, &settings);

        // again need to fetch the updated website from solr before updating the external_links
        // otherwise it would set the metadata to the old metadata (it is updated above)
        // website_solr_vec = req(&settings, format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        // website_solr = website_solr_vec.get(0).unwrap();

        // update_meta(&metadata, website_solr, &conn, &settings);

        // again need to fetch the updated website from solr before updating the external_links
        // otherwise it would set the metadata to the old metadata (it is updated above)
        // website_solr_vec = req(&settings, format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        // website_solr = website_solr_vec.get(0).unwrap();

        // update_external_links(external_links, website_solr, &conn, &settings);

        Ok(())
    }

    // returns the Website saved to the database
    // or returns an error if the website could not be saved to the database
    fn save_website_info(&self, website_to_insert: Website) -> Result<Website, throw::Error<&'static str>> {
        // first save the website info(meta tags, title, text, etc.) in the database, and only if it was added successfully, add it to solr
            // (because the database should (eventually) have a unique constraint on url)
        let w = crate::db::DB::Website (website_to_insert);
        if let crate::db::DB::Website(website) = crate::db::Database::insert(&w).unwrap() {
            let w_solr = WebsiteSolr {id: website.id, title: website.title.clone(), text: website.text.clone(), url: website.url.clone(), base_url: website.base_url.clone(), rank: website.rank, type_of_website: website.type_of_website.clone(), metadata: None, external_links: None };
            crate::solr::insert(&w_solr).unwrap();
            println!("{:?}", website.id);
            Ok(website.clone())
        }
        // else if the website cannot be inserted in the database, throw an error
        else {
            throw_new!("Could not insert website in the database");
        }
    }

    fn save_metadata(&self, metadata_vec: &Vec<Metadata>, website_to_update: &WebsiteSolr) -> Result<Vec<Metadata>, throw::Error<&'static str>> {
        let mut m;
        let mut metadata_solr = Vec::new();
        for metadata in metadata_vec {
            m = crate::db::DB::Metadata (metadata.clone()); // TODO maybe add a separate table - like for the external links, in order to reuse the already inserted metadatas, instead of inserting them multiple times for different websites.
            if let crate::db::DB::Metadata (meta) = crate::db::Database::insert(&m).unwrap() {
                println!("meta id: {:?}", meta.id);
                metadata_solr.push(meta);
            }
            else {
                throw_new!("Could not insert metadata in the database");
            }
        }
        update_metadata(&metadata_solr, website_to_update).unwrap();
        Ok(metadata_solr)
    }

    fn save_external_links(&self, external_links: Vec< (ExternalLink, WebsiteRefExtLink) >, website_to_update: &WebsiteSolr) -> Result<Vec< (ExternalLink, WebsiteRefExtLink) >, throw::Error<&'static str>> {
        let mut el;
        let mut web_el;
        let mut external_links_solr = Vec::new();
        for mut external_link in external_links {
            el = crate::db::DB::ExternalLink (external_link.0);
            if let crate::db::DB::ExternalLink (ext_link) = crate::db::Database::insert(&el).unwrap() { // TODO add a unique constraint on the ExternalLink in the database, and if you try to insert an already existing ExternalLink to the database, get its id (and use it for the WebsiteRefExtLink) instead of inserting it twice
                external_link.1.ext_link_id = ext_link.id;
                web_el = crate::db::DB::WebsiteRefExtLink (external_link.1);
                if let crate::db::DB::WebsiteRefExtLink (webref_ext_link) = crate::db::Database::insert(&web_el).unwrap() {
                    println!("external link id: {:?}; website ref external link id: {:?}; website ref external link link id (should be = to external link id): {:?}", ext_link.id, webref_ext_link.id, webref_ext_link.ext_link_id);
                    external_links_solr.push( (ext_link, webref_ext_link) );
                }
                else {
                    throw_new!("Could not insert website ref external link in the database.");
                }
            }
            else {
                throw_new!("Could not insert external link in the database.");
            }
        }
        update_ext_links(&external_links_solr.iter().map(|(e_l, _w_ref_e_l)| e_l.clone()).collect::<Vec<ExternalLink>>(), website_to_update).unwrap();
        Ok(external_links_solr)
    }

    // wrapper around the functions that extract and update website info, metadata, and
    // external links
    // like save_website() but for updating
    fn update_website(&self, website: &WebsiteSolr) -> Result<(), Box<dyn Error>> {
        // TODO too similar to save_website()
        // extract common code
        let url = &website.url;
        let body = Crawler/*::<'a>*/::fetch_url(&url).unwrap();

        let website_id = website.id;
        let mut w = Crawler/*::<'a>*/::extract_website_info(&body, &url);
        w.id = website_id;
        w.rank = website.rank;
        let extracted_meta = Crawler/*::<'a>*/::extract_metadata_info(&body, None);

        match Crawler/*::<'a>*/::website_genre(&url) {
            Ok(genre) => w.type_of_website = genre,
            Err(err) => {
                println!("Encountered an error while trying to classify the website: {:?}", err);
                println!("Attempting offline classification.");
                w.type_of_website = Crawler::website_genre_offline_classification(&w.text, &extracted_meta); // use the extracted text that is saved in solr and the db instead of the raw, unprocessed website body
            }
        }

        self.update_website_info(w).unwrap();

        let updated_meta = self.modify_meta(extracted_meta, website_id);
        let extracted_ext_links = Crawler/*::<'a>*/::extract_external_links(&body, website_id, &url);

        let updated_ext_links = self.modify_ext_links(extracted_ext_links, website_id);

        let mut website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        let mut website_solr = website_solr_vec.get(0).unwrap();

        self.update_meta(&updated_meta, website_solr).unwrap();

        website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        website_solr = website_solr_vec.get(0).unwrap();
        self.update_external_links(updated_ext_links, website_solr).unwrap();

        Ok(())
    }

    // modify then return the given `meta` vector with updated metadata
    fn modify_meta(&self, extracted_meta: Vec<Metadata>, website_id: Option<u32>) -> Vec<Metadata> {
        extracted_meta.iter().map(|m| {
            println!("Updating meta with id: {:?}", m.id);
            Metadata {id: None, website_id: website_id, metadata_text: m.metadata_text.clone() }
        }).collect()
    }

    fn modify_ext_links(&self, extracted_ext_links: Vec<(ExternalLink, WebsiteRefExtLink)>, website_id: Option<u32>) -> Vec<(ExternalLink, WebsiteRefExtLink)> {
        extracted_ext_links.iter().map( |e_l| {
            // can get the id of this external_link from website_ref_ext_links
            // that was fetched from the db
            // no need to also fetch the external links from the db for the given website
            (
                ExternalLink { id: e_l.0.id, url: e_l.0.url.clone() },
                WebsiteRefExtLink { id: None, website_id: website_id, ext_link_id: e_l.1.ext_link_id }
            )
        }).collect()
    }

    // public wrapper function that is used for testing the method calls in the internal update_website() function
    // it is temporary for now, as it is not really needed
    pub fn test_website_update(&self, website: &WebsiteSolr) -> Result<(), Box<dyn Error>> {
        let website_id = website.id;
        let mut website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        let mut website_solr = website_solr_vec.get(0).unwrap();

        match &website.metadata {
            None => self.update_meta(&self.modify_meta(Vec::new(), website.id), website_solr).unwrap(),
            Some(meta) => {
                self.update_meta(&self.modify_meta(meta.into_iter().map(|m| Metadata { id: None, metadata_text: m.to_string(), website_id: website_id }).collect(), website_id), website_solr).unwrap()
            }
        };

        website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
        website_solr = website_solr_vec.get(0).unwrap();

        match &website.external_links {
            None => self.update_external_links(self.modify_ext_links(Vec::new(), website_id), website_solr).unwrap(),
            Some(ext_links) => self.update_external_links(self.modify_ext_links(ext_links.into_iter().map(|e_l| {
                (
                    ExternalLink {id: None, url: e_l.to_string()},
                    WebsiteRefExtLink {id: None, website_id: website_id, ext_link_id: None}
                 )
            }).collect(), website_id), website_solr).unwrap()
        };

        Ok(())
    }

    fn update_website_info(&self, website_to_update: Website) -> Result<Website, throw::Error<&'static str>>  {
        // TODO update the db::Database::update method to work for metadata and external_links - to work like insert()
        // Then update this function
        if let DB::Website(website) = crate::db::Database::update(&DB::Website(website_to_update)).unwrap() {
            let w_solr = WebsiteSolr {id: website.id, title: website.title.clone(), text: website.text.clone(), url: website.url.clone(), base_url: website.base_url.clone(), rank: website.rank, type_of_website: website.type_of_website.clone(), metadata: None, external_links: None };
            crate::solr::update(&w_solr).unwrap();
            println!("Updated website id: {:?}", website.id);
            Ok(website.clone())
        }
        else {
            throw_new!("Could not update website in the database");
        }
    }

    fn update_meta(&self, metadata_vec: &Vec<Metadata>, website_to_update: &WebsiteSolr) -> Result<Vec<Metadata>, throw::Error<&'static str>> {
        // first delete the metadata associated with the given website, so that after updating
        // them, older metadata enties will not be kept in the database
        crate::db::Database::delete_m(&vec![ website_to_update.id.unwrap() ]).unwrap();
        self.save_metadata(metadata_vec, website_to_update)
    }

    // TODO probably prefix the update (and possibly the save functions) in this file with something like
    // "crawler_" to differentiate them from the solr.rs functions with the same names
    fn update_external_links(&self, external_links: Vec< (ExternalLink, WebsiteRefExtLink) >, website_to_update: &WebsiteSolr) -> Result<Vec< (ExternalLink, WebsiteRefExtLink) >, throw::Error<&'static str>> {
        // delete and save external_links (like update_meta())
        crate::db::Database::delete_el(&vec![ website_to_update.id.unwrap() ]).unwrap();
        self.save_external_links(external_links, website_to_update)
    }

    // TODO make async

    // For now website genre classification is not really needed.
    // I found a lot of resources (mainly research papers) for web genre classification, but most use closed-source datasets for training.
    // The only dataset I could find was https://webis.de/data/genre-ki-04.html but it is from 2004, so it is probably quite outdated.
    // TODO javascript analysis -> execute javascript somehow? and check for popups, keywords that help determine website type, etc.
    // TODO different languages?

    // TODO train a model to guess the type of website (feed in the html document and classify its type)
    //          - http://www.cse.lehigh.edu/~brian/pubs/2007/classification-survey/LU-CSE-07-010.pdf
    //              -> this looks like a good source to use on web page classification
    //              -> it also contains some optimization options that can help speed up the web page analysis

    // TODO add a list of the 50 most popular domains/urls and their types:
    //      example: google: "product" or "search" or "search engine" or "service"
    //      gmail: "product" or "service"
    //      facebook/twitter/...: "social media"
    //      github: "product" or "service"
    //      stackoverflow: "help" or "forum"
    //      scientificamerican: "article" (if it is a url of some article) or "news" or "articles" (if it is just the base url or contacts page, etc.)
    // and statically assign the type based on that
    // if the website is not from the most popular ones: classify it using the classifier or again statically
    fn website_genre(url: &str) -> Result<String, Box<dyn Error>> {
        let classify_request = Request::new("classify").arg(url);
        let classify_result = classify_request.call_url("http://127.0.0.1:9999/classifier");
        // println!("Result of classification: {:?}", classify_result.unwrap());
        match classify_result?.as_array() {
            Some(res) => {
                match res.get(0) {
                   Some(res) => {
                       match res.as_str() {
                           Some(res) => return Ok(res.to_string()),
                           None => bail!("Classifier did not return an array of strings."),
                       }
                   },
                   None => bail!("Classifier returned an empty array."),
                }
            },
            None => bail!("Classifier does not respond."),
        }
    }

    fn website_genre_offline_classification(body: &str, meta: &/*'a */Vec<Metadata>) -> String {
        let body_lc = body.to_lowercase();
        let mut meta_lc;

        for m in meta {
            meta_lc = m.metadata_text.to_lowercase();
            if meta_lc.contains("og:article") {
                return "article".to_string();
            }
            else if meta_lc.contains("og:book") {
                return "product".to_string();
            }
            else if meta_lc.contains("og:website") {
                return "website".to_string();
            }
            else if meta_lc.contains("og:profile") {
                return "social".to_string(); // ?
            }
        }
        // TODO also check meta tags for website type
        /* TODO (if og:type meta tag is present, use its value as a website genre)
            Most web pages that have og:type set are articles, but keep in mind it is not always the case. og:type can also be "website"
            Also, add a list of well know domains that don't need to be classified, like facebook, google, gmail, twitter, etc.
        */

        // Python::with_gil(|py| {
            // let classify = PyModule::from_code(py, "", "classifier.classify.py", "classify").unwrap();
            // let classification: Result<&pyo3::PyAny, PyErr> = classify.call0("asdf");
            // classification.map_err(|e| {
            //     e.print(py);
            // });
            // assert_eq!(classification, "downloads");
            // println!("classification! : {:?}", classification);
            // Ok(())
        // });

        if ((body_lc.contains("install") || body_lc.contains("get started")) && body_lc.contains("version")) || (body_lc.contains("maintained") && body_lc.contains("develop")) {
            // product websites's rank should be mainly determined by users's reviews, users's interactions with the website and how many other websites link to this domain
            return "product".to_string();
        }
        else if (body_lc.contains("author") && body_lc.contains("article")) || body_lc.contains("written by") || body_lc.contains("further reading") {
            // rank should additionally be determined by the quality of the article
            // (why was the article written -> are there too many ads and a short article
            //                              -> do reviewers downvote it a lot
            //                              -> is there a "subscribe to our newsletter"
            //                              -> popups, etc.)
            return "article".to_string();
        }
        // TODO else if...
        return "default".to_string();
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------------
// TESTS

// TODO add asserts
pub fn test_crawler(url: &str) -> Result<(), Box<dyn Error>> {
    let body = Crawler::fetch_url(url).unwrap();
 
    // tests the functions implemented above
    let w = Crawler::extract_website_info(&body, &url); // TODO could call this in the save_website_info function
    let crawler = Crawler {};
    let mut website = crawler.save_website_info(w).unwrap();
    // should get the id from the save_website_info() function
    let website_id = website.id;
    let meta = Crawler::extract_metadata_info(&body, website_id);
    let ext_links = Crawler::extract_external_links(&body, website_id, &url);
    let mut website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
    let mut website_solr = website_solr_vec.get(0).unwrap();

    let mut metadata_to_update = crawler.save_metadata(&meta, website_solr).unwrap();
    // need to fetch the updated website from solr before updating the external_links,
    // otherwise it would set the metadata that was just updated to null
    website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
    website_solr = website_solr_vec.get(0).unwrap();
    let mut external_links_to_update = crawler.save_external_links(ext_links, website_solr).unwrap();

    println!("url is {:?}", &url);
    // println!("Website classification type: {:?}", website_genre(&body, &meta_copy, &url));

    match Crawler::website_genre(&url) {
        Ok(genre) => website.type_of_website = genre,
        Err(err) => {
            println!("Encountered an error while trying to classify the website: {:?}", err);
            println!("Attempting offline classification.");
            website.type_of_website = Crawler::website_genre_offline_classification(&website_solr.text, &meta); // use the extracted text that is saved in solr and the db instead of the raw, unprocessed website body
        }
    }

    website.title = "TEST 1234".to_string();
    website.rank += 1_f64;
    website.base_url = "new_base.com".to_string();

    // for now updating the website info will remove the metadata and external links stored in solr
    // maybe don't overwrite them to null?
    // (or fetch the old metadata and external_links and include them when updating the website in
    // solr
    crawler.update_website_info(website).unwrap();

    // again need to fetch the updated website from solr before updating the external_links
    // otherwise it would set the metadata to the old metadata (it is updated above)
    website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
    website_solr = website_solr_vec.get(0).unwrap();


    metadata_to_update[0].metadata_text = "CHANGED META TEST".to_string();
    crawler.update_meta(&metadata_to_update, website_solr).unwrap();

    // again need to fetch the updated website from solr before updating the external_links
    // otherwise it would set the metadata to the old metadata (it is updated above)
    website_solr_vec = req(format!("id:\"{:?}\"", website_id.unwrap())).unwrap();
    website_solr = website_solr_vec.get(0).unwrap();

    external_links_to_update.get_mut(0).map(|link_to_update| link_to_update.0.url = "CHANGED URL".to_string());
    crawler.update_external_links(external_links_to_update, website_solr).unwrap();

    assert_ne!(Crawler::extract_base_url(url).unwrap(), url, "Extracting the base of the url returns url.");

    Ok(())
}
