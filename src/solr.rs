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
pub struct WebsiteSolr {
    #[serde(deserialize_with = "from_str")]
    pub id: Option<u32>,
    pub title: String,
    pub text: String,
    pub url: String,
    pub rank: f64,
    pub type_of_website: String,
    // not in the db, but present in solr:
    pub metadata: Option<Vec<String>>,
    pub external_links: Option<Vec<String>>
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

#[tokio::main]
pub async fn insert(settings: &settings::Settings, website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let solr = &settings.solr;

    let method = "update";

    let url = format!("http://{}:{}/solr/{}/{}/json/docs?commit=true",  &solr.server, &solr.port, &solr.collection, &method);

    // TODO pass object as parameter to the method (like the db::insert)
        reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&website)
        .send()
        .await?;

    Ok(())
    /*
    curl -X POST -H 'Content-Type: application/json' 'http://localhost:8983/solr/querty/update/json/docs?commit=true' --data-binary '
{
  "id": "2222",
  "title": "heyo",

"text": "ok",
"url": "test.com",
"rank": 0.999,
"type_of_website": "test", "metadata":[], "external_links":[]}
'
    */
}