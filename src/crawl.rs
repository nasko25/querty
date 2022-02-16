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
                // TODO check robots.txt of each domain in external_links and add the allowed urls
                // to be crawled next
                match add_next_crawl_url(websites_saved[0].external_links.clone().unwrap()).await {
                    Ok(_) => (),
                    Err(err) => { colour::red!("Could not add next crawl url to the database: {}\n", err); },
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

// TODO make private
pub async fn add_next_crawl_url(external_links: Vec<String>) -> Result<(), reqwest::Error>{
    // TODO
    // get robots.txt from each external link
    // parse robots.txt and eppend the path to the external_link
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
