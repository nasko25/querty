// TODO solr
diesel::table! {
    // depending on where the keyword is (title, text, url), this will afftect the rank of the keyword
    // also consecutive keywords will have a higher rank the closer they are together
    // tf-idf
    website (id) {
        id -> Nullable<Unsigned<Integer>>,
        title -> Text,
        text -> Longtext,                       // the body/whole text of the website
        url -> Varchar,
        base_url -> Varchar,
        rank -> Double,
        type_of_website -> Varchar,             // TODO table for website types?
    }
}

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

diesel::table! {
    external_links(id) {
        id -> Nullable<Unsigned<Integer>>,
        url -> Varchar,
    }
}

diesel::table! {
    website_ref_ext_links(id) {
        id -> Nullable<Unsigned<Integer>>,
        // TODO NOT NULL? (and not Nullable?)
        website_id -> Nullable<Unsigned<Integer>>,
        ext_link_id -> Nullable<Unsigned<Integer>>,
    }
}

diesel::joinable!(website_ref_ext_links -> website (website_id));
diesel::joinable!(website_ref_ext_links -> external_links (ext_link_id));

diesel::allow_tables_to_appear_in_same_query!(
    website,
    website_ref_ext_links,
    external_links
);

diesel::table! {
    next_urls_to_crawl(url) {
        id -> Nullable<Unsigned<Integer>>,
        url -> Varchar,
    }
}
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
