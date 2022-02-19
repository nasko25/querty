## Dependencies
The project uses:
* MySQL - to store user information as well as indexed websites and keywords. The config for the database can be found in `config/config.toml`.
* Apache Solr - which will be used as a backbone for queries and indexing the found websites.
* Rust nightly - one of the Rust libraries requires rust nightly (more information can be found in the Rust nightly section below)

The application needs a configured mysql instance (the configuration can be specified in the config file), with an already created empty database.
It will create the tables it requires.

In order to use the DataImportHandler offered by Apache Solr, you also need to change the database information in `config/solr/data-config.xml`.
The application requires that Solr already has imported the data from the mysql instance (as they will hopefully run in parallel, and the mysql instance will be used as a backup in case solr fails for some reason).

## Some mysql errors I encountered
You need mysql-connector-java in order to use the DataImportHandler. It can be downloaded from MySQL's website and should be placed in {SOLR_HOME_DIR}/contrib/dataimporthandler-extras/lib/

There was also a time zone issue for me, so I had to run
`mysql_tzinfo_to_sql /usr/share/zoneinfo | mysql -u root mysql -p`
And then change my default time zone to Europe/Sofia:
I added:
```
[mysqld]
default_time_zone=Europe/Sofia
```
to `/etc/mysql/my.cnf`

---

When running `cargo build` I got the following error:
"/usr/bin/ld: cannot find -lmysqlclient: No such file or directory"
After `apt search mysqlclient`, I found a `apt install`ed `default-libmysqlclient-dev`.

## First time setting up Solr:
`./bin/solr start -e cloud`

## Starting Solr
```
./bin/solr start -c -p 8983 -s example/cloud/node1/solr
./bin/solr start -c -p 7574 -s example/cloud/node2/solr -z localhost:9983
```

## Stopping Solr
```./bin/solr stop -all```

## Creating and Deleting collections
The scripts for creating and deleting a collection can be found as a comment in solrconfig.xml
More information can be found here: https://lucene.apache.org/solr/guide/8_6/solr-tutorial.html

## How to run?
Before running set the environment variables `SOLR_PATH` and `SOLR_CONFIG_PATH` to point to solr and the solr configuration files respectively. The solr configuration files are in `config/solr`, so set SOLR_CONFIG_PATH to something like `~/querty/config/solr` (tildas are supported).
Another option is to set the `path_to_solr` and `path_to_solr_config` variables in `config/config.toml` (The environment variables are checked first and if they are not set, the config file variables are used).

To test the rust code:
```bash
cargo run
```
(make sure the mysql database and solr are running and the configuration in `config/config.toml` is correct)

To test or run the website classifier server check the README in `classifier/`

## Rust Nightly
Since the web API is using [Rocket](https://rocket.rs/) (for now), and Rocket requires rust-nighly, in order to `cargo run` the project, you will have to configure rust-nightly:
```
rustup default nightly
```
or if you want to configure it only for the current directory:
```
rustup override set nightly
```
Then update rust:
```
rustup update && cargo update
```

### Note:
I'm not sure how good of an idea it is to create the web API in rust. I mainly went for it, because the idea of this project is for me to learn rust. If this project ever becomes more serious, the web API will need to be rewritten in a more suitable language and framework. However, this is not a priority for now.
