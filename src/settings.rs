use config::{ConfigError};

// used to load global configuration variables
use once_cell::sync::Lazy;
use std::sync::Mutex;

// TODO getters instead of pub?
#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub server: String,
    pub port: u16,
    pub db_name: String,
    pub user: String,
    pub pass: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Solr {
    pub server: String,
    pub port: u16,
    pub collection: String,
    pub path_to_solr: String,
    pub path_to_solr_config: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    debug: bool,
    pub database: Database,
    pub solr: Solr
}

impl Settings {
    pub fn new(debug: bool) -> Result<Self, ConfigError> {
        let mut config = config::Config::new();
        let file = config::File::with_name("config/config.toml");

        config.set("debug", debug).unwrap();
        config.merge(file).unwrap();

        config.try_into()
    }
    pub fn get_serv(&self) -> &String {
        &self.database.server
    }
}

pub static SETTINGS: Lazy<Settings> = Lazy::new (|| {
    Settings::new(false).unwrap()
});
