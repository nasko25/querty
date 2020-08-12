use config::{ConfigError};

#[derive(Debug, Deserialize)]
struct Database {
    server: String,
    port: u16
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    debug: bool,
    database: Database
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