use reqwest;
use crate::settings;
use crate::db::from_str;

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    #[serde(rename = "responseHeader")]
    response_header: Header,
    response: ResponseBody
}

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    #[serde(rename = "zkConnected")]
    zk_connected: bool,
    status: i8,
    #[serde(rename = "QTime")]
    q_time: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseBody {
    #[serde(rename = "numFound")]
    num_found: i64,
    start: i32,
    #[serde(rename = "maxScore")]
    max_score: f32,
    #[serde(rename = "numFoundExact")]
    num_found_exact: bool,
    docs: Vec<WebsiteSolr>
}

#[derive(Debug, Serialize, Deserialize)]
struct WebsiteSolr {
    #[serde(deserialize_with = "from_str")]
    id: Option<u32>,
    title: String,
    text: String,
    url: String,
    rank: f64,
    type_of_website: String,
    // not in the db, but present in solr:
    metadata: Option<Vec<String>>,
    external_links: Option<Vec<String>>
}

#[tokio::main]
pub async fn req(settings: &settings::Settings) -> Result<(), reqwest::Error> {
    let solr = &settings.solr;
    println!("Solr config: {:?}", solr);

    let method = "select";
    let query = "*:*";
    // TODO more options
    let url =  format!("http://{}:{}/solr/{}/{}?q={}", &solr.server, &solr.port, &solr.collection, &method, &query);

    println!("{}", reqwest::get(&url).await?.text().await?);
    let res: Response = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    println!("Result: {:?}", res.response.docs.get(1).unwrap().metadata);

    Ok(())
}