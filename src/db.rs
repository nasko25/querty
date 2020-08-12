// TODO tables https://docs.rs/diesel/1.4.5/diesel/macro.table.html
// https://github.com/diesel-rs/diesel/blob/master/examples/mysql/getting_started_step_1/src/schema.rs
use diesel::prelude::*;
#[cfg(test)]
use diesel::mysql::MysqlConnection;

mod schema {
    diesel::table! {
        website (id) {
            id -> Integer,
            title -> Text,
            metadata -> Text,
            url -> Varchar,
            rank -> Integer,
            type_of_website -> Varchar,             // TODO table for website types?
        }
    }
    diesel::table! {
        keywords (website_id, keyword) {
            website_id -> Integer,
            keyword -> Varchar,
            rank_per_kw -> Varchar,
        }
    }
}

// use schema::website;

#[derive(Queryable)]
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
    server: String,
    port: u16
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