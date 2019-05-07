use chrono::prelude::*;

use chrono::serde::ts_seconds;

use serde::Serialize;

#[derive(Serialize)]
pub struct Event<'a> {
    pub name: &'a str,
    pub teaser: &'a str,
    pub description: &'a str,
    pub occurrences: Vec<Occurrence<'a>>,
}

#[derive(Serialize)]
pub struct Occurrence<'a> {
    #[serde(with = "ts_seconds")]
    pub start: DateTime<Utc>,
    pub duration: Duration,
    pub location: &'a Location<'a>,
}

type Duration = u64;

#[derive(Serialize)]
pub struct Location<'a> {
    pub name: &'a str,
    pub address: &'a str,
}
