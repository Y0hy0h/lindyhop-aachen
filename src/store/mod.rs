mod db;
mod model;
pub mod routes;

use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use chrono::{NaiveDate, NaiveDateTime};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::{fairing, fairing::Fairing, Rocket};
use uuid::Uuid;

use db::{SqlEvent, SqlLocation, SqlOccurrence};
use diesel::result::QueryResult;
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};

pub use model::*;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct Id<Item> {
    id: Uuid,
    #[serde(skip)]
    phantom: PhantomData<Item>,
}

impl<Item> From<Uuid> for Id<Item> {
    fn from(uuid: Uuid) -> Self {
        Id {
            id: uuid,
            phantom: PhantomData,
        }
    }
}

pub struct Store(db::Connection);

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }

    pub fn read_all(&self) -> Overview {
        let locs: HashMap<Id<Location>, Location> = self.all();
        let evts: HashMap<Id<Event>, EventWithOccurrences> = self.all();

        Overview {
            locations: locs,
            events: evts,
        }
    }

    pub fn occurrences_by_date(&self) -> BTreeMap<NaiveDate, Vec<OccurrenceWithEvent>> {
        use db::schema::events::dsl::events;
        use db::schema::occurrences::dsl::{occurrences, start};

        let sql_occurrences = occurrences
            .filter(start.gt(chrono::Local::now().naive_local()))
            .order(start.asc())
            .load::<SqlOccurrence>(&*self.0)
            .unwrap();

        sql_occurrences
            .into_iter()
            .map(|sql_occurrence| {
                let sql_event = events
                    .find(sql_occurrence.event_id.clone())
                    .first::<SqlEvent>(&*self.0)
                    .unwrap();
                let (_, occurrence) = sql_occurrence.into();
                let (_, event) = sql_event.into();
                OccurrenceWithEvent { occurrence, event }
            })
            .fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<NaiveDate, Vec<OccurrenceWithEvent>>, entry| {
                    acc.entry(entry.occurrence.occurrence.start.date())
                        .and_modify(|entries| entries.push(entry.clone()))
                        .or_insert(vec![entry]);
                    acc
                },
            )
    }
}

pub trait Actions<T> {
    type Id;
    type DeleteError;

    fn all(&self) -> HashMap<Self::Id, T>;
    fn create(&self, item: T) -> QueryResult<Self::Id>;
    fn read(&self, id: Self::Id) -> QueryResult<T>;
    fn update(&self, id: Self::Id, new_item: T) -> QueryResult<T>;
    fn delete(&self, id: Self::Id) -> Result<T, Self::DeleteError>;
}

use db::schema::locations::dsl::locations as schema;
impl Actions<Location> for Store {
    type Id = Id<Location>;
    type DeleteError = diesel::result::Error;

    fn all(&self) -> HashMap<Self::Id, Location> {
        schema
            .load::<SqlLocation>(&*self.0)
            .expect("Could not load database")
            .into_iter()
            .map(|x| x.into())
            .collect()
    }

    fn create(&self, item: Location) -> QueryResult<Self::Id> {
        let sql_item: SqlLocation = item.into();
        diesel::insert_into(schema)
            .values(&sql_item)
            .execute(&*self.0)?;

        Ok(sql_item.id.into())
    }

    fn read(&self, item_id: Self::Id) -> QueryResult<Location> {
        use db::SqlId;

        schema
            .find(SqlId::from(item_id))
            .first::<SqlLocation>(&*self.0)
            .map(|x| x.into())
            .map(|(_, x)| x)
    }

    fn update(&self, item_id: Self::Id, new_item: Location) -> QueryResult<Location> {
        use db::SqlId;

        let raw_id: SqlId<Location> = item_id.into();
        let (_, previous): (Id<Location>, Location) =
            schema.find(&raw_id).first::<SqlLocation>(&*self.0)?.into();

        diesel::update(schema.find(&raw_id))
            .set::<SqlLocation>(new_item.into())
            .execute(&*self.0)?;

        Ok(previous)
    }

    fn delete(&self, id: Self::Id) -> Result<Location, Self::DeleteError> {
        use db::SqlId;
        let raw_id: SqlId<Location> = id.into();
        let (_, previous): (Id<Location>, Location) =
            schema.find(&raw_id).first::<SqlLocation>(&*self.0)?.into();

        diesel::delete(schema.find(&raw_id)).execute(&*self.0)?;

        Ok(previous)
    }
}

#[derive(Debug)]
pub enum DeleteError<T> {
    DieselError(diesel::result::Error),
    Dependency(Vec<T>),
}

impl Actions<EventWithOccurrences> for Store {
    type Id = Id<Event>;
    type DeleteError = DeleteError<OccurrenceWithLocation>;

    fn all(&self) -> HashMap<Self::Id, EventWithOccurrences> {
        use db::schema::events::dsl::events;

        events
            .load::<SqlEvent>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_event| {
                use db::schema::occurrences::dsl::start;
                let occurrences: Vec<OccurrenceWithLocation> = SqlOccurrence::belonging_to(&sql_event)
                    .filter(start.gt(chrono::Local::now().naive_local()))
                    .load::<SqlOccurrence>(&*self.0)
                    .expect("Loading from database failed.")
                    .into_iter()
                    .map(|sql_occurrence| {
                        let (_, occurrence) = sql_occurrence.into();

                        occurrence
                    })
                    .collect();

                let (id, event) = sql_event.into();

                (id, EventWithOccurrences { event, occurrences })
            })
            .collect()
    }

    fn create(&self, item: EventWithOccurrences) -> QueryResult<Self::Id> {
        use db::schema::events::dsl::events;
        let sql_event: SqlEvent = item.event.into();
        diesel::insert_into(events)
            .values(&sql_event)
            .execute(&*self.0)?;

        use db::schema::occurrences::dsl::occurrences;
        let sql_occurrences: Vec<SqlOccurrence> = item
            .occurrences
            .into_iter()
            .map(|occurrence| (occurrence, sql_event.id.clone()).into())
            .collect();
        diesel::insert_into(occurrences)
            .values(&sql_occurrences)
            .execute(&*self.0)?;

        Ok(sql_event.id.into())
    }

    fn read(&self, item_id: Self::Id) -> QueryResult<EventWithOccurrences> {
        use db::schema::events::dsl::events;
        use db::SqlId;
        let sql_event = events
            .find(SqlId::from(item_id))
            .first::<SqlEvent>(&*self.0)?;

        use db::schema::occurrences::dsl::start;
        let occurrences: Vec<OccurrenceWithLocation> = SqlOccurrence::belonging_to(&sql_event)
            .filter(start.gt(chrono::Local::now().naive_local()))
            .load::<SqlOccurrence>(&*self.0)?
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        let (_, event) = sql_event.into();

        Ok(EventWithOccurrences { event, occurrences })
    }

    fn update(
        &self,
        item_id: Self::Id,
        new_item: EventWithOccurrences,
    ) -> QueryResult<EventWithOccurrences> {
        use db::SqlId;

        let raw_id: SqlId<Event> = item_id.into();
        use db::schema::events::dsl::events;
        let sql_previous = events.find(raw_id.clone()).first::<SqlEvent>(&*self.0)?;

        let associated_occurrences = SqlOccurrence::belonging_to(&sql_previous);
        let previous_occurrences: Vec<OccurrenceWithLocation> = associated_occurrences
            .load::<SqlOccurrence>(&*self.0)?
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        diesel::delete(associated_occurrences).execute(&*self.0)?;

        let new_sql_item: SqlEvent = new_item.event.into();
        diesel::update(&sql_previous)
            .set(new_sql_item)
            .execute(&*self.0)?;

        use db::schema::occurrences::dsl::occurrences as occurrences_table;
        let sql_occurrences: Vec<SqlOccurrence> = new_item
            .occurrences
            .into_iter()
            .map(|occurrence| (occurrence, raw_id.clone()).into())
            .collect();
        diesel::insert_into(occurrences_table)
            .values(&sql_occurrences)
            .execute(&*self.0)?;

        let (_, previous) = sql_previous.into();
        Ok(EventWithOccurrences {
            event: previous,
            occurrences: previous_occurrences,
        })
    }

    fn delete(&self, id: Self::Id) -> Result<EventWithOccurrences, Self::DeleteError> {
        use db::SqlId;

        let raw_id: SqlId<Event> = id.into();
        use db::schema::events::dsl::events;
        let sql_previous = events
            .find(raw_id)
            .first::<SqlEvent>(&*self.0)
            .map_err(DeleteError::DieselError)?;

        let occurrences: Vec<OccurrenceWithLocation> = SqlOccurrence::belonging_to(&sql_previous)
            .load::<SqlOccurrence>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        if occurrences.len() > 0 {
            return Err(DeleteError::Dependency(occurrences));
        }

        diesel::delete(&sql_previous)
            .execute(&*self.0)
            .map_err(DeleteError::DieselError)?;

        let (_, previous) = sql_previous.into();
        Ok(EventWithOccurrences {
            event: previous,
            occurrences,
        })
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
