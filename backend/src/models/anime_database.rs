// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::anime_database;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: anime_database = serde_json::from_str(&json).unwrap();
// }

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeDatabase {
    #[serde(rename = "$schema")]
    schema: String,

    license: License,

    repository: String,

    score_range: ScoreRange,

    last_update: String,

    data: Vec<Datum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Datum {
    sources: Vec<String>,

    title: String,

    #[serde(rename = "type")]
    datum_type: Type,

    episodes: i64,

    status: Status,

    anime_season: AnimeSeason,

    picture: String,

    thumbnail: String,

    duration: Option<Duration>,

    score: Option<Score>,

    synonyms: Vec<String>,

    studios: Vec<String>,

    producers: Vec<String>,

    related_anime: Vec<String>,

    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeSeason {
    season: Season,

    year: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Season {
    #[serde(rename = "FALL")]
    Fall,

    #[serde(rename = "SPRING")]
    Spring,

    #[serde(rename = "SUMMER")]
    Summer,

    #[serde(rename = "UNDEFINED")]
    Undefined,

    #[serde(rename = "WINTER")]
    Winter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "MOVIE")]
    Movie,

    #[serde(rename = "ONA")]
    Ona,

    #[serde(rename = "OVA")]
    Ova,

    #[serde(rename = "SPECIAL")]
    Special,

    #[serde(rename = "TV")]
    Tv,

    #[serde(rename = "UNKNOWN")]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    value: i64,

    unit: Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Unit {
    #[serde(rename = "SECONDS")]
    Seconds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    arithmetic_geometric_mean: f64,

    arithmetic_mean: f64,

    median: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "FINISHED")]
    Finished,

    #[serde(rename = "ONGOING")]
    Ongoing,

    #[serde(rename = "UNKNOWN")]
    Unknown,

    #[serde(rename = "UPCOMING")]
    Upcoming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    name: String,

    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreRange {
    min_inclusive: f64,

    max_inclusive: f64,
}
