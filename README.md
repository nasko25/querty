## Dependencies
The project uses:
* MySQL - to store user information as well as indexed websites and keywords. The config for the database can be found in `config/config.toml`.
* Apache Solr - which will be used as a backbone for queries and indexing the found websites.

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