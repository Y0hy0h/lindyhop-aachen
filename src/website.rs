use std::collections::HashMap;

use chrono::prelude::*;
use diesel::result::QueryResult;
use maud::{html, Markup, DOCTYPE};
use rocket::Rocket;

use crate::store::{
    Actions, Event, EventWithOccurrences, Id, Location, OccurrenceFilter, OccurrenceWithEvent,
    OccurrenceWithLocation, Store,
};

pub fn mount(rocket: Rocket, prefix: &'static str) -> Rocket {
    rocket.mount(
        prefix,
        routes![occurrence_overview, event_overview, event_details],
    )
}

#[get("/")]
fn occurrence_overview(store: Store) -> Markup {
    base_html(
        html! {
            ol.schedule {
                @let locations: HashMap<Id<Location>, Location> = store.all();
                @for occurrences_for_date in store.occurrences_by_date(&OccurrenceFilter::upcoming()) {
                    li { ( render_entry(&occurrences_for_date, &locations) ) }
                }
            }
        },
        &Page::OccurrenceOverview,
    )
}

#[get("/veranstaltungen")]
fn event_overview(store: Store) -> Markup {
    base_html(
        html! {
            ol.events {
                @let locations: HashMap<Id<Location>, Location> = store.all();
                @let events = store.all_events_with_occurrences(&OccurrenceFilter::upcoming());
                @for event in events.values() {
                    li { ( render_event(event, &locations) ) }
                }
            }
        },
        &Page::EventsOverview,
    )
}

#[get("/veranstaltungen/<id>")]
fn event_details(store: Store, id: Id<Event>) -> QueryResult<Markup> {
    let EventWithOccurrences { event, occurrences } =
        store.read_event_with_occurrences(id, &OccurrenceFilter::upcoming())?;
    let locations: HashMap<Id<Location>, Location> = store.all();

    Ok(base_html(
        html! {
            div.event {
                div.details {
                    h1 { ( event.title ) }
                    p { ( event.description ) }
                }
                ol.occurrences {
                    h2 { "Termine" }
                    @for occurrence in occurrences {
                        li {
                            ( quickinfo_occurrence(&occurrence, &locations) )
                        }
                    }
                }
            }
        },
        &Page::EventsOverview,
    ))
}

#[derive(PartialEq)]
enum Page {
    OccurrenceOverview,
    EventsOverview,
    Infos,
}

impl Page {
    fn url(&self) -> &'static str {
        use Page::*;

        match self {
            OccurrenceOverview => "/",
            EventsOverview => "/veranstaltungen",
            Infos => "/infos",
        }
    }

    fn title(&self) -> &'static str {
        use Page::*;

        match self {
            OccurrenceOverview => "Termine",
            EventsOverview => "Veranstaltungen",
            Infos => "Infos",
        }
    }
}

fn base_html(main: Markup, current_page: &Page) -> Markup {
    use Page::*;
    html! {
        ( DOCTYPE )
        html lang="de" {
            head {
                meta name="viewport" content="width=device-width, initial-scale=1";

                link href="/static/main.css" rel="stylesheet";
            }
            body {
                header {
                    div.header {
                        a.title href="/" { h1 { "Lindy Hop Aachen" } }
                        nav {
                            ol {
                                @for page in vec![OccurrenceOverview, EventsOverview, Infos] {
                                    li { ( nav_entry(page, current_page) ) }
                                }
                            }
                        }
                    }
                }
                main {
                    ( main )
                }
            }
        }
    }
}

fn nav_entry(page: Page, current: &Page) -> Markup {
    html! {
        a.current[current == &page] href=( page.url() ) { ( page.title() ) }
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
    let day = format_weekday(&date.weekday());
    let format = format!("{}, %d.%m.", day);

    date.format(&format).to_string()
}

fn format_weekday(day: &Weekday) -> &'static str {
    use chrono::Weekday::*;

    match day {
        Mon => "Mo",
        Tue => "Di",
        Wed => "Mi",
        Thu => "Do",
        Fri => "Fr",
        Sat => "Sa",
        Sun => "So",
    }
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

fn render_event(
    event_with_occurrences: &EventWithOccurrences,
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    html! {
        div.event {
            div.overview {
                h2 { ( event_with_occurrences.event.title ) }
                p { (event_with_occurrences.event.teaser ) }
            }
            div.occurrences {
                h3 { "Termine" }
                ol {
                    @let preview_length = 5;
                    @let occurrences = event_with_occurrences.occurrences.iter().take(preview_length);
                    @let remaining = event_with_occurrences.occurrences.len().checked_sub(preview_length).unwrap_or(0);
                    @for occurrence in occurrences {
                        li {
                            ( quickinfo_occurrence(occurrence, locations) )
                        }
                    }
                    @if remaining > 0 {
                        span.overflow { ( format!("(+ {} weitere)", remaining) ) }
                    }
                }
            }
        }
    }
}

fn quickinfo_occurrence(
    occurrence_with_location: &OccurrenceWithLocation,
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    let date = format_date(&occurrence_with_location.occurrence.start.date());
    let time = occurrence_with_location.occurrence.start.format("%H:%M");
    let maybe_location = locations.get(&occurrence_with_location.location_id);
    let location_name = match maybe_location {
        Some(location) => &location.name,
        None => "Steht noch nicht fest.",
    };
    html! {
        span.quick-info { ( format!("{} {} - {}", date, time, location_name)) }
    }
}
