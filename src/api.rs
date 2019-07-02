use std::collections::HashMap;

use rocket::Rocket;
use rocket_contrib::json::Json;

use crate::store::{Id, Location, LocationWithOccurrences, Overview, OccurrenceFilter, OccurrenceFilterError, Store};

macro_rules! derive_routes {
    ($mod: ident, $type: ident) => {
        pub mod $mod {
            use std::collections::HashMap;
            use std::iter::FromIterator;

            use crate::store::Actions;
            #[allow(unused_imports)]
            use crate::store::{Event, EventWithOccurrences, Id, Location, Occurrence, Store};

            use rocket::Route;
            use rocket_contrib::json::Json;

            type Result<T> = std::result::Result<T, String>;

            #[get("/")]
            fn all(store: Store) -> Json<HashMap<Id<$type>, $type>> {
                Json(HashMap::from_iter(store.all()))
            }

            #[post("/", data = "<obj>")]
            fn create(store: Store, obj: Json<$type>) -> Result<Json<Id<$type>>> {
                store.create(obj.0).map_err(|err| err.to_string()).map(Json)
            }

            #[get("/<id>")]
            fn read(store: Store, id: Id<$type>) -> Result<Json<$type>> {
                store
                    .read(id.into())
                    .map_err(|err| err.to_string())
                    .map(Json)
            }

            #[put("/<id>", data = "<obj>")]
            pub fn update(store: Store, id: Id<$type>, obj: Json<$type>) -> Result<Json<$type>> {
                store
                    .update(id.into(), obj.0)
                    .map_err(|err| err.to_string())
                    .map(Json)
            }

            #[delete("/<id>")]
            fn delete(store: Store, id: Id<$type>) -> Result<Json<$type>> {
                store
                    .delete(id.into())
                    .map_err(|err| format!("{:?}", err))
                    .map(Json)
            }

            pub fn routes() -> Vec<Route> {
                routes![all, create, read, update, delete]
            }
        }
    };
}

derive_routes!(locations, Location);

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
        filter: Result<OccurrenceFilter, OccurrenceFilterError>,
    ) -> Result<Json<HashMap<Id<Event>, EventWithOccurrences>>, OccurrenceFilterError> {
        let mut filter = filter?;
        if let (None, None) = (filter.before, filter.after) {
            filter = OccurrenceFilter::upcoming();
        }
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
        filter: Result<OccurrenceFilter, OccurrenceFilterError>,
    ) -> Result<Json<EventWithOccurrences>, OccurrenceFilterError> {
        let mut filter = filter?;
        if let (None, None) = (filter.before, filter.after) {
            filter = OccurrenceFilter::upcoming();
        }
        Ok(Json(
            store
                .read_event_with_occurrences(id.into(), &filter)
                .unwrap(),
        ))
    }

    #[put("/<id>?<filter..>", data = "<obj>")]
    fn update(
        store: Store,
        id: Id<Event>,
        obj: Json<EventWithOccurrences>,
        filter: Result<OccurrenceFilter, OccurrenceFilterError>,
    ) -> Result<Json<EventWithOccurrences>, OccurrenceFilterError> {
        let mut filter = filter?;
        if let (None, None) = (filter.before, filter.after) {
            filter = OccurrenceFilter::upcoming();
        }

        Ok(Json(
            store
                .update_event_with_occurrences(id, obj.0, &filter)
                .unwrap(),
        ))
    }

    #[delete("/<id>")]
    fn delete(store: Store, id: Id<Event>) -> Result<Json<EventWithOccurrences>, String> {
        store
            .delete_event_with_occurrences(id.into())
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
fn api_overview(store: Store, filter: Result<OccurrenceFilter, OccurrenceFilterError>) -> Result<Json<Overview>, OccurrenceFilterError> {
    let mut filter = filter?;
    if let (None, None) = (filter.before, filter.after) {
        filter = OccurrenceFilter::upcoming();
    }

    Ok(Json(store.read_all(&filter)))
}

#[get("/locations_with_occurrences?<filter..>")]
fn api_locations_with_occurrences(
    store: Store,
    filter: Result<OccurrenceFilter, OccurrenceFilterError>
) -> Result<Json<HashMap<Id<Location>, LocationWithOccurrences>>, OccurrenceFilterError> {
    let mut filter = filter?;
    if let (None, None) = (filter.before, filter.after) {
        filter = OccurrenceFilter::upcoming();
    }

    Ok(Json(store.locations_with_occurrences(&filter)))
}
