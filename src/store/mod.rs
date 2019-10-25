mod db;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use juniper::FieldResult;
use rocket::fairing::{self, Fairing};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::Rocket;

use db::*;

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn new_schema_instance() -> Schema {
    Schema::new(Query, Mutation)
}

#[juniper::object(Context=Store)]
impl Event {
    fn id(&self) -> Id {
        self.id
    }
    fn title(&self) -> &String {
        &self.title
    }

    fn teaser(&self) -> &String {
        &self.teaser
    }

    fn description(&self) -> &String {
        &self.description
    }

    fn occurrences(
        &self,
        context: &Store,
        filter: Option<OccurrenceFilter>,
    ) -> FieldResult<Vec<Occurrence>> {
        use db::schema::occurrences::dsl::*;
        let result = occurrences
            .filter(event_id.eq(self.id))
            .filter(filter.unwrap_or_default().to_sql_clause())
            .load(&*context.0)?;
        Ok(result)
    }
}

#[juniper::object(Context=Store)]
impl Occurrence {
    fn id(&self) -> Id {
        self.id
    }

    fn event(&self, context: &Store) -> FieldResult<Event> {
        use db::schema::events::dsl::*;
        let result = events.find(self.event_id).first(&*context.0)?;
        Ok(result)
    }

    fn start(&self) -> &NaiveDateTime {
        &self.start
    }
    fn duration(&self) -> i32 {
        self.duration
    }

    fn location(&self, context: &Store) -> FieldResult<Location> {
        use db::schema::locations::dsl::*;
        let result = locations.find(self.location_id).first(&*context.0)?;
        Ok(result)
    }
}

#[juniper::object(Context=Store)]
impl Location {
    fn id(&self) -> Id {
        self.id
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn address(&self) -> &String {
        &self.address
    }

    fn occurrences(
        &self,
        context: &Store,
        filter: Option<OccurrenceFilter>,
    ) -> FieldResult<Vec<Occurrence>> {
        use db::schema::occurrences::dsl::*;
        let result = occurrences
            .filter(location_id.eq(self.id))
            .filter(filter.unwrap_or_default().to_sql_clause())
            .load(&*context.0)?;
        Ok(result)
    }
}

#[derive(juniper::GraphQLInputObject, Default)]
pub struct OccurrenceFilter {
    after: Option<NaiveDateTime>,
    before: Option<NaiveDateTime>,
}

impl OccurrenceFilter {
    fn to_sql_clause(
        &self,
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
        if let Some(before) = self.before {
            query = Box::new(query.and(start.lt(before)))
        }
        if let Some(after) = self.after {
            query = Box::new(query.and(start.gt(after)))
        }

        query
    }
}

pub struct Query;

#[juniper::object(Context=Store)]
impl Query {
    fn all_events(context: &Store) -> FieldResult<Vec<Event>> {
        use db::schema::events::dsl::*;
        let result = events.load(&*context.0)?;
        Ok(result)
    }

    fn event(context: &Store, id: Id) -> FieldResult<Event> {
        use db::schema::events::dsl as table;
        let result = table::events.find(id).first(&*context.0)?;
        Ok(result)
    }

    fn all_locations(context: &Store) -> FieldResult<Vec<Location>> {
        use db::schema::locations::dsl::*;
        let result = locations.load(&*context.0)?;
        Ok(result)
    }

    fn location(context: &Store, id: Id) -> FieldResult<Location> {
        use db::schema::locations::dsl as table;
        let result = table::locations.find(id).first(&*context.0)?;
        Ok(result)
    }
}

pub struct Mutation;

#[juniper::object(Context=Store)]
impl Mutation {
    fn add_event(context: &Store, new_event: NewEvent) -> FieldResult<Id> {
        use db::schema::events::dsl::*;
        let result = context.0.transaction(|| {
            diesel::insert_into(events)
                .values(&new_event)
                .execute(&*context.0)?;
            events.select(id).order(id.desc()).first(&*context.0)
        })?;
        Ok(result)
    }

    fn update_event(
        context: &Store,
        id_to_update: Id,
        new_event: UpdateEvent,
    ) -> FieldResult<Event> {
        use db::schema::events::dsl::*;
        diesel::update(events.find(id_to_update))
            .set(new_event)
            .execute(&*context.0)?;
        let result = events.find(id_to_update).first(&*context.0)?;
        Ok(result)
    }

    fn remove_event(context: &Store, id_to_remove: Id) -> FieldResult<Event> {
        use db::schema::events::dsl::*;
        let item = events.find(id_to_remove).first(&*context.0)?;
        diesel::delete(&item).execute(&*context.0)?;
        Ok(item)
    }

    fn add_occurrence(context: &Store, new_occurrence: NewOccurrence) -> FieldResult<Id> {
        use db::schema::occurrences::dsl::*;
        let result = context.0.transaction(|| {
            diesel::insert_into(occurrences)
                .values(&new_occurrence)
                .execute(&*context.0)?;
            occurrences.select(id).order(id.desc()).first(&*context.0)
        })?;
        Ok(result)
    }

    fn update_occurrence(
        context: &Store,
        id_to_update: Id,
        new_occurrence: UpdateOccurrence,
    ) -> FieldResult<Occurrence> {
        use db::schema::occurrences::dsl::*;
        diesel::update(occurrences.find(id_to_update))
            .set(new_occurrence)
            .execute(&*context.0)?;
        let result = occurrences.find(id_to_update).first(&*context.0)?;
        Ok(result)
    }

    fn remove_occurrence(context: &Store, id_to_remove: Id) -> FieldResult<Occurrence> {
        use db::schema::occurrences::dsl::*;
        let item = occurrences.find(id_to_remove).first(&*context.0)?;
        diesel::delete(&item).execute(&*context.0)?;
        Ok(item)
    }

    fn add_location(context: &Store, new_location: NewLocation) -> FieldResult<Id> {
        use db::schema::locations::dsl::*;
        let result = context.0.transaction(|| {
            diesel::insert_into(locations)
                .values(&new_location)
                .execute(&*context.0)?;
            locations.select(id).order(id.desc()).first(&*context.0)
        })?;
        Ok(result)
    }

    fn update_location(
        context: &Store,
        id_to_update: Id,
        new_location: UpdateLocation,
    ) -> FieldResult<Location> {
        use db::schema::locations::dsl::*;
        let item = locations.find(id_to_update).first(&*context.0)?;
        diesel::update(&item)
            .set(new_location)
            .execute(&*context.0)?;

        Ok(item)
    }

    fn remove_location(context: &Store, id_to_remove: Id) -> FieldResult<Location> {
        use db::schema::locations::dsl::*;
        let item = locations.find(id_to_remove).first(&*context.0)?;
        diesel::delete(&item).execute(&*context.0)?;
        Ok(item)
    }

    fn replace_occurrences(
        context: &Store,
        event_id: Id,
        filter: Option<OccurrenceFilter>,
        new_occurrences: Vec<NewOccurrence>,
    ) -> FieldResult<Event> {
        use db::schema::occurrences::dsl as table;
        let current_occurrences = table::occurrences
            .filter(table::event_id.eq(event_id))
            .filter(filter.unwrap_or_default().to_sql_clause());
        diesel::delete(current_occurrences).execute(&*context.0)?;
        diesel::insert_into(table::occurrences)
            .values(new_occurrences)
            .execute(&*context.0)?;
        use db::schema::events::dsl as events_table;
        let result = events_table::events.find(event_id).first(&*context.0)?;
        Ok(result)
    }
}

pub struct Store(db::Connection);

impl juniper::Context for Store {}

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
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
            .and_then(db::initialize)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <db::Connection as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        db::Connection::from_request(request).map(Store)
    }
}
