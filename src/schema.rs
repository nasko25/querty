// TODO solr
diesel::table! {
    // depending on where the keyword is (title, text, url), this will afftect the rank of the keyword
    // also consecutive keywords will have a higher rank the closer they are together
    // tf-idf
    website (id) {
        id -> Nullable<Integer>,
        title -> Text,
        metadata -> Text,
        text -> Text,                           // the body/whole text of the website
        url -> Varchar,
        rank -> Integer,
        type_of_website -> Varchar,             // TODO table for website types?
    }
}
diesel::table! {
    // TODO probably not needed table!
    keywords (website_id, keyword) {
        website_id -> Integer,
        keyword -> Varchar,
        rank_per_kw -> Varchar,
    }
}
// ____________________________________________________________________________________________________________
// users
// TODO what data to track with users?
diesel::table! {
    users {
        id -> Integer,
        username -> Varchar,
        rank -> Double,
        CountryISO_A2 -> Varchar,
    }
}