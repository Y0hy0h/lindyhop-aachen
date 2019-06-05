use rocket_contrib::json::Json;
use rocket::Route;

use crate::error::{Result, Error};
use crate::store::{Store, Id, Event, Occurrence, Location, action::Actions};

macro_rules! derive_routes {
    ($mod: ident, $type: ident) => {
        pub mod $mod {
        use super::*;

        #[get("/")]
        fn all(store: Store) -> Json<Vec<$type>> {
            Json(store.all()
                .into_iter()
                .map(|x| x.1)
                .collect())
        }

        #[post("/", data = "<obj>")]
        fn store(store: Store, obj: Json<$type>) -> Result<String> {
            store.create(obj.0)
                .map_err(|x| Error::Database(x.to_string()))
                .map(|x| x.to_string())
        }

        #[post("/<id>", data = "<obj>")]
        fn update(store: Store, id: String, obj: Json<$type>) -> Result<Json<$type>> {
            Id::parse_str(&id)
                .map_err(|x| Error::ParseId(x.to_string()))
                .and_then(|x| store.update(x, obj.0).map_err(|x| Error::Database(x.to_string())))
                .map(|x| Json(x))
        }

        #[get("/<id>")]
        fn get(store: Store, id: String) -> Result<Json<$type>> {
            Id::parse_str(&id)
                .map_err(|x| Error::ParseId(x.to_string()))
                .and_then(|x| store.read(x).map_err(|x| Error::Database(x.to_string())))
                .map(|x| Json(x))
        }

        #[get("/<id>/delete")]
        fn delete(store: Store, id: String) -> Result<Json<$type>> {
            Id::parse_str(&id)
                .map_err(|x| Error::ParseId(x.to_string()))
                .and_then(|x| store.delete(x).map_err(|x| Error::Database(x.to_string())))
                .map(|x| Json(x))
        }

        pub fn routes() -> Vec<Route> {
            routes![all, store, update, get, delete]
        }
        }
    }
}

derive_routes!(event, Event);
derive_routes!(occurrence, Occurrence);
derive_routes!(location, Location);
