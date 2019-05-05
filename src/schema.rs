table! {
    events (id) {
        id -> Nullable<Integer>,
        name -> Text,
        teaser -> Text,
        description -> Text,
    }
}

table! {
    locations (id) {
        id -> Nullable<Integer>,
        name -> Text,
        address -> Text,
    }
}

table! {
    occurrences (id) {
        id -> Nullable<Integer>,
        start -> Timestamp,
        event_id -> Integer,
        location_id -> Integer,
    }
}

joinable!(occurrences -> events (event_id));
joinable!(occurrences -> locations (location_id));

allow_tables_to_appear_in_same_query!(
    events,
    locations,
    occurrences,
);
