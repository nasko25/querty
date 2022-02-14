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

                // delete the url after crawling it
                match db::Database::delete_crawled_url(next_url.url) {
                    Ok(_) => (),
                    Err(err) => return Err(err),
                }
                async_std::task::sleep(std::time::Duration::from_secs(20)).await;
                println!("Crawl running...");
            }
            Ok(())
        })
    };
}
