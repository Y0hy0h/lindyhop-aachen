use rocket::Route;
use rocket_contrib::{json::Json, uuid::Uuid};

use crate::store::{action::Actions, Event, Location, Occurrence, Store};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Responder, Debug)]
pub enum Error {
    ParseId(String),
    Database(String),
}

macro_rules! derive_routes {
    ($mod: ident, $type: ident) => {
        pub mod $mod {
            use super::*;

            #[get("/")]
            fn all(store: Store) -> Json<Vec<$type>> {
                Json(store.all().into_iter().map(|x| x.1).collect())
            }

            #[post("/", data = "<obj>")]
            fn store(store: Store, obj: Json<$type>) -> Result<String> {
                store
                    .create(obj.0)
                    .map_err(|x| Error::Database(x.to_string()))
                    .map(|x| x.to_string())
            }

            #[post("/<id>", data = "<obj>")]
            fn update(store: Store, id: Uuid, obj: Json<$type>) -> Result<Json<$type>> {
                store
                    .update(id.into_inner(), obj.0)
                    .map_err(|x| Error::Database(x.to_string()))
                    .map(|x| Json(x))
            }

            #[get("/<id>")]
            fn get(store: Store, id: Uuid) -> Result<Json<$type>> {
                store.read(id.into_inner())
                    .map_err(|x| Error::Database(x.to_string()))
                    .map(|x| Json(x))
            }

            #[get("/<id>/delete")]
            fn delete(store: Store, id: Uuid) -> Result<Json<$type>> {
                store.delete(id.into_inner())
                    .map_err(|x| Error::Database(x.to_string()))
                    .map(|x| Json(x))
            }

            pub fn routes() -> Vec<Route> {
                routes![all, store, update, get, delete]
            }
        }
    };
}

derive_routes!(event, Event);
derive_routes!(occurrence, Occurrence);
derive_routes!(location, Location);
