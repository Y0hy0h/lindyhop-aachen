mod db;

use chrono::NaiveDateTime;
use diesel::prelude::*;
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

    fn occurrences(&self, context: &Store) -> Vec<Occurrence> {
        use db::schema::occurrences::dsl::*;
        occurrences
            .filter(event_id.eq(self.id))
            .load(&*context.0)
            .expect("Error loading from database.")
    }
}

#[juniper::object(Context=Store)]
impl Occurrence {
    fn id(&self) -> Id {
        self.id
    }

    fn event(&self, context: &Store) -> Event {
        use db::schema::events::dsl::*;
        events
            .find(self.event_id)
            .first(&*context.0)
            .expect("Error loading from database.")
    }

    fn start(&self) -> &NaiveDateTime {
        &self.start
    }
    fn duration(&self) -> i32 {
        self.duration
    }

    fn location(&self, context: &Store) -> Location {
        use db::schema::locations::dsl::*;
        locations
            .find(self.location_id)
            .first(&*context.0)
            .expect("Error loading from database.")
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

    fn occurrences(&self, context: &Store) -> Vec<Occurrence> {
        use db::schema::occurrences::dsl::*;
        occurrences
            .filter(location_id.eq(self.id))
            .load(&*context.0)
            .expect("Error loading from database.")
    }
}

pub struct Query;

#[juniper::object(Context=Store)]
impl Query {
    fn all_events(context: &Store) -> Vec<Event> {
        use db::schema::events::dsl::*;
        events
            .load(&*context.0)
            .expect("Error loading from database.")
    }

    fn all_locations(context: &Store) -> Vec<Location> {
        use db::schema::locations::dsl::*;
        locations
            .load(&*context.0)
            .expect("Error loading from database.")
    }
}

pub struct Mutation;

#[juniper::object(Context=Store)]
impl Mutation {
    fn add_event(context: &Store, new_event: NewEvent) -> Id {
        use db::schema::events::dsl::*;
        context
            .0
            .transaction(|| {
                diesel::insert_into(events)
                    .values(&new_event)
                    .execute(&*context.0)?;
                events.select(id).order(id.desc()).first(&*context.0)
            })
            .expect("Error inserting into database.")
    }

    fn update_event(context: &Store, id_to_update: Id, new_event: UpdateEvent) -> Event {
        use db::schema::events::dsl::*;
        diesel::update(events.find(id_to_update))
            .set(new_event)
            .execute(&*context.0)
            .expect("Error updating in database.");
        events
            .find(id_to_update)
            .first(&*context.0)
            .expect("Error fetching from database.")
    }

    fn remove_event(context: &Store, id_to_remove: Id) -> Event {
        use db::schema::events::dsl::*;
        let item = events
            .find(id_to_remove)
            .first(&*context.0)
            .expect("Error fetching from database.");
        diesel::delete(&item)
            .execute(&*context.0)
            .expect("Error deleting from database.");
        item
    }

    fn add_occurrence(context: &Store, new_occurrence: NewOccurrence) -> Id {
        use db::schema::occurrences::dsl::*;
        context
            .0
            .transaction(|| {
                diesel::insert_into(occurrences)
                    .values(&new_occurrence)
                    .execute(&*context.0)?;
                occurrences.select(id).order(id.desc()).first(&*context.0)
            })
            .expect("Error inserting into database.")
    }

    fn update_occurrence(
        context: &Store,
        id_to_update: Id,
        new_occurrence: UpdateOccurrence,
    ) -> Occurrence {
        use db::schema::occurrences::dsl::*;
        diesel::update(occurrences.find(id_to_update))
            .set(new_occurrence)
            .execute(&*context.0)
            .expect("Error updating in database.");
        occurrences
            .find(id_to_update)
            .first(&*context.0)
            .expect("Error fetching from database.")
    }

    fn remove_occurrence(context: &Store, id_to_remove: Id) -> Occurrence {
        use db::schema::occurrences::dsl::*;
        let item = occurrences
            .find(id_to_remove)
            .first(&*context.0)
            .expect("Error fetching from database.");
        diesel::delete(&item)
            .execute(&*context.0)
            .expect("Error deleting from database.");
        item
    }

    fn add_location(context: &Store, new_location: NewLocation) -> Id {
        use db::schema::locations::dsl::*;
        context
            .0
            .transaction(|| {
                diesel::insert_into(locations)
                    .values(&new_location)
                    .execute(&*context.0)?;
                locations.select(id).order(id.desc()).first(&*context.0)
            })
            .expect("Error inserting into database.")
    }

    fn update_location(
        context: &Store,
        id_to_update: Id,
        new_location: UpdateLocation,
    ) -> Location {
        use db::schema::locations::dsl::*;
        let item = locations
            .find(id_to_update)
            .first(&*context.0)
            .expect("Error fetching from database.");
        diesel::update(&item)
            .set(new_location)
            .execute(&*context.0)
            .expect("Error updating in database.");
        item
    }

    fn remove_location(context: &Store, id_to_remove: Id) -> Location {
        use db::schema::locations::dsl::*;
        let item = locations
            .find(id_to_remove)
            .first(&*context.0)
            .expect("Error fetching from database.");
        diesel::delete(&item)
            .execute(&*context.0)
            .expect("Error deleting from database.");
        item
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
