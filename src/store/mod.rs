mod db;
#[macro_use]
pub mod action;
pub mod routes;

use chrono::NaiveDateTime;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::{fairing, fairing::Fairing, Rocket};
use uuid::Uuid;

use db::{SqlEvent, SqlLocation, SqlOccurrence};
use diesel::result::QueryResult;
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};

use action::Actions;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub name: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Occurrence {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub event_id: Id,
    pub location_id: Id,
}

type Duration = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub address: String,
}

mod location_action {
    use super::db::schema::locations::{dsl::locations as schema, table};
    use super::*;

    derive_actions!(Location, SqlLocation);
}

mod event_actions {
    use super::db::schema::events::{dsl::events as schema, table};
    use super::*;

    derive_actions!(Event, SqlEvent);
}

mod occurrence_actions {
    use super::db::schema::occurrences::{dsl::occurrences as schema, table};
    use super::*;

    derive_actions!(Occurrence, SqlOccurrence);
}

pub type Id = Uuid;

pub struct Store(db::Connection);

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }

    pub fn read_all(&self) -> (Vec<(Id, Location)>, Vec<(Id, Event, Vec<Occurrence>)>) {
        let locs: Vec<(Id, Location)> = self.all();

        use db::schema::events::dsl::events;
        let evts: Vec<(Id, Event, Vec<Occurrence>)> = events
            .load::<SqlEvent>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_event| {
                let (id, event) = sql_event.into();

                use db::schema::occurrences::dsl::occurrences;
                let occrs: Vec<Occurrence> = occurrences
                    .load::<SqlOccurrence>(&*self.0)
                    .expect("Loading from database failed. bla")
                    .into_iter()
                    .map(|sql_occurrence| {
                        let (_, occurrence) = sql_occurrence.into();

                        occurrence
                    })
                    .collect();

                (id, event, occrs)
            })
            .collect();

        (locs, evts)
    }
}

pub struct StoreFairing;

impl Fairing for StoreFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Events Store Fairing",
            kind: fairing::Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let result = db::Connection::fairing()
            .on_attach(rocket)
            .and_then(|rocket| db::initialize(rocket));

        result
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <db::Connection as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        db::Connection::from_request(request).map(Store)
    }
}
