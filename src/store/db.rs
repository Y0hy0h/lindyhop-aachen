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

    let store = Store(conn);

    store.create(Location { name: "Test".to_string(), address: "Somewhere".to_string()}).expect("Could not create entry!");
    
    let event = store.create(Event { name: "Social Dance".into(), teaser: "Blub".into(), description: "kakak".into() }).expect("Could not create event entry!");

    println!("{:?}", event);

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
            end -> Timestamp,
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

#[derive(Queryable, Insertable, AsChangeset)]
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

impl From<Event> for SqlEvent {
    fn from(event: Event) -> SqlEvent {
        let id = Uuid::new_v4();

        SqlEvent {
            id: SqlId(id),
            name: event.name,
            teaser: event.teaser,
            description: event.description
        }
    }
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "occurrences"]
pub struct SqlOccurrence {
    pub id: SqlId,
    pub event_id: SqlId,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub location_id: SqlId,
}

impl From<SqlOccurrence> for (Id, Occurrence) {
    fn from(occurrence: SqlOccurrence) -> (Id, Occurrence) {
        (
            occurrence.id.0,
            Occurrence {
                start: occurrence.start,
                end: occurrence.end,
                location_id: occurrence.location_id.into(),
                event_id: occurrence.event_id.into()
            },
        )
    }
}

impl From<Occurrence> for SqlOccurrence {
    fn from(occurrence: Occurrence) -> SqlOccurrence {
        let id = Uuid::new_v4();

        SqlOccurrence {
            id: SqlId(id),
            start: occurrence.start,
            end: occurrence.end,
            location_id: occurrence.location_id.into(),
            event_id: occurrence.event_id.into()
        }
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


