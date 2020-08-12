// TODO tables https://docs.rs/diesel/1.4.5/diesel/macro.table.html
// https://github.com/diesel-rs/diesel/blob/master/examples/mysql/getting_started_step_1/src/schema.rs
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;

use crate::schema::website;

#[derive(Queryable, Insertable)]
#[table_name = "website"]
pub struct Website {
    pub id: i32,
    pub title: String,
    pub metadata: String,
    pub url: String,
    pub rank: i32,
    pub type_of_website: String
}

#[derive(Queryable)]
pub struct Keywords {
    pub website_id: i32,
    pub keyword: String,
    pub rank_per_kw: String
}

pub struct Database {
    // server: String,
    // port: u16
}

impl Database {
    pub fn establish_connection(db_url: &str) -> MysqlConnection {
        MysqlConnection::establish(&db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
    }
    // pub fn conn() {
    //     // TODO
    // }
}