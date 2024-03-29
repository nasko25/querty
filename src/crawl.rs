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

            // first crawl the given url
            let mut websites_saved = solr::req(format!("url:\"{}\"", next_url.url)).unwrap();
            let crawler = Crawler {};
            crawler.analyse_website(&next_url.url, &websites_saved).unwrap();

            websites_saved = solr::req(format!("url:\"{}\"", next_url.url)).unwrap();

            // println!("{:?}", websites_saved[0].external_links);
            // check sitemap.xml of each domain in external_links and analyse them
            match generate_urls_from_sitemap(websites_saved[0].external_links.clone().unwrap()) {
                Ok(generated_urls) => {
                    // crawl the urls fetched from the sitemaps
                    let crawler = Crawler {};
                    crawler.analyse_websites(generated_urls);
                    // TODO this will probably cause popular websites to be crawled too frequently
                },
                Err(err) => { colour::red!("Could not generate urls from sitemap.xml: {}\n", err); }

            }

            let parsed_url = match Url::parse(&next_url.url) {
                Ok(result)  => result,
                Err(err) => { colour::red! ("Cannot parse url {}: {}", &next_url.url, err); db::Database::delete_crawled_url(next_url.url)?; continue; }
            };

            let parsed_url_host = match parsed_url.host() {
                Some(result) => result,
                None => { colour::red!("{} has no host,", parsed_url); db::Database::delete_crawled_url(next_url.url)?; continue; }
            };
            // generate urls from the sitemap of the given url and also crawl them
            match generate_urls_from_sitemap(vec![format!("{}/{}", parsed_url_host.to_string(), parsed_url.path())]) {
                Ok(generated_urls) => {
                    crawler.analyse_websites(generated_urls);
                },
                Err(err) => { colour::red!("Could not generate urls from sitemap.xml: {}\n", err); }
            }

            // delete the url after crawling it
            db::Database::delete_crawled_url(next_url.url)?;
            async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            println!("Crawl running...");
        }
        Ok(())
    })
}

#[tokio::main]
pub async fn generate_urls_from_sitemap(base_urls: Vec<String>) -> Result<Vec<String>, reqwest::Error> {
    let client = reqwest::Client::new();
    let sitemaps = &mut Vec::<SiteMapEntry>::new();
    let urls = &mut HashSet::<String>::new();

    // keep track of the already fetched sitemaps, so that you are not stuck in a loop
    let mut fetched_sitemaps = HashSet::<String>::new();

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

    // recutsively crawl the sitemap.xml files that were added to the sitemaps vector
    //  they were added from recursive sitemap.xml mappings, so the crawler should
    //  first ensure that this sitemap url hasn't already been fetched
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
                    match url_entry.loc.get_url() {
                        Some(url_entry_unwraped)    => urls.insert(url_entry_unwraped.to_string()),
                        None                        => { println!("Url entry from {} is not a valid url: {:?}", url, url_entry.loc); false }
                    };
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
        // insert url to be crawled next only if it is not already in solr (as url)
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
