#![feature(proc_macro_hygiene, decl_macro)]

mod events;

#[macro_use]
extern crate rocket;
use rocket::http::ContentType;
use rocket::response::content::Content;
use rocket::State;

use rocket_contrib::serve::StaticFiles;

use events::{Event, Location, Occurrence};

use chrono::prelude::*;

use maud::{html, Markup};

#[get("/")]
fn index(events: State<Events>) -> Markup {
    html! {
        h1 { "Lindy Hop Aachen" }
        ol {
            @for event in &events.0 {
                li { ( render_event(event) ) }
            }
        }
    }
}

fn render_event(event: &Event) -> Markup {
    html! {
        (event.name) " - " (event.teaser) ": " (event.occurrences.get(0).unwrap().location.name)
    }
}

#[get("/api/events")]
fn read_events() -> Content<&'static str> {
    Content(ContentType::JSON,
        r#"
        {
            "locations": [
                {
                    "id": 1,
                    "name": "Chico Mendès",
                    "address": "Pontstraße 74-76, 52062 Aachen"
                },
                {
                    "id": 2,
                    "name": "Sencillito",
                    "address": "Alexander Strasse 109, 52066 Aachen"
                }
            ],
            "events": [
                {
                    "id": 1,
                    "name": "Anfängerkurs",
                    "teaser": "Für diejenigen, die Lindy Hop ausprobieren möchten.",
                    "description": "Unter Anleitung werden dir die Grundschritte des Lindy Hops beigebracht.",
                    "occurrences": [
                        {
                            "id": 1,
                            "start": 1554140700,
                            "duration": 45,
                            "location": 1
                        },
                        {
                            "id": 2,
                            "start": 1554748200,
                            "duration": 90,
                            "location": 2
                        }
                    ]
                },
                {
                    "id": 2,
                    "name": "Social Dance",
                    "teaser": "Einfach Lindy Hop tanzen.",
                    "description": "Hier triffst du viele andere Menschen, die ebenfalls Lust haben, Lindy Hop zu tanzen.",
                    "occurrences": [
                        {
                            "id": 3,
                            "start": 1554143400,
                            "duration": 90,
                            "location": 1
                        },
                        {
                            "id": 4,
                            "start": 1554748200,
                            "duration": 90,
                            "location": 2
                        }
                    ]
                }
            ]
        }
        "#
    )
}

struct Events<'a>(Vec<Event<'a>>);

const CHICO: Location = Location {
    name: "Chico Mendès",
    address: "Aachen",
};

const SENCILLITO: Location = Location {
    name: "Sencillito",
    address: "Aachen",
};

fn main() {
    rocket::ignite()
        .manage(Events(vec![
            Event {
                name: "Social Dance",
                teaser: "Einfach tanzen.",
                description: "Lindy Hop tanzen in einer Bar.",
                occurrences: vec![
                    Occurrence {
                        start: Local
                            .ymd(2019, 4, 1)
                            .and_hms(20, 30, 00)
                            .with_timezone(&Utc),
                        duration: 90,
                        location: &CHICO,
                    },
                    Occurrence {
                        start: Local
                            .ymd(2019, 4, 8)
                            .and_hms(20, 30, 00)
                            .with_timezone(&Utc),
                        duration: 90,
                        location: &SENCILLITO,
                    },
                ],
            },
            Event {
                name: "Anfängerkurs",
                teaser: "Hereinschnuppern.",
                description:
                    "Ein Einführung für diejenigen, die noch nie Lindy Hop getanzt haben.",
                occurrences: vec![
                    Occurrence {
                        start: Local
                            .ymd(2019, 4, 1)
                            .and_hms(19, 45, 00)
                            .with_timezone(&Utc),
                        duration: 45,
                        location: &CHICO,
                    },
                    Occurrence {
                        start: Local
                            .ymd(2019, 4, 8)
                            .and_hms(20, 30, 00)
                            .with_timezone(&Utc),
                        duration: 90,
                        location: &SENCILLITO,
                    },
                ],
            },
        ]))
        .mount("/admin", {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/admin/dist");
            StaticFiles::from(path)
        })
        .mount("/", routes![index, read_events])
        .launch();
}
