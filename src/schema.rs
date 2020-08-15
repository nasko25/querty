// TODO solr
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