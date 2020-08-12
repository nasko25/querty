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