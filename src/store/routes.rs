macro_rules! derive_routes {
    ($mod: ident, $type: ident) => {
        pub mod $mod {
            use std::collections::HashMap;
            use std::iter::FromIterator;

            use crate::store::action::Actions;
            use crate::store::{Event, EventWithOccurrences, Id, Location, Occurrence, Store};

            use rocket::Route;
            use rocket_contrib::{json::Json, uuid::Uuid};

            type Result<T> = std::result::Result<T, String>;

            #[get("/")]
            fn all(store: Store) -> Json<HashMap<Id, $type>> {
                Json(HashMap::from_iter(store.all()))
            }

            #[post("/", data = "<obj>")]
            fn create(store: Store, obj: Json<$type>) -> Result<Json<Id>> {
                store.create(obj.0).map_err(|err| err.to_string()).map(Json)
            }

            #[get("/<id>")]
            fn read(store: Store, id: Uuid) -> Result<Json<$type>> {
                store
                    .read(id.into_inner())
                    .map_err(|err| err.to_string())
                    .map(Json)
            }

            #[put("/<id>", data = "<obj>")]
            fn update(store: Store, id: Uuid, obj: Json<$type>) -> Result<Json<$type>> {
                store
                    .update(id.into_inner(), obj.0)
                    .map_err(|err| err.to_string())
                    .map(Json)
            }

            #[delete("/<id>")]
            fn delete(store: Store, id: Uuid) -> Result<Json<$type>> {
                store
                    .delete(id.into_inner())
                    .map_err(|err| format!("{:?}", err))
                    .map(Json)
            }

            pub fn routes() -> Vec<Route> {
                routes![all, create, read, update, delete]
            }
        }
    };
}

derive_routes!(event, Event);
derive_routes!(occurrence, Occurrence);
derive_routes!(location, Location);
derive_routes!(event_with_occurrences, EventWithOccurrences);
