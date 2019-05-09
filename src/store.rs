use chrono::NaiveDateTime;
use diesel::{self, prelude::*};
use diesel::{Insertable, Queryable};
use rocket::fairing;
use rocket::fairing::Fairing;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Rocket};
use serde::{Deserialize, Serialize};

// DB

#[database("sqlite_database")]
pub struct DbConn(SqliteConnection);

embed_migrations!();

pub fn initialize(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = DbConn::get_one(&rocket).expect("Database connection failed.");
    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            println!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

pub mod schema {
    table! {
        events {
            id -> Integer,
            name -> Text,
            teaser -> Text,
            description -> Text,
        }
    }
    table! {
        occurrences {
            id -> Integer,
            start -> Timestamp,
            event_id -> Integer,
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

use schema::*;

// Types

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: Id,
    pub name: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Occurrence {
    pub id: Id,
    pub start: NaiveDateTime,
    pub duration: Duration,
    pub location_id: Id,
}

type Duration = u64;

#[derive(Serialize, Deserialize, Debug, Queryable, Insertable)]
#[table_name = "locations"]
pub struct Location {
    pub id: Id,
    pub name: String,
    pub address: String,
}

type Id = i32;

// Store

pub struct Store(DbConn);

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }

    pub fn read_all(&self) -> Vec<Location> {
        use schema::locations::dsl::*;
        locations
            .load::<Location>(&*self.0)
            .expect("Error loading dummy loc.")
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
        DbConn::fairing()
            .on_attach(rocket)
            .and_then(|rocket| initialize(rocket))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <DbConn as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        DbConn::from_request(request).map(Store)
    }
}
