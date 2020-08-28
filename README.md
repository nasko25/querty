## Dependencies
The project uses:
    * MySQL - to store user information as well as indexed websites and keywords. The config for the database can be found in `config/config.toml`.
    * Apache Solr - which will be used as a backbone for queries and indexing the found websites.

The application needs a configured mysql instance (the configuration can be specified in the config file), with an already created empty database.
It will create the tables it requires.
    
In order to use the DataImportHandler offered by Apache Solr, you also need to change the database information in `config/solr/data-config.xml`.
The application requires that Solr already has imported the data from the mysql instance (as they will hopefully run in parallel, and the mysql instance will be used as a backup in case solr fails for some reason).