table! {
    plays (id) {
        id -> Int4,
        recording_id -> Int4,
        program_id -> Int4,
        started_at -> Timestamptz,
    }
}

table! {
    program_tags (id) {
        id -> Int4,
        program_id -> Int4,
        tag_id -> Int4,
    }
}

table! {
    programs (id) {
        id -> Int4,
        name -> Text,
        starts_at -> Time,
        ends_at -> Time,
    }
}

table! {
    recording_tags (id) {
        id -> Int4,
        recording_id -> Int4,
        tag_id -> Int4,
    }
}

table! {
    recordings (id) {
        id -> Int4,
        filename -> Text,
        title -> Text,
        artist -> Text,
        link -> Nullable<Text>,
    }
}

table! {
    tags (id) {
        id -> Int4,
        name -> Text,
    }
}

joinable!(plays -> programs (program_id));
joinable!(plays -> recordings (recording_id));
joinable!(program_tags -> programs (program_id));
joinable!(program_tags -> tags (tag_id));
joinable!(recording_tags -> recordings (recording_id));
joinable!(recording_tags -> tags (tag_id));

allow_tables_to_appear_in_same_query!(
    plays,
    program_tags,
    programs,
    recording_tags,
    recordings,
    tags,
);
