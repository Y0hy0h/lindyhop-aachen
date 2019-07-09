use std::collections::HashMap;

use rocket::Rocket;
use rocket_contrib::json::Json;

use crate::store::{
    Id, Location, LocationWithOccurrences, OccurrenceFilter, OccurrenceFilterError, Overview, Store,
};

pub mod locations {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use crate::store::Actions;
    use crate::store::{Id, Location, Store};

    use rocket::Route;
    use rocket_contrib::json::Json;

    type Result<T> = std::result::Result<T, String>;

    #[get("/")]
    fn all(store: Store) -> Json<HashMap<Id<Location>, Location>> {
        Json(HashMap::from_iter(store.all()))
    }

    #[post("/", data = "<obj>")]
    fn create(store: Store, obj: Json<Location>) -> Result<Json<Id<Location>>> {
        store.create(obj.0).map_err(|err| err.to_string()).map(Json)
    }

    #[get("/<id>")]
    fn read(store: Store, id: Id<Location>) -> Result<Json<Location>> {
        store.read(id).map_err(|err| err.to_string()).map(Json)
    }

    #[put("/<id>", data = "<obj>")]
    pub fn update(store: Store, id: Id<Location>, obj: Json<Location>) -> Result<Json<Location>> {
        store
            .update(id, obj.0)
            .map_err(|err| err.to_string())
            .map(Json)
    }

    #[delete("/<id>")]
    fn delete(store: Store, id: Id<Location>) -> Result<Json<Location>> {
        store
            .delete(id)
            .map_err(|err| format!("{:?}", err))
            .map(Json)
    }

    pub fn routes() -> Vec<Route> {
        routes![all, create, read, update, delete]
    }
}

mod events {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use crate::store::{
        Event, EventWithOccurrences, Id, OccurrenceFilter, OccurrenceFilterError, Store,
    };

    use rocket::Route;
    use rocket_contrib::json::Json;

    #[get("/?<filter..>")]
    fn all(
        store: Store,
        filter: OccurrenceFilter,
    ) -> Result<Json<HashMap<Id<Event>, EventWithOccurrences>>, OccurrenceFilterError> {
        Ok(Json(HashMap::from_iter(
            store.all_events_with_occurrences(&filter),
        )))
    }

    #[post("/", data = "<obj>")]
    fn create(store: Store, obj: Json<EventWithOccurrences>) -> Result<Json<Id<Event>>, String> {
        store
            .create_event_with_occurrences(obj.0)
            .map_err(|err| err.to_string())
            .map(Json)
    }

    #[get("/<id>?<filter..>")]
    fn read(
        store: Store,
        id: Id<Event>,
        filter: OccurrenceFilter,
    ) -> Result<Json<EventWithOccurrences>, OccurrenceFilterError> {
        Ok(Json(
            store.read_event_with_occurrences(id, &filter).unwrap(),
        ))
    }

    #[put("/<id>?<filter..>", data = "<obj>")]
    fn update(
        store: Store,
        id: Id<Event>,
        obj: Json<EventWithOccurrences>,
        filter: OccurrenceFilter,
    ) -> Result<Json<EventWithOccurrences>, OccurrenceFilterError> {
        Ok(Json(
            store
                .update_event_with_occurrences(id, obj.0, &filter)
                .unwrap(),
        ))
    }

    #[delete("/<id>")]
    fn delete(store: Store, id: Id<Event>) -> Result<Json<EventWithOccurrences>, String> {
        store
            .delete_event_with_occurrences(id)
            .map_err(|err| format!("{:?}", err))
            .map(Json)
    }
    pub fn routes() -> Vec<Route> {
        routes![all, create, read, update, delete]
    }
}

pub fn mount(rocket: Rocket, prefix: &'static str) -> Rocket {
    rocket
        .mount(
            prefix,
            routes![api_overview, api_locations_with_occurrences],
        )
        .mount(&format!("{}/locations", prefix), locations::routes())
        .mount(&format!("{}/events", prefix), events::routes())
}

#[get("/?<filter..>")]
fn api_overview(
    store: Store,
    filter: OccurrenceFilter,
) -> Result<Json<Overview>, OccurrenceFilterError> {
    Ok(Json(store.read_all(&filter)))
}

#[get("/locations_with_occurrences?<filter..>")]
fn api_locations_with_occurrences(
    store: Store,
    filter: OccurrenceFilter,
) -> Result<Json<HashMap<Id<Location>, LocationWithOccurrences>>, OccurrenceFilterError> {
    Ok(Json(store.locations_with_occurrences(&filter)))
}
