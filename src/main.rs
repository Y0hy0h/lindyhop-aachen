#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod store;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use maud::{html, Markup, DOCTYPE};
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use store::{EventWithOccurrences, Overview, Store};

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
                @let Overview {locations, events} = store.read_all();
                ul {
                    @for EventWithOccurrences {event, occurrences} in events.values() {
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
                    @for location in locations.values() {
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

// We also want to serve the file when subroutes are called, e. g. `/admin/event/42`.
// Removing this would break reloading the admin on subroutes.
#[get("/admin/<path..>")]
#[allow(unused_variables)]
fn admin_subroute(path: PathBuf) -> Option<NamedFile> {
    admin()
}

fn admin() -> Option<NamedFile> {
    NamedFile::open(Path::new("admin/dist/index.html")).ok()
}

#[get("/")]
fn api_overview(store: Store) -> Json<Overview> {
    Json(store.read_all())
}

fn main() {
    use store::routes::*;

    rocket::ignite()
        .attach(Store::fairing())
        .mount(
            "/static",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")),
        )
        .mount("/", routes![index, admin_route, admin_subroute])
        .mount("/api", routes![api_overview])
        .mount("/api/events/", event_with_occurrences::routes())
        .mount("/api/locations/", location::routes())
        .launch();
}
