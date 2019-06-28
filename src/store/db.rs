use std::fmt::Debug;

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

    result
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

use std::io::Write;
use std::marker::PhantomData;

use super::*;
use diesel::backend::Backend;
use diesel::deserialize;
use diesel::expression::{bound::Bound, AsExpression};
use diesel::serialize::{self, Output};
use diesel::sql_types::{Binary, HasSqlType};
use diesel::sqlite::Sqlite;
use diesel::types::{FromSql, ToSql};
use schema::*;

// SqlId implementation inspired by https://github.com/forte-music/core/blob/fc9cd6217708b0dd6ae684df3a53276804479c59/src/models/id.rs#L67
#[derive(Debug, Deserialize, FromSqlRow, Clone, Hash, PartialEq, Eq)]
pub struct SqlId<Item>(Uuid, PhantomData<Item>);

impl<Item> From<Uuid> for SqlId<Item> {
    fn from(uuid: Uuid) -> Self {
        SqlId(uuid, PhantomData)
    }
}

impl<Item> From<SqlId<Item>> for super::Id<Item> {
    fn from(id: SqlId<Item>) -> super::Id<Item> {
        id.0.into()
    }
}

impl<Item> From<super::Id<Item>> for SqlId<Item> {
    fn from(id: super::Id<Item>) -> SqlId<Item> {
        id.into()
    }
}

impl<DB: Backend + HasSqlType<Binary>, Item: Debug> ToSql<Binary, DB> for SqlId<Item> {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        let bytes = self.0.as_bytes();
        <[u8] as ToSql<Binary, DB>>::to_sql(bytes, out)
    }
}

impl<Item> FromSql<Binary, Sqlite> for SqlId<Item> {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        let bytes_vec = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        Ok(Uuid::from_slice(&bytes_vec)?.into())
    }
}

impl<Item> AsExpression<Binary> for SqlId<Item> {
    type Expression = Bound<Binary, SqlId<Item>>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'a, Item> AsExpression<Binary> for &'a SqlId<Item> {
    type Expression = Bound<Binary, &'a SqlId<Item>>;

    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

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
            occurrence.id.0.into(),
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
            event_id: event_id,
        }
    }
}

#[derive(Queryable, Clone,Identifiable, Insertable, Debug, AsChangeset)]
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
