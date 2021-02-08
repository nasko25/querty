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

// TODO comments

// TODO maybe add a field for links to other websies that can be used by something like PageRank?
// TODO add a field last_visited that will indicate when the bot (that will be used to rank the quality of the website) last visited the site
#[derive(Queryable, Insertable, Debug, Serialize, Deserialize, Identifiable, Clone)]
#[table_name = "website"]
pub struct Website {
    #[serde(deserialize_with = "from_str")]
    pub id: Option<u32>,
    pub title: String,
    pub text: String,
    pub url: String,
    pub base_url: String,
    pub rank: f64,
    pub type_of_website: String
}

use std::str::FromStr;
use std::fmt::Display;
use serde::de::{self, Deserialize, Deserializer};
// CITATION: https://github.com/serde-rs/json/issues/317#issuecomment-300251188
pub fn from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
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

#[derive(Queryable, Insertable, Debug, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub rank: f64,
    pub country_iso_a2: String
}

#[derive(Identifiable, Queryable, Associations, Debug, Insertable, Clone)]
#[belongs_to(Website)]
#[table_name = "metadata"]
pub struct Metadata {
    pub id: Option<u32>,
    pub metadata_text: String,
    pub website_id: Option<u32>,
}

// TODO PartialEq will in the future be implied by Eq
#[derive(Identifiable, Queryable, Associations, Debug, Insertable, Clone, Hash, Eq, PartialEq)]
#[table_name = "external_links"]
pub struct ExternalLink {
    pub id: Option<u32>,
    pub url: String,
}

#[derive(Identifiable, Queryable, Associations, Debug, Insertable, Clone, Hash, Eq, PartialEq)]
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
    Metadata(Metadata),
    ExternalLink(ExternalLink),
    WebsiteRefExtLink(WebsiteRefExtLink)
}

impl Database {
    pub fn establish_connection(db_url: &str) -> MysqlConnection {
        MysqlConnection::establish(&db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
    }

    pub fn create_tables(conn: &MysqlConnection) -> Result<usize, diesel::result::Error>{
        // TODO website url should be unique
        let mut return_code = match sql_query("
            CREATE TABLE IF NOT EXISTS website (
                id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                title TEXT,
                text TEXT,
                url VARCHAR(100) UNIQUE,
                base_url VARCHAR(100),
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

        // TODO (website_id, ext_link_id) should be unique
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

    // drop all tables in the database
    // useful in development when changing the db or solr
    // that will create an inconsistent state
    // TODO maybe add a function that deletes everyting from solr as well
        // and call them both in a clear_databases() function; probably in main
    pub fn drop_tables(conn: &MysqlConnection) -> Result<usize, diesel::result::Error> {
        return sql_query("
            DROP TABLE IF EXISTS users, metadata, website_ref_ext_links, external_links, website"
        ).execute(conn);
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
            },
            DB::ExternalLink(ext_l) => {
                let inserted = insert_into(external_links).values(ext_l).execute(conn);
                let ret = external_links.order(external_links::id.desc()).first::<ExternalLink>(conn).unwrap();
                match inserted {
                    Ok(_) => return Ok(DB::ExternalLink(ret)),
                    Err(err) => return Err(err),
                }
            },
            DB::WebsiteRefExtLink(web_ref_ext_link) => {
                let inserted = insert_into(website_ref_ext_links).values(web_ref_ext_link).execute(conn);
                let ret = website_ref_ext_links.order(website_ref_ext_links::id.desc()).first::<WebsiteRefExtLink>(conn).unwrap();
                match inserted {
                    Ok(_) => return Ok(DB::WebsiteRefExtLink(ret)),
                    Err(err) => return Err(err),
                }
            }
        }
    }

    // select website(s)
    // TODO return Result<>
    pub fn select_w(ids: &Option<Vec<u32>>, conn: &MysqlConnection) -> Vec<Website> {
        let mut websites = Vec::<Website>::new();
        match ids {
            Some(ids_ref) => {
                for w_id in ids_ref {
                    for w in crate::schema::website::dsl::website.filter(crate::schema::website::dsl::id.eq(w_id)).load::<Website>(conn).expect("Error loading website").iter() {
                        websites.push(w.clone()); // TODO is clone necessary?
                    }
                }
            },
            None => {
                for w in crate::schema::website::dsl::website.load::<Website>(conn).expect("Error loading websites").iter() {
                    websites.push(w.clone());
                }
            }
        }

        websites
    }

    pub fn select_m(websites: &Option<Vec<Website>>, conn: &MysqlConnection) -> Vec<Metadata>{
        let mut md = Vec::<Metadata>::new();
        match websites {
            Some(ws) => {
                for w in ws {
                    for m in metadata::table.filter(metadata::website_id.eq(w.id)).load::<Metadata>(conn).expect("Error loading metadata").iter() {
                        md.push(m.clone());
                    }
                }
            },
            None => {
                for m in metadata.load::<Metadata>(conn).expect("Error loading all metadata").iter() {
                    md.push(m.clone());
                }
            }
        }
        md
    }

    pub fn select_el(website_opt: &Option<&Website>, conn: &MysqlConnection) -> Vec<ExternalLink>{
        let mut els = Vec::<ExternalLink>::new();
        if website_opt.is_some() {
            let link_ids = WebsiteRefExtLink::belonging_to(website_opt.unwrap()).select(website_ref_ext_links::ext_link_id).load::<Option<u32>>(conn).expect("Error loading external_link ids");
            for link_id in link_ids {
                for el in external_links::table.filter(external_links::id.eq(link_id)).load::<ExternalLink>(conn).expect("Error loading external links.") {
                    els.push(el);
                }
            }
        }
        els
    }

    pub fn select_webref(website_opt: &Option<&Website>, conn: &MysqlConnection) -> Vec<WebsiteRefExtLink> {
        let mut webrefs = Vec::<WebsiteRefExtLink>::new();
        if website_opt.is_some() {
            let link_ids = WebsiteRefExtLink::belonging_to(website_opt.unwrap()).load::<WebsiteRefExtLink>(conn).expect("Error loading external_link ids");
            for link_id in link_ids {
                    webrefs.push(link_id);
            }
        }
       webrefs
    }

    // update the website_id_opt based on its id
    pub fn update(db: &DB, conn: &MysqlConnection) -> Result<DB, diesel::result::Error>{
        match db {
            DB::Website(w) => {
                // let w = website_id_opt.as_ref().unwrap();
                diesel::update(website.filter(crate::schema::website::dsl::id.eq(w.id))).set(
                    ( crate::schema::website::title.eq(&w.title),
                    crate::schema::website::text.eq(&w.text),
                    crate::schema::website::url.eq(&w.url),
                    crate::schema::website::base_url.eq(&w.base_url),
                    crate::schema::website::rank.eq(&w.rank),
                    crate::schema::website::type_of_website.eq(&w.type_of_website) )
                ).execute(conn)?;
                // TODO maybe use select_w
                let updated_row_vec = crate::schema::website::dsl::website.filter(crate::schema::website::dsl::id.eq(w.id)).load::<Website>(conn).expect("Error loading website");
                let updated_row = updated_row_vec.get(0).unwrap().clone();
                Ok(DB::Website(updated_row))
            },
            DB::User(u) => {
                diesel::update(users.filter(crate::schema::users::dsl::id.eq(u.id))).set(
                    ( crate::schema::users::username.eq(&u.username),
                    crate::schema::users::rank.eq(&u.rank),
                    crate::schema::users::country_iso_a2.eq(&u.country_iso_a2) )
                ).execute(conn)?;
                let updated_row_vec = crate::schema::users::dsl::users.filter(crate::schema::users::dsl::id.eq(u.id)).load::<User>(conn).expect("Error loading user");
                let updated_row = updated_row_vec.get(0).unwrap().clone();
                Ok(DB::User(updated_row))
            },
            DB::Metadata(m) => {
                diesel::update(metadata.filter(crate::schema::metadata::dsl::id.eq(m.id))).set(
                    ( crate::schema::metadata::metadata_text.eq(&m.metadata_text),
                    crate::schema::metadata::website_id.eq(&m.website_id) )
                ).execute(conn)?;
                let updated_row_vec = crate::schema::metadata::dsl::metadata.filter(crate::schema::metadata::dsl::id.eq(m.id)).load::<Metadata>(conn).expect("Error loading metadata");
                let updated_row = updated_row_vec.get(0).unwrap().clone();
                Ok(DB::Metadata(updated_row))
            },
            DB::ExternalLink(ext_l) => {
                diesel::update(external_links.filter(crate::schema::external_links::dsl::id.eq(ext_l.id))).set(
                    ( crate::schema::external_links::url.eq(&ext_l.url) )
                ).execute(conn)?;
                let updated_row_vec = crate::schema::external_links::dsl::external_links.filter(crate::schema::external_links::dsl::id.eq(ext_l.id)).load::<ExternalLink>(conn).expect("Error loading external links");
                let updated_row = updated_row_vec.get(0).unwrap().clone();
                Ok(DB::ExternalLink(updated_row))
            },
            DB::WebsiteRefExtLink(web_ref_ext_link) => {
                diesel::update(website_ref_ext_links.filter(crate::schema::website_ref_ext_links::dsl::id.eq(web_ref_ext_link.id))).set(
                    ( crate::schema::website_ref_ext_links::website_id.eq(&web_ref_ext_link.website_id),
                    crate::schema::website_ref_ext_links::ext_link_id.eq(&web_ref_ext_link.ext_link_id) )
                ).execute(conn)?;
                let updated_row_vec = crate::schema::website_ref_ext_links::dsl::website_ref_ext_links.filter(crate::schema::website_ref_ext_links::dsl::id.eq(web_ref_ext_link.id)).load::<WebsiteRefExtLink>(conn).expect("Error loading web_ref_ext_link");
                let updated_row = updated_row_vec.get(0).unwrap().clone();
                Ok(DB::WebsiteRefExtLink(updated_row))
            }
        }
    }
}
