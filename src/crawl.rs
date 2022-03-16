use crate::db;
use crate::solr;
use crate::crawler::Crawler;
use crate::db::NextUrl;
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
    let client = reqwest::Client::new();
    let sitemaps = &mut Vec::<SiteMapEntry>::new();
    let urls = &mut HashSet::<String>::new();

    // keep track of the already fetched sitemaps, so that you are not stuck in a loop
    // TODO Vec or HashSet
    let mut fetched_sitemaps = HashSet::<String>::new(); // TODO url or even string

    for base_url in base_urls {
        // first try https
        let mut url_to_fetch = format!("https://{}/sitemap.xml", base_url);
        fetch_and_handle_sitemaps(&url_to_fetch, &client, sitemaps, urls).await?;
        fetched_sitemaps.insert(url_to_fetch);

        // then try http either because https was not available or just in case there is a new url
        url_to_fetch = format!("http://{}/sitemap.xml", base_url);
        fetch_and_handle_sitemaps(&url_to_fetch, &client, sitemaps, urls).await?;
        fetched_sitemaps.insert(url_to_fetch);
    }

    while !sitemaps.is_empty() {
        match sitemaps.pop().unwrap().loc {
            SitemapNone         => (),
            SitemapUrl(sitemap) => {
                if !fetched_sitemaps.contains(&sitemap.to_string()) {
                    fetch_and_handle_sitemaps(&sitemap.to_string(), &client, sitemaps, urls).await?;
                    fetched_sitemaps.insert(sitemap.to_string());
                }
            },
            ParseErr(err)       => println!("Error when parsing sitemap: {}", err)
        };
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
                },
                SiteMapEntity::SiteMap(sitemap_entry) => {
                    println!("sitemap: {:?}", sitemap_entry);
                    sitemaps.push(sitemap_entry);
                },
                SiteMapEntity::Err(error) => {
                    // errors.push(error);
                    println!("ERROR when parsing sitemap: {}\nFor url: {}", error, url);
                    //std::process::exit(-1);
                },
            }
        }
    } else {
        println!("Sitemap is not available from {}: {}", url, response.status());
    }
    Ok(())
}

// TODO make private
pub fn add_next_crawl_urls(external_links: Vec<String>) -> Result<(), reqwest::Error>{
    for link in external_links.iter() {
        println!("Link: {}", link);
        // TODO insert url to be crawled next only if it is not already in solr (as url)?
        match solr::req(format!("url:\"{}\"", link)) {
            Ok(websites_solr) if websites_solr.is_empty()       => {
                db::Database::insert(&db::DB::NextUrl( NextUrl { id: None, url: link.to_string() }));
            },
            Ok(_)                                               => println!("url already in solr: {}", link),
            Err(err)                                            => return Err(err)
            // Err(err)                                            => println!("ERROR when fetching urls from solr: {}", err)
        }
    }
    Ok(())
}
