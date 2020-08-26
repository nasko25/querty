// TODO tables https://docs.rs/diesel/1.4.5/diesel/macro.table.html
// https://github.com/diesel-rs/diesel/blob/master/examples/mysql/getting_started_step_1/src/schema.rs
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::{ insert_into, sql_query };

use crate::schema::website;
use crate::schema::website::dsl::*;
use crate::schema::users;
use crate::schema::users::dsl::*;


#[derive(Queryable, Insertable, Debug, Serialize, Deserialize)]
#[table_name = "website"]
pub struct Website {
    pub id: Option<i32>,
    pub title: String,
    pub metadata: String,
    pub text: String,
    pub url: String,
    pub rank: i32,
    pub type_of_website: String
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "users"]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub rank: f64,
    pub CountryISO_A2: String
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

#[derive(Debug)]
pub enum DB {
    Website(Website),
    User(User),
}

impl Database {
    pub fn establish_connection(db_url: &str) -> MysqlConnection {
        MysqlConnection::establish(&db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
    }

    pub fn create_tables(conn: &MysqlConnection) -> Result<usize, diesel::result::Error>{
        let mut return_code = match sql_query("
            CREATE TABLE IF NOT EXISTS website (
                id INT AUTO_INCREMENT PRIMARY KEY,
                title TEXT,
                metadata TEXT,
                text TEXT,
                url VARCHAR(100),
                rank INT,
                type_of_website VARCHAR(50)
            )
        ").execute(conn) {
            Ok(r_code)  => r_code,
            Err(err) => return Err(err),
        };

        return_code += match sql_query("
            CREATE TABLE IF NOT EXISTS users (
                id INT AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(25) UNIQUE,
                rank DOUBLE NOT NULL,
                CountryISO_A2 VARCHAR(3)
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
            }
        }
    }
}