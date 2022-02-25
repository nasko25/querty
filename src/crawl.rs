// TODO make function again instead of macro
// use the crawler and the next_urls_to_crawl db table to crawl continuously
#[macro_export]
macro_rules! crawl {
    () => {
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
    };
}

#[tokio::main]
pub async fn generate_urls_from_sitemap(base_urls: Vec<String>) -> Result<Vec<String>, reqwest::Error> {
    // TODO fetch sitemap.xml for each given base url (if they are available)
    //  and generate valid urls from the parsed sitemap.xml files

    let client = reqwest::Client::new();
    // TODO parse sitemap.xml and return valid urls to be parsed
    for base_url in base_urls {
        // first try https
        let response = client.get(format!("https://{}/sitemap.xml", base_url)).send().await?;
        if response.status().is_success() {
            println!("{}", response.text().await?);
            std::process::exit(-1);
        } else {
            println!("Sitemap is not available: {}", response.status());
        }
    }

    Ok(Vec::new())
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
        println!("Website from external_links of the given url: {}", reqwest::get(link).await?.text().await?);
    }
    Ok(())
}
