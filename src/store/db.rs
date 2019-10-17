use juniper::{GraphQLInputObject};
use rocket::Rocket;
use rocket_contrib::databases::diesel;

#[database("sqlite_database")]
pub struct Connection(diesel::SqliteConnection);

embed_migrations!();

pub fn initialize(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = Connection::get_one(&rocket).expect("Database connection failed.");

    match embedded_migrations::run(&*conn) {
        Ok(()) => {
            println!("Ran migrations successfully."); // TODO: Use proper logging.
            Ok(rocket)
        }
        Err(e) => {
            eprintln!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

pub mod schema {
    table! {
        events {
            id -> Integer,
            title -> Text,
            teaser -> Text,
            description -> Text,
        }
    }
    table! {
        occurrences {
            id -> Integer,
            event_id -> Integer,
            start -> Timestamp,
            duration -> Integer,
            location_id -> Integer,
        }
    }
    table! {
        locations {
            id -> Integer,
            name -> Text,
            address -> Text,
        }
    }
}

use chrono::NaiveDateTime;

use schema::*;

pub type Id = i32;

#[derive(Queryable, Insertable, Debug, Identifiable, Clone, PartialEq, AsChangeset)]
#[table_name = "events"]
pub struct Event {
    pub id: Id,
    pub title: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Clone, Debug, Insertable, AsChangeset, GraphQLInputObject)]
#[table_name = "events"]
pub struct NewEvent {
    pub title: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Clone, Debug, AsChangeset, GraphQLInputObject)]
#[table_name = "events"]
pub struct UpdateEvent {
    pub title: Option<String>,
    pub teaser: Option<String>,
    pub description: Option<String>,
}

#[derive(
    Queryable,
    Insertable,
    Clone,
    Debug,
    Identifiable,
    PartialEq,
    AsChangeset,
    Associations,
)]
#[belongs_to(Event, foreign_key = "event_id")]
#[belongs_to(Location, foreign_key = "location_id")]
#[table_name = "occurrences"]
pub struct Occurrence {
    pub id: Id,
    pub event_id: Id,
    pub start: NaiveDateTime,
    pub duration: i32,
    pub location_id: Id,
}

#[derive(Clone, Debug, Insertable, AsChangeset, GraphQLInputObject)]
#[table_name = "occurrences"]
pub struct NewOccurrence {
    pub event_id: Id,
    pub start: NaiveDateTime,
    pub duration: i32,
    pub location_id: Id,
}

#[derive(Clone, Debug, AsChangeset, GraphQLInputObject)]
#[table_name = "occurrences"]
pub struct UpdateOccurrence {
    pub event_id: Option<Id>,
    pub start: Option<NaiveDateTime>,
    pub duration: Option<i32>,
    pub location_id: Option<Id>,
}

#[derive(Queryable, Clone, Identifiable, Debug)]
#[table_name = "locations"]
pub struct Location {
    pub id: Id,
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug, Insertable, AsChangeset, GraphQLInputObject)]
#[table_name = "locations"]
pub struct NewLocation {
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug, AsChangeset, GraphQLInputObject)]
#[table_name = "locations"]
pub struct UpdateLocation {
    pub name: Option<String>,
    pub address: Option<String>,
}
