// use the crawler and the next_urls_to_crawl db table to crawl continuously
#[macro_export]
macro_rules! crawl {
    () => {
        async_std::task::spawn(async {
            let mut next_url: NextUrl;
            loop {
                next_url = match db::Database::select_next_crawl_url() {
                    Ok(url) => url,
                    Err(err) => return Err(err),
                };
                // TODO ...
                // also add urls that have not yet been crawled linked from that url

                // delete the url after crawling it
                db::Database::delete_crawled_url(next_url.url);
                async_std::task::sleep(std::time::Duration::from_secs(20)).await;
                println!("Crawl running...");
            }
            Ok(next_url)
        })
    };
}
