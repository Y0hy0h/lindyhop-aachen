#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod id_map;
mod store;

use std::path::{Path, PathBuf};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
use maud::{html, Markup, DOCTYPE};
use rocket::response::NamedFile;
use rocket_contrib::serve::StaticFiles;

use store::{Store};

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
                ul {
                    @for location in store.read_all() {
                        li { ( location.name ) }
                    }
                }
            }
        }
    }
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
        .mount(
            "/",
            routes![
                index,
                admin_route,
                admin_subroute,
            ],
        )
        .launch();
}
