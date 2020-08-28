// TODO solr
diesel::table! {
    // depending on where the keyword is (title, text, url), this will afftect the rank of the keyword
    // also consecutive keywords will have a higher rank the closer they are together
    // tf-idf
    website (id) {
        id -> Nullable<Unsigned<Integer>>,
        title -> Text,
        text -> Text,                           // the body/whole text of the website
        url -> Varchar,
        rank -> Integer,
        type_of_website -> Varchar,             // TODO table for website types?
    }
}
// TODO table metadata! + foreign key constraint https://github.com/ChristophWurst/diesel_many_to_many/blob/master/src/schema.rs
diesel::table! {
    metadata(id) {
        id -> Nullable<Unsigned<Integer>>,
        #[sql_name = "metadata"]
        metadata_text -> Text,
        website_id -> Nullable<Unsigned<Integer>>,
    }
}
diesel::joinable!(metadata -> website (website_id));

diesel::allow_tables_to_appear_in_same_query!(
    website,
    metadata
);
// ____________________________________________________________________________________________________________
// users
// TODO what data to track with users?
diesel::table! {
    users {
        id -> Nullable<Unsigned<Integer>>,
        username -> Varchar,
        rank -> Double,
        #[sql_name = "CountryISO_A2"]
        country_iso_a2 -> Varchar,
    }
}