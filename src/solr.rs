use reqwest;
use crate::settings;
use crate::db::from_str;
use crate::db::Metadata;
use crate::db::ExternalLink;

use std::process::Command;

extern crate shellexpand;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebsiteSolr {
    #[serde(deserialize_with = "from_str")]
    pub id: Option<u32>,
    pub title: String,
    pub text: String,
    pub url: String,
    pub base_url: String,
    pub rank: f64,
    pub type_of_website: String,
    // not in the db, but present in solr:
    pub metadata: Option<Vec<String>>,
    pub external_links: Option<Vec<String>>
}

#[tokio::main]
pub async fn req(settings: &settings::Settings, query: String) -> Result<Vec<WebsiteSolr>, reqwest::Error> {
    let solr = &settings.solr;
    println!("Solr config: {:?}", solr);

    let method = "select";
    // TODO more options
    let url =  format!("http://{}:{}/solr/{}/{}?q={}", &solr.server, &solr.port, &solr.collection, &method, &query);

    println!("{}", reqwest::get(&url).await?.text().await?);
    let res: Response = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;

    // println!("Result: {:?}", res.response.docs.get(1).unwrap().metadata);

    Ok(res.response.docs)
}

#[tokio::main]
pub async fn insert(settings: &settings::Settings, website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let solr = &settings.solr;

    let method = "update";

    let url = format!("http://{}:{}/solr/{}/{}/json/docs?commit=true",  &solr.server, &solr.port, &solr.collection, &method);
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
"base_url": "test.com",
"rank": 0.999,
"type_of_website": "test", "metadata":[], "external_links":[]}
'
    */
}

#[tokio::main]
pub async fn update(settings: &settings::Settings, website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let solr = &settings.solr;
    let method = "update";
    let url = format!("http://{}:{}/solr/{}/{}/json/docs?commit=true",  &solr.server, &solr.port, &solr.collection, &method);
    reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .json(website)
        .send()
        .await?;

    Ok(())
}

pub fn update_metadata(settings: &settings::Settings, metadata: &Vec<Metadata>, website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let mut new_metadata = Vec::new();
    for m in metadata {
        new_metadata.push(m.metadata_text.clone());
    }

    let mut website_mut = website.clone();
    website_mut.metadata = Some(new_metadata);

    update(settings, &website_mut)
}

// TODO code duplication?
pub fn update_ext_links(settings: &settings::Settings, external_links: &Vec<ExternalLink>, website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let mut new_ext_links = Vec::new();
    for el in external_links {
        new_ext_links.push(el.url.clone());
    }

    let mut website_mut = website.clone();
    website_mut.external_links = Some(new_ext_links);

    update(settings, &website_mut)
}

#[tokio::main]
pub async fn dataimport(settings: &settings::Settings) -> Result<(), reqwest::Error> {
    let solr = &settings.solr;
    let method = "dataimport";
    let url = format!("http://{}:{}/solr/{}/{}?command=full-import&commit=true&clean=true", &solr.server, &solr.port, &solr.collection, &method);
    reqwest::Client::new()
        .post(&url)
        .send()
        .await?;

    Ok(())
}

// create and delete collections
// useful in development and when reindexing
//      NOTE: in order to reindex, you need to delete the collection, and create it again
//      of course this will delete everything that solr stored, so it needs to be reinserted again
//      from the relational db (using solr's data import functionality)
#[tokio::main]
pub async fn create_collection(settings: &settings::Settings) -> Result<std::process::Output, reqwest::Error> {
    // https://doc.rust-lang.org/std/process/struct.Command.html
    let solr = &settings.solr;

    let out = if cfg!(target_os = "windows") {
        // TODO not tested for windows
        // shellexpand?
        Command::new("\"%HOMEDRIVE%%HOMEPATH%\"\\solr-8.6.2\\bin\\solr")
            .args(&["create", "\\c", &solr.collection, "\\s", "2", "\\rf", "2", "\\d", "\"%HOMEDRIVE%%HOMEPATH%\"\\querty\\config\\solr", "\\p", "8983"])
            .output()
            .expect("Failed to create a new solr collection")
    } else {
        // TODO maybe don't hardcode the 2 paths?
        Command::new(shellexpand::tilde("~/solr-8.6.2/bin/solr").as_ref())
            .arg("create")
            .arg("-c")
            .arg(&solr.collection)
            .arg("-s")
            .arg("2")
            .arg("-rf")
            .arg("2")
            .arg("-d")
            .arg(shellexpand::tilde("~/querty/config/solr").as_ref())
            .arg("-p")
            .arg("8983")
            .output()
            .expect("Failed to create a new solr collection")
    };

    println!("Output after creating a collection: {:}", std::str::from_utf8(&out.stdout).unwrap());
    return Ok(out);
}

#[tokio::main]
pub async fn delete_collection(settings: &settings::Settings) -> Result<std::process::Output, reqwest::Error> {
    let solr = &settings.solr;
    let out = if cfg!(target_os = "windows") {
        // TODO not tested for windows
        // shellexpand?
        Command::new("\"%HOMEDRIVE%%HOMEPATH%\"\\solr-8.6.2\\bin\\solr")
            .args(&["delete", "\\c", &solr.collection, "\\p", "8983"])
            .output()
            .expect("Failed to delete the solr collection")
    } else {
        Command::new(shellexpand::tilde("~/solr-8.6.2/bin/solr").as_ref())
            .arg("delete")
            .arg("-c")
            .arg(&solr.collection)
            .arg("-p")
            .arg("8983")
            .output()
            .expect("Failed to delete the solr collection")
    };

    println!("Output after deleting a collection: {:}", std::str::from_utf8(&out.stdout).unwrap());
    return Ok(out);
}
