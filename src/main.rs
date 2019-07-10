#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

mod api;
mod store;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use maud::{html, Markup, DOCTYPE};
use rocket::fairing::AdHoc;
use rocket::response::NamedFile;
use rocket::State;

use store::{
    Actions, Event, Id, Location, OccurrenceFilter, OccurrenceWithEvent, OccurrenceWithLocation,
    Store,
};

#[get("/")]
fn index(store: Store) -> Markup {
    html! {
        ( DOCTYPE )
        html lang="de" {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1";

                link href="static/main.css" rel="stylesheet";
            }
            body {
                header {
                    div class="box" {}
                    object type="image/svg+xml" data="static/shoe_text.svg" {}
                    div class="links" {
                        span {
                            a class="link" href="occurrences" { "Termine" }
                            a class="link" href="events" {"Veranstaltungen"}
                        }
                        span {
                            a class="link" href="#infos" {"Infos"}
                            a class="link" href="#newsletter" {"Newsletter"}
                        }
                    }
                }
                main {
                    ol.schedule {
                        @let locations: HashMap<Id<Location>, Location> = store.all();
                        @for occurrences_for_date in store.occurrences_by_date(&OccurrenceFilter::upcoming()) {
                            li { ( render_entry(&occurrences_for_date, &locations) ) }
                        }
                    }
                }
            }
        }
    }
}

fn render_entry(
    (date, entries): &(NaiveDate, Vec<OccurrenceWithEvent>),
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    html! {
        div.date { ( format_date(date) ) }
        ol.events {
            @for occurrence_entry in entries {
                li.event { ( render_occurrence(occurrence_entry, locations) ) }
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

fn render_occurrence(
    entry: &OccurrenceWithEvent,
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    html! {
        @let entry_html =  html_from_occurrence(&entry.occurrence, &entry.event, locations);
        div.quick-info { ( entry_html.quick_info ) }
        h2.title { ( entry_html.title ) }
        div.content {
            div.description {
                div.teaser { ( entry_html.teaser ) }
            }
        }
    }
}

struct OccurrenceHtml {
    title: Markup,
    quick_info: Markup,
    teaser: Markup,
}

fn html_from_occurrence(
    occurrence: &OccurrenceWithLocation,
    event: &Event,
    locations: &HashMap<Id<Location>, Location>,
) -> OccurrenceHtml {
    let maybe_location = locations.get(&occurrence.location_id);
    let location_name = match maybe_location {
        Some(location) => &location.name,
        None => "Steht noch nicht fest.",
    };

    OccurrenceHtml {
        title: html! { ( event.title ) },
        quick_info: html! { ( format!("{} - {}", occurrence.occurrence.start.format("%H:%M"), location_name) ) },
        teaser: html! { ( event.teaser ) },
    }
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

#[derive(Debug)]
struct AssetsDir(PathBuf);

#[get("/static/<file..>")]
fn static_file(file: PathBuf, assets_dir: State<AssetsDir>) -> Option<NamedFile> {
    let path = assets_dir.0.join(file);
    print!("{:?}", path);
    NamedFile::open(path).ok()
}

fn main() {
    let rocket = rocket::ignite()
        .attach(Store::fairing())
        .attach(AdHoc::on_attach("Assets Config", |rocket| {
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
        }))
        .mount(
            "/",
            routes![static_file, index, admin_route, admin_subroute],
        );
    api::mount(rocket, "/api").launch();
}
