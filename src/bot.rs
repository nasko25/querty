use reqwest;
use scraper::{Html, Selector};

use diesel::MysqlConnection;

use crate::solr::WebsiteSolr;
use crate::db::Website;

// TODO conn should probably not be passed here
#[tokio::main]
pub async fn analyse_website(url: &str, websites_saved: &Vec<WebsiteSolr>, conn: &MysqlConnection) -> Result<(), reqwest::Error>{
    // await is not necessary
    let res = reqwest::get(url).await?;
    assert!(res.status().is_success());

    let body = res.text().await?;

    website_type(&body);

    if  websites_saved.is_empty() {
        save_website_info(&body, &url, &conn);
    }
    Ok(())
}

fn save_website_info(body: &str, url: &str, conn: &MysqlConnection) {
    // TODO
    // first save the website info(meta tags, title, text, etc.) in the database, and if it is successful (check!) then add it to solr
    // (because the database should (eventually) have a unique constraint on url)
    // if the website cannot be inserted in the database, throw an error

    let fragment = Html::parse_document(body);
    let mut selector = Selector::parse("title").unwrap();

    // if there are multiple titles, only the first is used
    let title_raw = fragment.select(&selector).next().unwrap().inner_html();
    let title = title_raw.trim();
    println!("Website title: {:?}", title);

    // select all text
    selector = Selector::parse("body").unwrap();
    let text = fragment.select(&selector).next().unwrap().text().collect::<Vec<_>>();
    println!("\nWebsite body text: {:?}", text);

    // trim the trailing and leading white spaces
    let mut trimmed_text = Vec::new();
    let mut trimmed_element;
    for element in text {
        trimmed_element = element.trim();
        if trimmed_element != "" {
            trimmed_text.push(trimmed_element);
        }
    }

    println!("\nWebsite body text trimmed: {:?}", trimmed_text.join(", "));

    let w = crate::db::DB::Website (Website { id: None, title: title.to_string(), text: trimmed_text.join(", "), url: url.to_string(), rank: 1.0, type_of_website: "default".to_string()} );
    if let crate::db::DB::Website(mut website) = crate::db::Database::insert(&w, conn).unwrap() {
        // let w_solr = WebsiteSolr {};
        println!("{:?}", website.id);
    }
}

// TODO javascript analysis -> execute javascript somehow? and check for popups, keywords that help determine website type, etc.
// TODO different languages?

// TODO train a model to guess the type of website (feed in the html document and classify its type)
//          - http://www.cse.lehigh.edu/~brian/pubs/2007/classification-survey/LU-CSE-07-010.pdf
//              -> this looks like a good source to use on web page classification
//              -> it also contains some optimization options that can help speed up the web page analysis
fn website_type(body: &str) -> &str {

    let fragment = Html::parse_document(body);
    let selector = Selector::parse("meta").unwrap();
    // println!("Selected meta tags: {:?}", fragment.select(&selector));
    // meta tags can provide info for the type of website
    // TODO content of meta tag can have capital letters -> case insensitive search for "article"
    for element in fragment.select(&selector) {
        println!("element.value(): {:?}, element charset: {:?}, element name: {:?}, element content: {:?}, element.value.name: {:?}", 
            element.value(), element.value().attr("charset"), element.value().attr("name"), element.value().attr("content"), element.value().name());
    }

    let body_lc = body.to_lowercase();

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