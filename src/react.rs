use std::mem::discriminant;
use std::fmt;
use diesel::MysqlConnection;

use crate::solr;
use crate::crawler;

// for now all users reacts will change the website's rank with +/-1.0
// later this could depend on users' ranks
// var = variance
// #[derive(PartialEq)]
pub enum React {
    Upvote { var: f64 },
    Downvote { var: f64 },
}

pub(crate) enum ReactError {
    InvalidArgument { mes: String },
    RankNotUpdated { mes: String },
    GenericError,
    NoWebsiteWithThatId
}

impl fmt::Display for ReactError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReactError::InvalidArgument { mes } => write!(f, "{}", mes),
            ReactError::RankNotUpdated { mes } => write!(f, "{}", mes),
            ReactError::GenericError => write!(f, "An error occured in user_react()"), // TODO more sensible error message
            ReactError::NoWebsiteWithThatId => write!(f, "No website with the provided id exists in solr.")
        }
    }
}

pub(super) fn user_react(website_id: &str, react_type: React) -> Result<f64, ReactError> {
    println!("Updating the website with id {} after user react.", website_id);
    let mut websites_saved = solr::req(format!("id:\"{}\"", website_id)).unwrap();
    // websites_saved should either be empty (if there are no websites with that id in solr)
    //      in which case the function will return an error
    //
    // or websites_saved should have a length of 1 (because olny 1 website should have been fetched from solr
    // because id should be unique)
    if websites_saved.is_empty() {
        return Err(ReactError::NoWebsiteWithThatId);
    }
    // since website ranks should be between -10 and 10 and user react FOR NOW will only update it
    // with +/-1, I can do this ugly check
    else if websites_saved.len() == 1 && ((websites_saved[0].rank <= 9.0 && discriminant(&react_type) == discriminant(&React::Upvote{ var: 0.0 })) || (websites_saved[0].rank >= -9.0 && discriminant(&react_type) == discriminant(&React::Downvote {var: 0.0}))) {
        println!("{:?}'s old rank: {}", websites_saved[0].id, websites_saved[0].rank);
        websites_saved[0].rank += match react_type {
            React::Upvote { var } => {
                println!("Upvote variance: {}", var);
                1.0
            },
            React::Downvote { var } => {
                println!("Downvote variance: {}", var);
                -1.0
            },
        };
    }
    else if websites_saved.len() != 1 {
        return Err(ReactError::InvalidArgument { mes: "Vector is not empty and has a size != 1.".to_string() });
    }

    else {
        return Err(ReactError::GenericError);
    }
    let crawler = crawler::Crawler {};
    crawler.analyse_website(&websites_saved[0].url, &websites_saved).unwrap();

    if websites_saved.is_empty() {
        return Err(ReactError::RankNotUpdated { mes: "Url has not been analysed previously, so its rank was set to 0.".to_string() });
    }

    Ok(websites_saved[0].rank)
}
