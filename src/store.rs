use chrono::NaiveDateTime;
use diesel::{self, prelude::*};
use rocket::fairing;
use rocket::fairing::Fairing;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Rocket};
use serde::{Deserialize, Serialize};

// Types

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub name: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Occurrence {
    pub event_id: Id,
    pub start: NaiveDateTime,
    pub duration: Duration,
    pub location_id: Id,
}

type Duration = u64;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub address: String,
}

type Id = i32;

// Store

pub struct Store(db::Connection);

use db::SqlLocation;
impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }

    pub fn read_all(&self) -> Vec<(Id, Location)> {
        use db::schema::locations::dsl::*;
        locations
            .load::<SqlLocation>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|loc| loc.into())
            .collect()
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
        db::Connection::fairing()
            .on_attach(rocket)
            .and_then(|rocket| db::initialize(rocket))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <db::Connection as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        db::Connection::from_request(request).map(Store)
    }
}

mod db {
    use diesel::{self, prelude::*};
    use rocket::Rocket;

    #[database("sqlite_database")]
    pub struct Connection(SqliteConnection);

    embed_migrations!();

    pub fn initialize(rocket: Rocket) -> Result<Rocket, Rocket> {
        let conn = Connection::get_one(&rocket).expect("Database connection failed.");
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

    use super::*;
    use schema::*;

    #[derive(Queryable, Insertable)]
    #[table_name = "locations"]
    pub struct SqlLocation {
        pub id: i32,
        pub name: String,
        pub address: String,
    }

    impl From<SqlLocation> for (Id, Location) {
        fn from(loc: SqlLocation) -> (Id, Location) {
            (
                loc.id,
                Location {
                    name: loc.name,
                    address: loc.address,
                },
            )
        }
    }
}
