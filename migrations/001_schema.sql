CREATE TABLE recordings (
    id INTEGER PRIMARY KEY,
    filename TEXT NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    link TEXT
);

CREATE TABLE programs (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    starts_at NaiveTime_TEXT NOT NULL,
    ends_at NaiveTime_TEXT NOT NULL
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE recording_tags (
    id INTEGER PRIMARY KEY,
    recording_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL
);

CREATE TABLE program_tags (
    id INTEGER PRIMARY KEY,
    program_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL
);

CREATE TABLE plays (
    id INTEGER PRIMARY KEY,
    recording_id INTEGER NOT NULL,
    program_id INTEGER NOT NULL,
    started_at DateTime_TEXT NOT NULL
);
