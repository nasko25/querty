use reqwest;
use crate::db::from_str;
use crate::db::Metadata;
use crate::db::ExternalLink;
use crate::web_api;

use crate::settings::SETTINGS;

use std::process::Command;
use std::env;

use std::collections::{ HashMap, HashSet };
use indexmap::IndexSet;
use std::hash::{ Hash, Hasher };
use std::fmt;
use std::error;

use url::{ Url };
use urlencoding::encode;

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
    //#[serde(rename = "maxScore")]
    //max_score: f32,
    #[serde(rename = "numFoundExact")]
    num_found_exact: bool,
    docs: Vec<WebsiteSolr>
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseSuggester {
    #[serde(rename = "responseHeader")]
    response_header: Header,
    suggest: HashMap<String, SimilarWords>
}

#[derive(Debug, Serialize, Deserialize)]
struct SimilarWords {
    #[serde(flatten)]
    suggestion: HashMap<String, Suggestions>
}

#[derive(Debug, Serialize, Deserialize)]
struct Suggestions {
    #[serde(rename = "numFound")]
    num_found: i32,
    suggestions: IndexSet<Term>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Term {
    pub term: String,
    weight: i64,
    payload: String
}

// term will need a Hash and Eq traits to create a HashSet with Terms
impl Hash for Term {
    fn hash<H: Hasher> (&self, state: &mut H) {
        self.term.hash(state);
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.term.eq(&other.term)
    }
}

impl Eq for Term {}

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
pub async fn req(query: String) -> Result<Vec<WebsiteSolr>, reqwest::Error> {
    let solr = &SETTINGS.solr;
    println!("Solr config: {:?}", solr);

    let method = "select";
    // TODO more options
    // The query needs to already be url encoded. This is to allow url parameters. TODO In the future it
    // would be better to pass the potentially multiple query strings to req(), where they would be
    // encoded and passed as url parameters
    //  For now, the query string needs to be url encoded externally, so that the '&' characters
    //  would not be encoded if it is used to separate url parameters.
    let url =  &format!("http://{}:{}/solr/{}/{}?q={}", &solr.server, &solr.port, &solr.collection, &method, &encode(&query));

    let res: Response = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    // println!("Result: {:?}", res.response.docs.get(1).unwrap().metadata);

    Ok(res.response.docs)
}

#[tokio::main]
pub async fn insert(website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let solr = &SETTINGS.solr;

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
pub async fn update(website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let solr = &SETTINGS.solr;
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

pub fn update_metadata(metadata: &Vec<Metadata>, website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let mut new_metadata = Vec::new();
    for m in metadata {
        new_metadata.push(m.metadata_text.clone());
    }

    let mut website_mut = website.clone();
    website_mut.metadata = Some(new_metadata);

    update(&website_mut)
}

// TODO code duplication?
pub fn update_ext_links(external_links: &Vec<ExternalLink>, website: &WebsiteSolr) -> Result<(), reqwest::Error> {
    let mut new_ext_links = Vec::new();
    for el in external_links {
        new_ext_links.push(el.url.clone());
    }

    let mut website_mut = website.clone();
    website_mut.external_links = Some(new_ext_links);

    update(&website_mut)
}

#[tokio::main]
pub async fn dataimport() -> Result<(), reqwest::Error> {
    let solr = &SETTINGS.solr;
    let method = "dataimport";
    let url = format!("http://{}:{}/solr/{}/{}?command=full-import&commit=true&clean=true", &solr.server, &solr.port, &solr.collection, &method);
    reqwest::Client::new()
        .post(&url)
        .send()
        .await?;

    Ok(())
}

// Create an error struct that can be thrown if the query argument passed to the suggester has a
// wrong format
#[derive(Debug, Clone)]
struct SuggesterUnexpectedParam(String);

impl fmt::Display for SuggesterUnexpectedParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid parameter passed to suggest(): {}", self.0)
    }
}

impl error::Error for SuggesterUnexpectedParam {}

// Create an error struct that can be thrown if the JSON deserialized object is missing a field
#[derive(Debug, Clone)]
struct SuggesterJSONError(String);

impl fmt::Display for SuggesterJSONError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JSON object returned from solr has unexpected format; {}", self.0)
    }
}


#[tokio::main]
pub async fn suggest(query: String) -> Result<IndexSet<Term>, Box<dyn error::Error> /*reqwest::Error*/> {
    if query.chars().count() < 2 || query.chars().count() > 255 {
        //throw_new!("query should be between 2 and 255 characters long");
        //Err("asd")
        //println!("{}", SuggesterUnexpectedParam("query string should be between 2 and 255 characters long".to_string()));
        return Err(SuggesterUnexpectedParam("query string should be between 2 and 255 characters long".to_string()).into());
    }

    let solr = &SETTINGS.solr;
    let method = "suggest";

    let sanitized_query = web_api::sanitize_query(&query);
    let url = format!("http://{}:{}/solr/{}/{}?suggest=true&suggest.build=true&suggest.dictionary=mySuggester&wt=json&suggest.q=text%3A{}&sort={}", &solr.server, &solr.port, &solr.collection, &method, encode(&query), web_api::build_sort_query(sanitized_query, &web_api::SortQueryType::SUGGEST));

    let response: ResponseSuggester = reqwest::Client::new()
        .get(&url)
        .send()
        .await.expect("Solr is not responding to the /suggest request")
        .json()
        .await.expect("Solr's response is not valid json");

    //println!("response: {:?}", response.suggest.values().next().unwrap().suggestion.values().next().unwrap().suggestions);

    /*
     *
     * This is how a response JSON objevt looks like:
        {
            "responseHeader":{
                "zkConnected":true,
                "status":0,
                "QTime":13
            },
            "command":"build",
            "suggest":{
                "mySuggester":{
                    "an":{
                        "numFound":4,
                        "suggestions":[
                            {
                                "term":"and",
                                "weight":398245770157805504,
                                "payload":""
                            },
                            {
                                "term":"and",
                                "weight":72057594037927936,
                                "payload":""
                            },
                            {
                                "term":"an",
                                "weight":63719323225248880,
                                "payload":""
                            },
                            {
                                "term":"antonio",
                                "weight":15929830806312220,
                                "payload":""
                            }
                        ]
                    }
                }
            }
        }

     * From this object the vector of terms should be extracted and returned.
     *
     * */

    match response
        .suggest            // get the first value from the suggest HashMap (which is the value corresponding to the "mySuggester" key above)
        .values()
        .next() {
            Some(suggestion) => {
                match suggestion
                    .suggestion     // get the value from the SimilarWords suggestion HashMap (which is the value corresponding to the query string "an" key above)
                    .values()
                    .next() {
                        Some(suggestions) => Ok(suggestions.suggestions.clone()),
                        None => return Err(SuggesterUnexpectedParam("Cannot extract values from Suggestions".to_string()).into())
                    }
            },
            None => return Err(SuggesterUnexpectedParam("Cannot extract values from ResponseSuggester".to_string()).into())
        }
}

// create and delete collections
// useful in development and when reindexing
//      NOTE: in order to reindex, you need to delete the collection, and create it again
//      of course this will delete everything that solr stored, so it needs to be reinserted again
//      from the relational db (using solr's data import functionality)
#[tokio::main]
pub async fn create_collection() -> Result<std::process::Output, reqwest::Error> {
    // https://doc.rust-lang.org/std/process/struct.Command.html
    let solr = &SETTINGS.solr;

    let out = if cfg!(target_os = "windows") {
        // TODO not tested for windows
        // shellexpand?
        Command::new("\"%HOMEDRIVE%%HOMEPATH%\"\\solr-8.6.2\\bin\\solr")
            .args(&["create", "\\c", &solr.collection, "\\s", "2", "\\rf", "2", "\\d", "\"%HOMEDRIVE%%HOMEPATH%\"\\querty\\config\\solr", "\\p", "8983"])
            .output()
            .expect("Failed to create a new solr collection")
    } else {
        Command::new(shellexpand::tilde(&env::var("SOLR_PATH").unwrap_or_else(|_| {
                    // TODO should these prints be everywhere env::var is used?
                    println!("SOLR_PATH environment variable not set.");
                    println!("Using \"{}\" (from the config.toml) instead.\n", solr.path_to_solr);
                    solr.path_to_solr.clone()
                }
            )).as_ref())
            .arg("create")
            .arg("-c")
            .arg(&solr.collection)
            .arg("-s")
            .arg("2")
            .arg("-rf")
            .arg("2")
            .arg("-d")
            .arg(shellexpand::tilde(&env::var("SOLR_CONFIG_PATH").unwrap_or_else(|_| solr.path_to_solr_config.clone())).as_ref())
            .arg("-p")
            .arg("8983")
            .output()
            .expect("Failed to create a new solr collection")
    };

    println!("Output after creating a collection: {:}", std::str::from_utf8(&out.stdout).unwrap());
    return Ok(out);
}

#[tokio::main]
pub async fn delete_collection() -> Result<std::process::Output, reqwest::Error> {
    let solr = &SETTINGS.solr;
    let out = if cfg!(target_os = "windows") {
        // TODO not tested for windows
        // shellexpand?
        Command::new("\"%HOMEDRIVE%%HOMEPATH%\"\\solr-8.6.2\\bin\\solr")
            .args(&["delete", "\\c", &solr.collection, "\\p", "8983"])
            .output()
            .expect("Failed to delete the solr collection")
    } else {
        Command::new(shellexpand::tilde(&env::var("SOLR_PATH").unwrap_or_else(|_| solr.path_to_solr.clone())).as_ref())
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
