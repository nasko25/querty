use crate::db;
use crate::db::NextUrl;
use tokio::task;

// use the crawler and the next_urls_to_crawl db table to crawl continuously
pub fn crawl() -> task::JoinHandle<Result<(), diesel::result::Error>>{
    task::spawn(async {
        loop {
            let next_url: NextUrl = match db::Database::select_next_crawl_url() {
                Ok(url) => url,
                Err(err) => return Err(err),
            };
            // TODO ...
        }
    })
}
