mod id;

use std::fmt::Debug;

use diesel::{self, prelude::*};
use rocket::Rocket;
use uuid::Uuid;

#[database("sqlite_database")]
pub struct Connection(SqliteConnection);

embed_migrations!();

pub fn initialize(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = Connection::get_one(&rocket).expect("Database connection failed.");

    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            eprintln!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

pub mod schema {
    table! {
        events {
            id -> Binary,
            title -> Text,
            teaser -> Text,
            description -> Text,
        }
    }
    table! {
        occurrences {
            id -> Binary,
            event_id -> Binary,
            start -> Timestamp,
            duration -> Integer,
            location_id -> Binary,
        }
    }
    table! {
        locations {
            id -> Binary,
            name -> Text,
            address -> Text,
        }
    }
}

use chrono::NaiveDateTime;

use super::{Event, Id, Location, Occurrence, OccurrenceWithLocation};
pub use id::SqlId;
use schema::*;

#[derive(Queryable, Insertable, Debug, Identifiable, Clone, PartialEq, AsChangeset)]
#[table_name = "events"]
pub struct SqlEvent {
    pub id: SqlId<Event>,
    pub title: String,
    pub teaser: String,
    pub description: String,
}

impl From<SqlEvent> for (super::Id<Event>, Event) {
    fn from(event: SqlEvent) -> Self {
        (
            event.id.into(),
            Event {
                title: event.title,
                teaser: event.teaser,
                description: event.description,
            },
        )
    }
}

impl From<Event> for SqlEvent {
    fn from(event: Event) -> SqlEvent {
        let id = Uuid::new_v4();

        SqlEvent {
            id: id.into(),
            title: event.title,
            teaser: event.teaser,
            description: event.description,
        }
    }
}

#[derive(
    Queryable, Insertable, Clone, Debug, Identifiable, PartialEq, AsChangeset, Associations,
)]
#[belongs_to(SqlEvent, foreign_key = "event_id")]
#[belongs_to(SqlLocation, foreign_key = "location_id")]
#[table_name = "occurrences"]
pub struct SqlOccurrence {
    pub id: SqlId<Occurrence>,
    pub event_id: SqlId<Event>,
    pub start: NaiveDateTime,
    pub duration: i32,
    pub location_id: SqlId<Location>,
}

impl From<SqlOccurrence> for (Id<Occurrence>, OccurrenceWithLocation) {
    fn from(occurrence: SqlOccurrence) -> Self {
        (
            occurrence.id.into_inner().into(),
            (OccurrenceWithLocation {
                occurrence: Occurrence {
                    start: occurrence.start,
                    duration: occurrence.duration as u32,
                },
                location_id: occurrence.location_id.into(),
            }),
        )
    }
}

impl From<(OccurrenceWithLocation, SqlId<Event>)> for SqlOccurrence {
    fn from(
        (
            OccurrenceWithLocation {
                occurrence,
                location_id,
            },
            event_id,
        ): (OccurrenceWithLocation, SqlId<Event>),
    ) -> SqlOccurrence {
        let id = Uuid::new_v4();

        SqlOccurrence {
            id: id.into(),
            start: occurrence.start,
            duration: occurrence.duration as i32,
            location_id: location_id.into(),
            event_id,
        }
    }
}

#[derive(Queryable, Clone, Identifiable, Insertable, Debug, AsChangeset)]
#[table_name = "locations"]
pub struct SqlLocation {
    pub id: SqlId<Location>,
    pub name: String,
    pub address: String,
}
impl From<Location> for SqlLocation {
    fn from(location: Location) -> SqlLocation {
        let id = Uuid::new_v4();

        SqlLocation {
            id: id.into(),
            name: location.name,
            address: location.address,
        }
    }
}
impl From<SqlLocation> for (Id<Location>, Location) {
    fn from(location: SqlLocation) -> (Id<Location>, Location) {
        (
            location.id.into(),
            Location {
                name: location.name,
                address: location.address,
            },
        )
    }
}
