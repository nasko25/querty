use crate::db;
use crate::solr;
use crate::crawler::Crawler;
use sitemap::reader::{ SiteMapReader, SiteMapEntity };
use sitemap::structs::{ SiteMapEntry, UrlEntry };
use sitemap::structs::Location::{ ParseErr, Url as SitemapUrl, None as SitemapNone };

use reqwest::Url;
use std::collections::HashSet;

// TODO use the crawler and the next_urls_to_crawl db table to crawl continuously
pub fn crawl() -> async_std::task::JoinHandle<Result<(), diesel::result::Error>> {
    async_std::task::spawn(async {
        loop {
            let next_url = match db::Database::select_next_crawl_url() {
                Ok(url) => url,
                // break if there are no more urls to be crawled
                Err(diesel::result::Error::NotFound) => break,
                Err(err) => return Err(err),
            };
            // TODO ...
            // also add urls that have not yet been crawled linked from that url

            let mut websites_saved = solr::req(format!("url:\"{}\"", next_url.url)).unwrap();
            let crawler = Crawler {};
            crawler.analyse_website(&next_url.url, &websites_saved).unwrap();

            websites_saved = solr::req(format!("url:\"{}\"", next_url.url)).unwrap();
            // println!("{:?}", websites_saved[0].external_links);
            // TODO check sitemap.xml of each domain in external_links and add the allowed urls
            // to be crawled next
            match generate_urls_from_sitemap(websites_saved[0].external_links.clone().unwrap()) {
                Ok(generated_urls) => match add_next_crawl_urls(generated_urls) {
                    Ok(_) => (),
                    Err(err) => { colour::red!("Could not add next crawl url to the database: {}\n", err); },
                },
                Err(err) => { colour::red!("Could not generate urls from sitemap.xml: {}\n", err); }

            }

            // delete the url after crawling it
            db::Database::delete_crawled_url(next_url.url)?;
            async_std::task::sleep(std::time::Duration::from_secs(20)).await;
            println!("Crawl running...");
        }
        Ok(())
    })
}

// recutsively crawl sitemap.xml that were added to the sitemaps db table
//  they were added from recursive sitemap.xml mappings, so the crawler should
//  first check whether this sitemap url was already fetched
pub fn rec_crawl_sitemaps() {
    // TODO
}

#[tokio::main]
pub async fn generate_urls_from_sitemap(base_urls: Vec<String>) -> Result<Vec<String>, reqwest::Error> {
    // TODO fetch sitemap.xml for each given base url (if they are available)
    //  and generate valid urls from the parsed sitemap.xml files

    let client = reqwest::Client::new();
    let sitemaps = &mut Vec::<SiteMapEntry>::new();
    let urls = &mut HashSet::<String>::new();

    // keep track of the already fetched sitemaps, so that you are not stuck in a loop
    let mut fetched_sitemaps = Vec::<String>::new(); // TODO url or even string

    // TODO parse sitemap.xml and return valid urls to be parsed
    for base_url in base_urls {
        // TODO extract that fetching and handling of the sitemaps to a new functon that will be
        // used by https, http handlers, and the while !sitemaps.is_empty() loop
        // first try https
        let mut url_to_fetch = format!("https://{}/sitemap.xml", base_url);
        fetch_and_handle_sitemaps(&url_to_fetch, &client, sitemaps, urls).await?;
        fetched_sitemaps.push(url_to_fetch);

        // then try http either because https was not available or just in case there is a new url
        url_to_fetch = format!("http://{}/sitemap.xml", base_url);
        fetch_and_handle_sitemaps(&url_to_fetch, &client, sitemaps, urls).await?;
        fetched_sitemaps.push(url_to_fetch);
    }

    while !sitemaps.is_empty() {
        match sitemaps.pop().unwrap().loc {
            SitemapNone         => (),
            SitemapUrl(sitemap) => fetched_sitemaps.push(sitemap.to_string()),
            ParseErr(err)       => println!("Error when parsing sitemap: {}", err)
        };
        // TODO
    }

    Ok(urls.clone().into_iter().collect())
}

async fn fetch_and_handle_sitemaps(url: &String, client: &reqwest::Client, sitemaps: &mut Vec::<SiteMapEntry>, urls: &mut HashSet::<String>) -> Result<(), reqwest::Error> {
    let response = client.get(url).send().await?;
    if response.status().is_success() {
        for entity in SiteMapReader::new(response.text().await?.as_bytes()) {
            match entity {
                SiteMapEntity::Url(url_entry) => {
                    // TODO handle None instead of .unwrap()
                    //  also return Url or String?
                    urls.insert(url_entry.loc.get_url().unwrap().to_string());
                    println!("url: {:?}", url_entry);
                    std::process::exit(-1);
                },
                SiteMapEntity::SiteMap(sitemap_entry) => {
                    println!("sitemap: {:?}", sitemap_entry);
                    sitemaps.push(sitemap_entry);
                },
                SiteMapEntity::Err(error) => {
                    // errors.push(error);
                    println!("err: {}", error);
                    std::process::exit(-1);
                },
            }
        }
    } else {
        println!("Sitemap is not available from {}: {}", url, response.status());
    }
    Ok(())
}

// TODO make private
#[tokio::main]
pub async fn add_next_crawl_urls(external_links: Vec<String>) -> Result<(), reqwest::Error>{
    // TODO
    // get sitemap.xml from each external link
    // parse sitemap.xml and eppend the path to the external_link
    // add the external_link and all its variants to the next_urls_to_crawl db table
    //  if they are not already in solr (as urls)
    for link in external_links.iter() {
        println!("Link: {}", link);
        // TODO link is only the hostname
        //  build a url; first try https, then http
        // println!("Website from external_links of the given url: {}", reqwest::get(link).await?.text().await?);
    }
    Ok(())
}
