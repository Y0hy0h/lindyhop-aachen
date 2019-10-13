use diesel::prelude::*;
use rocket::Rocket;
use rocket_contrib::databases::diesel;

#[database("sqlite_database")]
pub struct Connection(diesel::SqliteConnection);

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

pub type SqlId = i32;

#[derive(Queryable, Insertable, Debug, Identifiable, Clone, PartialEq, AsChangeset)]
#[table_name = "events"]
pub struct SqlEvent {
    pub id: SqlId,
    pub title: String,
    pub teaser: String,
    pub description: String,
}

#[derive(
    Queryable, Insertable, Clone, Debug, Identifiable, PartialEq, AsChangeset, Associations,
)]
#[belongs_to(SqlEvent, foreign_key = "event_id")]
#[belongs_to(SqlLocation, foreign_key = "location_id")]
#[table_name = "occurrences"]
pub struct SqlOccurrence {
    pub id: SqlId,
    pub event_id: SqlId,
    pub start: NaiveDateTime,
    pub duration: i32,
    pub location_id: SqlId,
}

#[derive(Queryable, Clone, Identifiable, Insertable, Debug, AsChangeset)]
#[table_name = "locations"]
pub struct SqlLocation {
    pub id: SqlId,
    pub name: String,
    pub address: String,
}
