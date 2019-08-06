mod db;
mod model;

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fmt::Display;
use std::io::Cursor;
use std::marker::PhantomData;

use chrono::{NaiveDate, NaiveDateTime};
use rocket::http::RawStr;
use rocket::http::Status;
use rocket::request::{FormItem, FromParam, FromQuery, FromRequest, Outcome, Query, Request};
use rocket::response::{self, Responder, Response};
use rocket::{fairing, fairing::Fairing, Rocket};
use rocket_contrib::uuid::Uuid as RocketUuid;
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

impl<Item> Display for Id<Item> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

impl<'a, T> FromParam<'a> for Id<T> {
    type Error = <RocketUuid as FromParam<'a>>::Error;

    /// A value is successfully parsed if `param` is a properly formatted Uuid.
    /// Otherwise, a `ParseError` is returned.
    #[inline(always)]
    fn from_param(param: &'a RawStr) -> Result<Id<T>, Self::Error> {
        RocketUuid::from_param(param).map(|uuid| uuid.into_inner().into())
    }
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

    pub fn read_all(&self, filter: &OccurrenceFilter) -> Overview {
        let locs: HashMap<Id<Location>, Location> = self.all();
        let evts: HashMap<Id<Event>, EventWithOccurrences> =
            self.all_events_with_occurrences(filter);

        Overview {
            locations: locs,
            events: evts,
        }
    }

    pub fn occurrences_by_date(
        &self,
        filter: &OccurrenceFilter,
    ) -> BTreeMap<NaiveDate, Vec<OccurrenceWithEvent>> {
        use db::schema::events::dsl::events;
        use db::schema::occurrences::dsl::{occurrences, start};

        let sql_occurrences = occurrences
            .filter(apply_occurrence_filter(filter))
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
                let (event_id, event) = sql_event.into();
                OccurrenceWithEvent {
                    occurrence,
                    event_id,
                    event,
                }
            })
            .fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<NaiveDate, Vec<OccurrenceWithEvent>>, entry| {
                    acc.entry(entry.occurrence.occurrence.start.date())
                        .and_modify(|entries| entries.push(entry.clone()))
                        .or_insert_with(|| vec![entry]);
                    acc
                },
            )
    }

    pub fn locations_with_occurrences(
        &self,
        filter: &OccurrenceFilter,
    ) -> HashMap<Id<Location>, LocationWithOccurrences> {
        use db::schema::locations::dsl::locations;

        locations
            .load::<SqlLocation>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_location| {
                let occurrences: HashMap<Id<Occurrence>, Occurrence> =
                    SqlOccurrence::belonging_to(&sql_location)
                        .filter(apply_occurrence_filter(filter))
                        .load::<SqlOccurrence>(&*self.0)
                        .expect("Loading from database failed.")
                        .into_iter()
                        .map(|sql_occurrence| {
                            let (id, occurrence) = sql_occurrence.into();

                            (id, occurrence.occurrence)
                        })
                        .collect();

                let (id, location) = sql_location.into();

                (
                    id,
                    LocationWithOccurrences {
                        location,
                        occurrences,
                    },
                )
            })
            .collect()
    }
}

pub trait Actions<T> {
    type Id;

    fn all(&self) -> HashMap<Self::Id, T>;
    fn create(&self, item: T) -> QueryResult<Self::Id>;
    fn read(&self, id: Self::Id) -> QueryResult<T>;
    fn update(&self, id: Self::Id, new_item: T) -> QueryResult<T>;
    fn delete(&self, id: Self::Id) -> QueryResult<T>;
}

use db::schema::locations::dsl::locations as schema;
impl Actions<Location> for Store {
    type Id = Id<Location>;

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

    fn delete(&self, id: Self::Id) -> QueryResult<Location> {
        use db::SqlId;
        let raw_id: SqlId<Location> = id.into();
        let (_, previous): (Id<Location>, Location) =
            schema.find(&raw_id).first::<SqlLocation>(&*self.0)?.into();

        diesel::delete(schema.find(&raw_id)).execute(&*self.0)?;

        Ok(previous)
    }
}

#[derive(Debug)]
pub struct OccurrenceFilter {
    pub before: Option<NaiveDateTime>,
    pub after: Option<NaiveDateTime>,
}

impl Default for OccurrenceFilter {
    fn default() -> Self {
        OccurrenceFilter {
            before: None,
            after: None,
        }
    }
}

impl OccurrenceFilter {
    pub fn upcoming() -> Self {
        let today = NaiveDateTime::new(
            chrono::Local::today().naive_local(),
            chrono::NaiveTime::from_hms(0, 0, 0),
        );
        OccurrenceFilter {
            before: None,
            after: Some(today),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum OccurrenceFilterError {
    InvalidBeforeDate,
    InvalidAfterDate,
    InvalidRange,
}

impl<'r> Responder<'r> for OccurrenceFilterError {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        Response::build()
            .sized_body(Cursor::new(serde_json::to_string(&self).unwrap()))
            .status(Status::UnprocessableEntity)
            .ok()
    }
}

impl<'q> FromQuery<'q> for OccurrenceFilter {
    type Error = OccurrenceFilterError;

    fn from_query(mut query: Query<'q>) -> Result<Self, Self::Error> {
        use OccurrenceFilterError::*;
        let before: Option<NaiveDateTime> = query
            .clone()
            .find(|i| i.key == "before")
            .map(|item| decode_datetime(item).ok_or(InvalidBeforeDate))
            .transpose()?;
        let after: Option<NaiveDateTime> = query
            .find(|i| i.key == "after")
            .map(|item| decode_datetime(item).ok_or(InvalidAfterDate))
            .transpose()?;

        if after < before {
            return Err(InvalidRange)?;
        }

        Ok(OccurrenceFilter { before, after })
    }
}

fn decode_datetime(item: FormItem) -> Option<NaiveDateTime> {
    chrono::NaiveDateTime::parse_from_str(&item.value.url_decode_lossy(), "%Y-%m-%dT%H:%M:%S").ok()
}

impl Store {
    pub fn all_events_with_occurrences(
        &self,
        filter: &OccurrenceFilter,
    ) -> HashMap<Id<Event>, EventWithOccurrences> {
        use db::schema::events::dsl::events;

        events
            .load::<SqlEvent>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_event| {
                let occurrences: Vec<OccurrenceWithLocation> =
                    SqlOccurrence::belonging_to(&sql_event)
                        .filter(apply_occurrence_filter(filter))
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

    pub fn create_event_with_occurrences(
        &self,
        item: EventWithOccurrences,
    ) -> QueryResult<Id<Event>> {
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

    pub fn read_event_with_occurrences(
        &self,
        item_id: Id<Event>,
        filter: &OccurrenceFilter,
    ) -> QueryResult<EventWithOccurrences> {
        use db::schema::events::dsl::events;
        use db::SqlId;
        let sql_event = events
            .find(SqlId::from(item_id))
            .first::<SqlEvent>(&*self.0)?;

        let occurrences: Vec<OccurrenceWithLocation> = SqlOccurrence::belonging_to(&sql_event)
            .filter(apply_occurrence_filter(filter))
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

    pub fn update_event_with_occurrences(
        &self,
        item_id: Id<Event>,
        new_item: EventWithOccurrences,
        filter: &OccurrenceFilter,
    ) -> QueryResult<EventWithOccurrences> {
        use db::SqlId;

        let raw_id: SqlId<Event> = item_id.into();
        use db::schema::events::dsl::events;
        let sql_previous = events.find(raw_id.clone()).first::<SqlEvent>(&*self.0)?;

        let associated_occurrences = SqlOccurrence::belonging_to(&sql_previous);
        let previous_occurrences: Vec<OccurrenceWithLocation> = associated_occurrences
            .filter(apply_occurrence_filter(filter))
            .load::<SqlOccurrence>(&*self.0)?
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        diesel::delete(associated_occurrences.filter(apply_occurrence_filter(&filter)))
            .execute(&*self.0)?;

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

    pub fn delete_event_with_occurrences(
        &self,
        id: Id<Event>,
    ) -> QueryResult<EventWithOccurrences> {
        use db::SqlId;

        let raw_id: SqlId<Event> = id.into();
        use db::schema::events::dsl::events;
        let sql_previous = events.find(raw_id).first::<SqlEvent>(&*self.0)?;

        let occurrences: Vec<OccurrenceWithLocation> = SqlOccurrence::belonging_to(&sql_previous)
            .load::<SqlOccurrence>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        diesel::delete(SqlOccurrence::belonging_to(&sql_previous)).execute(&*self.0)?;

        diesel::delete(&sql_previous).execute(&*self.0)?;

        let (_, previous) = sql_previous.into();
        Ok(EventWithOccurrences {
            event: previous,
            occurrences,
        })
    }
}

fn apply_occurrence_filter(
    filter: &OccurrenceFilter,
) -> Box<
    dyn BoxableExpression<
        db::schema::occurrences::table,
        diesel::sqlite::Sqlite,
        SqlType = diesel::sql_types::Bool,
    >,
> {
    use db::schema::occurrences::dsl::*;
    let mut query: Box<
        dyn BoxableExpression<
            db::schema::occurrences::table,
            diesel::sqlite::Sqlite,
            SqlType = diesel::sql_types::Bool,
        >,
    > = Box::new(true.into_sql::<diesel::sql_types::Bool>());
    if let Some(before) = filter.before {
        query = Box::new(query.and(start.lt(before)))
    }
    if let Some(after) = filter.after {
        query = Box::new(query.and(start.gt(after)))
    }

    query
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
            .and_then(db::initialize)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <db::Connection as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        db::Connection::from_request(request).map(Store)
    }
}
