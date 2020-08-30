// TODO tables https://docs.rs/diesel/1.4.5/diesel/macro.table.html
// https://github.com/diesel-rs/diesel/blob/master/examples/mysql/getting_started_step_1/src/schema.rs
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::{ insert_into, sql_query };

use crate::schema::website;
use crate::schema::website::dsl::*;
use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::schema::metadata;
use crate::schema::metadata::dsl::*;
use crate::schema::external_links;
use crate::schema::external_links::dsl::*;
use crate::schema::website_ref_ext_links;
use crate::schema::website_ref_ext_links::dsl::*;


// TODO maybe add a field for links to other websies that can be used by something like PageRank?
#[derive(Queryable, Insertable, Debug, Serialize, Deserialize, Identifiable)]
#[table_name = "website"]
pub struct Website {
    #[serde(deserialize_with = "from_str")]
    pub id: Option<u32>,
    pub title: String,
    pub text: String,
    pub url: String,
    pub rank: i32,
    pub type_of_website: String
}

use std::str::FromStr;
use std::fmt::Display;
use serde::de::{self, Deserialize, Deserializer};
// CITATION: https://github.com/serde-rs/json/issues/317#issuecomment-300251188
fn from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    match T::from_str(&s).map_err(de::Error::custom) {
        Ok(r) => return Ok(Some(r)),
        Err(err) => return Err(err)
    }
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "users"]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub rank: f64,
    pub country_iso_a2: String
}

#[derive(Identifiable, Queryable, Associations, Debug, Insertable)]
#[belongs_to(Website)]
#[table_name = "metadata"]
pub struct Metadata {
    pub id: Option<u32>,
    pub metadata_text: String,
    pub website_id: Option<u32>,
}

// TODO insert external_links and website_ref_ext_links
#[derive(Identifiable, Queryable, Associations, Debug)]
#[table_name = "external_links"]
pub struct ExternalLink {
    pub id: Option<u32>,
    pub url: String,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Website)]
#[belongs_to(ExternalLink, foreign_key = "ext_link_id")]
#[table_name = "website_ref_ext_links"]
pub struct WebsiteRefExtLink {
    pub id: Option<u32>,
    pub website_id: Option<u32>,
    pub ext_link_id: Option<u32>,
}

pub struct Database {
    // server: String,
    // port: u16
}

#[derive(Debug)]
pub enum DB {
    Website(Website),
    User(User),
    Metadata(Metadata)
}

impl Database {
    pub fn establish_connection(db_url: &str) -> MysqlConnection {
        MysqlConnection::establish(&db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
    }

    pub fn create_tables(conn: &MysqlConnection) -> Result<usize, diesel::result::Error>{
        let mut return_code = match sql_query("
            CREATE TABLE IF NOT EXISTS website (
                id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                title TEXT,
                text TEXT,
                url VARCHAR(100),
                rank DOUBLE,
                type_of_website VARCHAR(50)
            )
        ").execute(conn) {
            Ok(r_code)  => r_code,
            Err(err) => return Err(err),
        };

        return_code += match sql_query("
            CREATE TABLE IF NOT EXISTS users (
                id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(25) UNIQUE,
                rank DOUBLE NOT NULL,
                CountryISO_A2 VARCHAR(3)
            )
        ").execute(conn) {
            Ok(r_code)  => r_code,
            Err(err) => return Err(err),
        };

        return_code += match sql_query("
            CREATE TABLE IF NOT EXISTS metadata (
                id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                metadata TEXT,
                website_id INT UNSIGNED,
                FOREIGN KEY (website_id) REFERENCES website(id)
            )
        ").execute(conn) {
            Ok(r_code)  => r_code,
            Err(err) => return Err(err),
        };

        return_code += match sql_query("
            CREATE TABLE IF NOT EXISTS external_links (
                id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                url VARCHAR(2200)
            )
        ").execute(conn) {
            Ok(r_code)  => r_code,
            Err(err) => return Err(err),
        };

        return_code += match sql_query("
            CREATE TABLE IF NOT EXISTS website_ref_ext_links (
                id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                website_id INT UNSIGNED,
                ext_link_id INT UNSIGNED,
                FOREIGN KEY (website_id) REFERENCES website(id) ON DELETE CASCADE,
                FOREIGN KEY (ext_link_id) REFERENCES external_links(id) ON DELETE CASCADE
            )
        ").execute(conn) {
            Ok(r_code)  => r_code,
            Err(err) => return Err(err),
        };

        Ok(return_code)
    }

    pub fn insert_w(w: &Website, conn: &MysqlConnection) -> Result<usize, diesel::result::Error> {
        println!("{:?}", website.order(website::id.desc()).first::<Website>(conn));
        match insert_into(website).values(w).execute(conn) {
            Ok(r_code) => return Ok(r_code),
            Err(err) => return Err(err),
        }
    }

    pub fn insert_u(u: &User, conn: &MysqlConnection) -> Result<usize, diesel::result::Error> {
        match insert_into(users).values(u).execute(conn) {
            Ok(r_code) => return Ok(r_code),
            Err(err) => return Err(err),
        }
    }

    pub fn insert(db: &DB, conn: &MysqlConnection) -> Result<DB, diesel::result::Error> {
        match db {
            DB::Website(w) => {
                let inserted = insert_into(website).values(w).execute(conn);
                let ret = match website.order(website::id.desc()).first::<Website>(conn) {
                    Ok(r) => r,
                    Err(err) => return Err(err)
                };
                match inserted {
                    Ok(_) => return Ok(DB::Website(ret)),
                    Err(err) => return Err(err),
                }
            },
            DB::User(u) => {
                let inserted = insert_into(users).values(u).execute(conn);
                let ret = users.order(users::id.desc()).first::<User>(conn).unwrap();
                match inserted {
                    Ok(_) => return Ok(DB::User(ret)),
                    Err(err) => return Err(err),
                }
            },
            DB::Metadata(m) => {
                let inserted = insert_into(metadata).values(m).execute(conn);
                let ret = metadata.order(metadata::id.desc()).first::<Metadata>(conn).unwrap();
                match inserted {
                    Ok(_) => return Ok(DB::Metadata(ret)),
                    Err(err) => return Err(err),
                }
            }
        }
    }
}