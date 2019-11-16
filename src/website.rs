use std::collections::HashMap;

use chrono::prelude::*;
use diesel::result::QueryResult;
use maud::{html, Markup, DOCTYPE};
use rocket::Rocket;

use crate::store::{Query, Store};

pub fn mount(rocket: Rocket, prefix: &'static str) -> Rocket {
    rocket.mount(
        prefix,
        routes![occurrence_overview, event_overview, event_details, homepage],
    )
}

#[get("/termine")]
fn occurrence_overview(store: Store) -> Markup {
    base_html(
        html! {
            ol.schedule {
                @let locations = Query::all_locations(&store);
                @for (date, entries) in store.occurrences_by_date(&OccurrenceFilter::upcoming()) {
                    li { ( render_entry(date, &entries, &locations) ) }
                }
            }
        },
        Some(&Page::OccurrenceOverview),
    )
}

#[get("/angebote")]
fn event_overview(store: Store) -> Markup {
    base_html(
        html! {
            ol.events {
                @let locations: HashMap<Id<Location>, Location> = store.all();
                @let events = store.all_events_with_occurrences(&OccurrenceFilter::upcoming());
                @for (id, event) in events.iter() {
                    li { ( render_event(id, event, &locations) ) }
                }
            }
        },
        Some(&Page::EventsOverview),
    )
}

#[get("/angebote/<id>")]
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
        Some(&Page::EventsOverview),
    ))
}

#[get("/")]
fn homepage(store: Store) -> Markup {
    base_html(
        html! {
            section.preview {
                h1 { "Nächster Termin" }
                ol.schedule {
                    li {
                        @let locations: HashMap<Id<Location>, Location> = store.all();
                        @let occurrences = store.occurrences_by_date(&OccurrenceFilter::upcoming());
                        @let next_event = occurrences.into_iter().next();
                        @if let Some((date, entries)) = next_event{
                            ( render_entry(date, &entries, &locations) )
                        }
                    }
                }
                a href=( Page::OccurrenceOverview.url() ) { "Alle Termine" }
            }
            section.infos {
                h1 { "Über uns" }
                p { "Wir sind eine Gruppe von Aachenern, die gerne Lindy Hop tanzt. Wir organisieren selbstständig Events. Diese Seite soll alles vorstellen, was für Lindy Hop in Aachen wichtig ist." }
                h1 { "Das erste Mal" }
                p { "Wenn du noch nie Lindy Hop getanzt hast, empfehlen wir dir, den Anfängerkurs abzuwarten. Diesen bieten wir einmal im Monat an. Im Anschluss kannst du das Erlernte im Social ausprobieren; da tanzt frei jeder mit jedem." }
                p { "Du brauchst keinen festen Partner. Du kannst dir aussuchen, ob du leader oder follower lernen willst. Auch wenn die meisten eines davon bevorzugen, können viele Tänzer sowohl führen als auch folgen, du kannst also jederzeit die andere Seite ausprobieren." }
                p { "Um dir schon einen Eindruck von der Musik zu verschaffen, kannst du gerne in unsere Playlisten reinhören." }
            }
        },
        None,
    )
}

#[derive(PartialEq)]
enum Page {
    OccurrenceOverview,
    EventsOverview,
}

impl Page {
    fn url(&self) -> String {
        use Page::*;

        match self {
            OccurrenceOverview => uri!(occurrence_overview).to_string(),
            EventsOverview => uri!(event_overview).to_string(),
        }
    }

    fn title(&self) -> &'static str {
        use Page::*;

        match self {
            OccurrenceOverview => "Termine",
            EventsOverview => "Angebote",
        }
    }
}

fn base_html(main: Markup, current_page: Option<&Page>) -> Markup {
    use Page::*;
    html! {
        ( DOCTYPE )
        html lang="de" {
            head {
                title { "Lindy Hop Aachen" }
                meta name="viewport" content="width=device-width, initial-scale=1";

                link href="/static/main.css" rel="stylesheet";
            }
            body {
                header {
                    div.header {
                        a.title href="/" { h1 { "Lindy Hop Aachen" } }
                        nav {
                            ol {
                                @for page in vec![OccurrenceOverview, EventsOverview] {
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

fn nav_entry(page: Page, current: Option<&Page>) -> Markup {
    html! {
        a.current[current == Some(&page)] href=( page.url() ) { ( page.title() ) }
    }
}

fn render_entry(
    date: NaiveDate,
    entries: &[OccurrenceWithEvent],
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    html! {
        div.date { ( format_date(date) ) }
        ol.events {
            @for occurrence_entry in entries {
                li.event {
                        ( render_occurrence(&occurrence_entry, locations) )
                }
            }
        }
    }
}

fn format_date(date: NaiveDate) -> String {
    let day = format_weekday(date.weekday());
    let format = format!("{}, %d.%m.", day);

    date.format(&format).to_string()
}

fn format_weekday(day: Weekday) -> &'static str {
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
                div.teaser {
                    p { ( entry_html.teaser ) }
                    a href=( format!("{}/{}", Page::EventsOverview.url(), entry.event_id) ) {
                        "Mehr erfahren"
                    }
                }
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
    event_id: &Id<Event>,
    event_with_occurrences: &EventWithOccurrences,
    locations: &HashMap<Id<Location>, Location>,
) -> Markup {
    html! {
        div.event {
            div.overview {
                h2 { ( event_with_occurrences.event.title ) }
                p { (event_with_occurrences.event.teaser ) }
                a href=( uri!(event_details: event_id) ) { "Mehr erfahren" }
            }
            div.occurrences {
                h3 { "Termine" }
                ol {
                    @let preview_length = 5;
                    @let occurrences = event_with_occurrences.occurrences.iter().take(preview_length);
                    @let remaining = event_with_occurrences.occurrences.len().saturating_sub(preview_length);
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
    let date = format_date(occurrence_with_location.occurrence.start.date());
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
