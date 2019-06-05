#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod id_map;
mod error;
mod store;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::path::{Path, PathBuf};

use maud::{html, Markup, DOCTYPE};
use rocket::response::NamedFile;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::Json;
use rocket_contrib::json::JsonValue;
use chrono::prelude::*;

use store::{Store, Id, Event, action::Actions};
use error::*;

#[get("/")]
fn index(store: Store) -> Markup {
    html! {
        ( DOCTYPE )
        html {
            head {
                link href="static/main.css" rel="stylesheet";
            }
            body {
                h1 { "Lindy Hop Aachen" }
                @let (locations, events) = store.read_all();
                ul {
                    @for (_, event, occurrences) in events {
                        li {
                            (event.name)
                            ul {
                                @for occurrence in occurrences {
                                    li { (format_date(&occurrence.start.date())) }
                                }
                            }
                        }
                    }
                }
                ul {
                    @for (_, location) in locations {
                        li { ( location.name ) }
                    }
                }
            }
        }
    }
}

fn format_date(date: &NaiveDate) -> String {
    use chrono::Weekday::*;

    let day = match date.weekday() {
        Mon => "Mo",
        Tue => "Di",
        Wed => "Mi",
        Thu => "Do",
        Fri => "Fr",
        Sat => "Sa",
        Sun => "So",
    };
    let format = format!("{}, %d.%m.", day);

    date.format(&format).to_string()
}

#[get("/admin")]
fn admin_route() -> Option<NamedFile> {
    admin()
}

#[post("/admin/event", data = "<obj>")]
fn store_event(store: Store, obj: Json<Event>) -> Result<String> {
    store.create(obj.0)
        .map_err(|x| Error::Database(x.to_string()))
        .map(|x| x.to_string())
}

#[post("/admin/event/<id>", data = "<obj>")]
fn update_event(store: Store, id: String, obj: Json<Event>) -> Result<Json<Event>> {
    Id::parse_str(&id)
        .map_err(|x| Error::ParseId(x.to_string()))
        .and_then(|x| store.update(x, obj.0).map_err(|x| Error::Database(x.to_string())))
        .map(|x| Json(x))
}

#[get("/admin/event/<id>")]
fn get_event(store: Store, id: String) -> Result<Json<Event>> {
    Id::parse_str(&id)
        .map_err(|x| Error::ParseId(x.to_string()))
        .and_then(|x| store.read(x).map_err(|x| Error::Database(x.to_string())))
        .map(|x| Json(x))
}

#[get("/admin/event/<id>/delete")]
fn delete_event(store: Store, id: String) -> Result<Json<Event>> {
    Id::parse_str(&id)
        .map_err(|x| Error::ParseId(x.to_string()))
        .and_then(|x| store.delete(x).map_err(|x| Error::Database(x.to_string())))
        .map(|x| Json(x))
}

/*#[get("/admin/<path..>")]
#[allow(unused_variables)]
fn admin_subroute(path: PathBuf) -> Option<NamedFile> {
    admin()
}*/

fn admin() -> Option<NamedFile> {
    NamedFile::open(Path::new("admin/dist/index.html")).ok()
}

fn main() {
    rocket::ignite()
        .attach(Store::fairing())
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .mount("/", routes![index, admin_route, get_event, store_event, update_event, delete_event])
        .launch();
}
