use chrono::NaiveDateTime;
use diesel::{self, prelude::*};
use rocket::fairing;
use rocket::fairing::Fairing;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Rocket};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Types

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub name: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Occurrence {
    pub start: NaiveDateTime,
    pub duration: Duration,
    pub id: Id,
}

type Duration = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub address: String,
}

type Id = Uuid;

// Store

pub struct Store(db::Connection);

use db::{SqlEvent, SqlLocation, SqlOccurrence};
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
                    .expect("Loading from database failed.")
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

pub trait Collection<T> {
    type Id;

    fn all(&self) -> Vec<(Self::Id, T)>;
    fn create(&self, item: T) -> Self::Id;
    fn read(&self, id: Self::Id) -> T;
    fn update(&self, id: Self::Id, new_item: T) -> T;
    fn delete(&self, id: Self::Id) -> T;
}

impl Collection<Location> for Store {
    type Id = Id;

    fn all(&self) -> Vec<(Self::Id, Location)> {
        use db::schema::locations::dsl::locations;
        locations
            .load::<SqlLocation>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|location| location.into())
            .collect()
    }

    fn create(&self, item: Location) -> Self::Id {
        use db::schema::locations;

        let sql_location: SqlLocation = item.into();
        diesel::insert_into(locations::table)
            .values(&sql_location)
            .execute(&*self.0)
            .expect("Error loading from database.");

        sql_location.id.into()
    }

    fn read(&self, item_id: Self::Id) -> Location {
        use db::schema::locations::dsl::locations;
        use db::SqlId;

        let sql_location = locations
            .find(SqlId::from(item_id))
            .first::<SqlLocation>(&*self.0)
            .expect("Error loading from database.");

        let (_, loc) = sql_location.into();
        loc
    }

    fn update(&self, item_id: Self::Id, new_item: Location) -> Location {
        use db::schema::locations::dsl::locations;
        use db::SqlId;

        let raw_id: SqlId = item_id.into();
        let (_, previous): (Id, Location) = locations
            .find(&raw_id)
            .first::<SqlLocation>(&*self.0)
            .expect("Error loading from database.")
            .into();

        diesel::update(locations.find(&raw_id))
            .set::<SqlLocation>(new_item.into())
            .execute(&*self.0)
            .expect("Error accessing database.");

        previous
    }

    fn delete(&self, id: Self::Id) -> Location {
        use db::schema::locations::dsl::locations;
        use db::SqlId;

        let raw_id: SqlId = id.into();
        let (_, previous): (Id, Location) = locations
            .find(&raw_id)
            .first::<SqlLocation>(&*self.0)
            .expect("Error loading from database.")
            .into();

        diesel::delete(locations.find(&raw_id))
            .execute(&*self.0)
            .expect("Error accessing database.");

        previous
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

mod db {
    use diesel::{self, prelude::*};
    use rocket::Rocket;
    use uuid::Uuid;

    #[database("sqlite_database")]
    pub struct Connection(SqliteConnection);

    embed_migrations!();

    pub fn initialize(rocket: Rocket) -> Result<Rocket, Rocket> {
        let conn = Connection::get_one(&rocket).expect("Database connection failed.");
        let result = match embedded_migrations::run(&*conn) {
            Ok(()) => Ok(rocket),
            Err(e) => {
                println!("Failed to run database migrations: {:?}", e);
                Err(rocket)
            }
        };

        Store(conn).create(Location { name: "Test".to_string(), address: "Somewhere".to_string()});

        result
    }

    pub mod schema {
        table! {
            events {
                id -> Binary,
                name -> Text,
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

    use std::io::Write;

    use super::*;
    use diesel::backend::Backend;
    use diesel::deserialize;
    use diesel::expression::{bound::Bound, AsExpression};
    use diesel::serialize::{self, Output};
    use diesel::sql_types::{Binary, HasSqlType};
    use diesel::sqlite::Sqlite;
    use diesel::types::{FromSql, ToSql};
    use schema::*;

    #[derive(Debug, Deserialize, FromSqlRow)]
    pub struct SqlId(Uuid);

    impl From<SqlId> for super::Id {
        fn from(id: SqlId) -> super::Id {
            id.0
        }
    }

    impl From<super::Id> for SqlId {
        fn from(id: super::Id) -> SqlId {
            SqlId(id)
        }
    }

    impl<DB: Backend + HasSqlType<Binary>> ToSql<Binary, DB> for SqlId {
        fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
            let bytes = self.0.as_bytes();
            <[u8] as ToSql<Binary, DB>>::to_sql(bytes, out)
        }
    }

    impl FromSql<Binary, Sqlite> for SqlId {
        fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
            let bytes_vec = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
            Ok(SqlId(Uuid::from_slice(&bytes_vec)?))
        }
    }

    impl AsExpression<Binary> for SqlId {
        type Expression = Bound<Binary, SqlId>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl<'a> AsExpression<Binary> for &'a SqlId {
        type Expression = Bound<Binary, &'a SqlId>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    #[derive(Queryable, Insertable)]
    #[table_name = "events"]
    pub struct SqlEvent {
        pub id: SqlId,
        pub name: String,
        pub teaser: String,
        pub description: String,
    }

    impl From<SqlEvent> for (Id, Event) {
        fn from(event: SqlEvent) -> (Id, Event) {
            (
                event.id.0,
                Event {
                    name: event.name,
                    teaser: event.teaser,
                    description: event.description,
                },
            )
        }
    }

    #[derive(Queryable, Insertable)]
    #[table_name = "occurrences"]
    pub struct SqlOccurrence {
        pub id: SqlId,
        pub event_id: SqlId,
        pub start: NaiveDateTime,
        pub duration: i32,
        pub location_id: SqlId,
    }

    impl From<SqlOccurrence> for (Id, Occurrence) {
        fn from(occurrence: SqlOccurrence) -> (Id, Occurrence) {
            (
                occurrence.id.0,
                Occurrence {
                    start: occurrence.start,
                    duration: occurrence.duration as u32,
                    id: occurrence.id.0,
                },
            )
        }
    }

    #[derive(Queryable, Insertable, AsChangeset)]
    #[table_name = "locations"]
    pub struct SqlLocation {
        pub id: SqlId,
        pub name: String,
        pub address: String,
    }

    impl From<Location> for SqlLocation {
        fn from(location: Location) -> SqlLocation {
            let id = Uuid::new_v4();

            SqlLocation {
                id: SqlId(id),
                name: location.name,
                address: location.address,
            }
        }
    }

    impl From<SqlLocation> for (Id, Location) {
        fn from(location: SqlLocation) -> (Id, Location) {
            (
                location.id.0,
                Location {
                    name: location.name,
                    address: location.address,
                },
            )
        }
    }
}
