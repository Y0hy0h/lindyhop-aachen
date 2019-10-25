#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod api;
mod store;
// mod website;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::path::{Path, PathBuf};

use rocket::fairing::AdHoc;
use rocket::response::NamedFile;
use rocket::State;

use store::Store;

fn main() {
    let rocket = rocket::ignite()
        .attach(Store::fairing())
        .attach(assets_fairing())
        .mount("/", routes![static_file, admin_route, admin_subroute]);
    let rocket = api::mount(rocket, "/api");
    //let rocket = website::mount(rocket, "/");
    rocket.launch();
}

fn assets_fairing() -> AdHoc {
    AdHoc::on_attach("Assets Config", |rocket| {
        let assets_dir = PathBuf::from(rocket.config().get_str("assets_dir").unwrap_or("."));
        if assets_dir.exists() {
            Ok(rocket.manage(AssetsDir(assets_dir)))
        } else {
            eprintln!(
                "The assets directory '{}' does not exist.",
                assets_dir.display()
            );

            Err(rocket)
        }
    })
}

#[get("/admin")]
fn admin_route() -> Option<NamedFile> {
    admin()
}

// We also want to serve the file when subroutes are called, e. g. `/admin/event/42`.
// Removing this route would break reloading the admin on subroutes.
#[get("/admin/<path..>")]
#[allow(unused_variables)]
fn admin_subroute(path: PathBuf) -> Option<NamedFile> {
    admin()
}

fn admin() -> Option<NamedFile> {
    NamedFile::open(Path::new("admin/dist/index.html")).ok()
}

#[derive(Debug)]
struct AssetsDir(PathBuf);

#[get("/static/<file..>")]
fn static_file(file: PathBuf, assets_dir: State<AssetsDir>) -> Option<NamedFile> {
    let path = assets_dir.0.join(file);
    NamedFile::open(path).ok()
}
