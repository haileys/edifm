pub mod schema;

use std::env;

use chrono::{Local, DateTime, NaiveTime, Timelike};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use rand::{thread_rng, Rng};

pub fn connect() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub mod models {
    use chrono::NaiveTime;

    #[derive(Queryable)]
    pub struct Program {
        pub id: i32,
        pub name: String,
        pub starts_at: NaiveTime,
        pub ends_at: NaiveTime,
    }

    #[derive(Queryable)]
    pub struct Recording {
        pub id: i32,
        pub filename: String,
        pub title: String,
        pub artist: String,
        pub link: Option<String>,
    }
}

pub fn current_program(conn: &PgConnection) -> Result<Option<models::Program>, diesel::result::Error> {
    use schema::programs::dsl::*;

    let now_subsec = Local::now().time();
    let now = NaiveTime::from_hms(now_subsec.hour(), now_subsec.minute(), now_subsec.second());

    programs
        .filter(starts_at.le(now))
        .filter(ends_at.ge(now))
        .first(conn)
        .optional()
}

pub fn select_next_recording(conn: &PgConnection)
    -> Result<Option<(models::Program, models::Recording)>, diesel::result::Error>
{
    use diesel::dsl::sql;
    use diesel::sql_types::Integer;

    use schema;

    let now_subsec = Local::now().time();
    let now = NaiveTime::from_hms(now_subsec.hour(), now_subsec.minute(), now_subsec.second());

    let previous_two_tunes = schema::plays::table
        .order(schema::plays::id.desc())
        .select(schema::plays::recording_id)
        .limit(2)
        .get_results::<i32>(conn)?;

    let query = schema::recordings::table
        .inner_join(schema::recording_tags::table.on(schema::recordings::id.eq(schema::recording_tags::recording_id)))
        .inner_join(schema::program_tags::table.on(schema::program_tags::tag_id.eq(schema::recording_tags::tag_id)))
        .inner_join(schema::programs::table.on(schema::programs::id.eq(schema::program_tags::program_id)))
        .left_join(schema::plays::table.on(schema::plays::recording_id.eq(schema::recordings::id)))
        .group_by((schema::programs::id, schema::recordings::id))
        .order(sql::<Integer>("COUNT(plays.*)").asc())
        .filter(schema::programs::starts_at.le(now))
        .filter(schema::programs::ends_at.ge(now))
        .filter(schema::recordings::id.ne_all(&previous_two_tunes))
        .limit(5)
        .select((schema::programs::id, schema::recordings::id));

    let results = query.get_results::<(i32, i32)>(conn)?;

    let selection = thread_rng().choose(&results);

    match selection {
        None => Ok(None),
        Some((program_id, recording_id)) => {
            let program = schema::programs::table.find(program_id).first(conn)?;
            let recording = schema::recordings::table.find(recording_id).first(conn)?;
            Ok(Some((program, recording)))
        }
    }
}

pub fn insert_play_record(conn: &PgConnection, program: models::Program, recording: models::Recording)
    -> Result<(), diesel::result::Error>
{
    use schema::plays;

    #[derive(Insertable)]
    #[table_name = "plays"]
    struct Play {
        recording_id: i32,
        program_id: i32,
        started_at: DateTime<Local>,
    }

    diesel::insert_into(schema::plays::table).values(Play {
        recording_id: recording.id,
        program_id: program.id,
        started_at: Local::now(),
    }).execute(conn)?;

    Ok(())
}
