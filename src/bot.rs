use reqwest;

#[tokio::main]
pub async fn analyse_website(url: &str) -> Result<(), reqwest::Error>{
    let body = reqwest::get(url)
    .await?
    .text()
    .await?;

    println!("body = {:?}; type = {:?}", body, website_type(&body));
    Ok(())
}

// TODO javascript analysis -> execute javascript somehow? and check for popups, keywords that help determine website type, etc.
// TODO different languages?
fn website_type(body: &str) -> &str {
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