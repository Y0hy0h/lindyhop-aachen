#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod id_map;
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
use chrono::prelude::*;

use store::Store;

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

#[get("/admin/<path..>")]
#[allow(unused_variables)]
fn admin_subroute(path: PathBuf) -> Option<NamedFile> {
    admin()
}

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
        .mount("/", routes![index, admin_route, admin_subroute,])
        .launch();
}
