use std::env;

use chrono::{Local, NaiveTime, Timelike};
use rand::seq::SliceRandom;

pub fn connect() -> rusqlite::Connection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    rusqlite::Connection::open(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub mod models {
    use chrono::NaiveTime;

    #[derive(Debug)]
    pub struct Program {
        pub id: i32,
        pub name: String,
        pub starts_at: NaiveTime,
        pub ends_at: NaiveTime,
    }

    #[derive(Debug)]
    pub struct Recording {
        pub id: i32,
        pub filename: String,
        pub title: String,
        pub artist: String,
        pub link: Option<String>,
    }
}


pub fn find_recording(conn: &rusqlite::Connection, recording_id: i64)
    -> Result<models::Recording, rusqlite::Error>
{
    conn.prepare("SELECT id, filename, title, artist, link FROM recordings WHERE id = ?1")?
        .query_row([recording_id], |row| {
            Ok(models::Recording {
                id: row.get(0)?,
                filename: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                link: row.get(4)?
            })
        })
}

pub fn find_program(conn: &rusqlite::Connection, program_id: i64)
    -> Result<models::Program, rusqlite::Error>
{
    conn.prepare("SELECT id, name, starts_at, ends_at FROM programs WHERE id = ?1")?
        .query_row([program_id], |row| {
            Ok(models::Program {
                id: row.get(0)?,
                name: row.get(1)?,
                starts_at: get_naivetime(row, 2)?,
                ends_at: get_naivetime(row, 3)?,
            })
        })
}

pub fn select_next_recording(conn: &rusqlite::Connection)
    -> Result<Option<(models::Program, models::Recording)>, rusqlite::Error>
{
    let now_subsec = Local::now().time();
    let now = NaiveTime::from_hms_opt(now_subsec.hour(), now_subsec.minute(), now_subsec.second()).unwrap();

    let results = conn
        .prepare("
            SELECT programs.id, recordings.id FROM recordings
            INNER JOIN recording_tags ON recording_tags.recording_id = recordings.id
            INNER JOIN program_tags ON program_tags.tag_id = recording_tags.tag_id
            INNER JOIN programs ON programs.id = program_tags.program_id
            LEFT JOIN plays ON plays.recording_id = recordings.id
            WHERE starts_at <= ?1 AND ends_at >= ?1 AND recordings.id NOT IN (
                SELECT recording_id FROM plays ORDER BY id DESC LIMIT 5
            )
            GROUP BY programs.id, recordings.id
            ORDER BY COUNT(plays.id) ASC
            LIMIT 8
        ")?
        .query_map([now.format("%T").to_string()], |row|
            Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<(i64, i64)>, _>>()?;

    let selection = results.choose(&mut rand::thread_rng());

    match selection {
        None => Ok(None),
        Some((program_id, recording_id)) => {
            let program = find_program(conn, *program_id)?;
            let recording = find_recording(conn, *recording_id)?;
            Ok(Some((program, recording)))
        }
    }
}

pub fn insert_play_record(conn: &rusqlite::Connection, program: &models::Program, recording: &models::Recording)
    -> Result<(), rusqlite::Error>
{
    conn.prepare("INSERT INTO plays (recording_id, program_id, started_at) VALUES (?1, ?2, ?3)")?
        .execute((recording.id, program.id, Local::now().format("%+").to_string()))?;

    Ok(())
}

fn get_naivetime(row: &rusqlite::Row, idx: usize) -> rusqlite::Result<NaiveTime> {
    let text = row.get_ref(idx)?.as_str()?;

    NaiveTime::parse_from_str(text, "%T")
        .map_err(|err| {
            let ty = rusqlite::types::Type::Text;
            rusqlite::Error::FromSqlConversionFailure(idx, ty, Box::new(err))
        })
}
