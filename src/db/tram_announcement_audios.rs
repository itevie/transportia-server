use std::fs;
use std::path::Path;
use std::sync::Arc;
use chrono::Utc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use serde::{Deserialize, Serialize};
use crate::db::Database;
use crate::db::resources::tram_announcement_audio::TramAnnouncementAudio;

pub static SQL_TRAM_ANNOUNCEMENT_AUDIO: &str = r#"
CREATE TABLE IF NOT EXISTS tram_announcement_audios (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_name TEXT NOT NULL,
    audio_group TEXT NOT NULL,
    label TEXT,
    operator_id INTEGER REFERENCES tram_operators(id),
    route_id INTEGER REFERENCES tram_routes(id),
    next_stop_id INTEGER REFERENCES tram_stops(id),
    created_at TEXT NOT NULL,
    UNIQUE(file_name, audio_group)
)
"#;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TramAnnouncementAudioQuery {
    pub file_name: Option<String>,
    pub audio_group: Option<String>,
    pub label: Option<String>,
    pub operator: Option<i64>,
    pub route: Option<i64>,
    pub next_stop: Option<i64>,
    pub search: Option<String>,
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone)]
pub struct TramAnnouncementAudios {
    pool: Pool<SqliteConnectionManager>,
}

impl TramAnnouncementAudios {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<TramAnnouncementAudio>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, file_name, audio_group, label, operator_id,
                   route_id, next_stop_id, created_at
            FROM tram_announcement_audios
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram_announcement_audio(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_groups(&self) -> rusqlite::Result<Vec<String>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            "SELECT DISTINCT audio_group
         FROM tram_announcement_audios
         ORDER BY audio_group"
        )?;

        let groups = statement
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;

        Ok(groups)
    }

    pub fn search(
        &self,
        query: &TramAnnouncementAudioQuery,
    ) -> rusqlite::Result<Vec<TramAnnouncementAudio>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).min(1000);

        let mut sql = String::from(
            r#"
        SELECT
            taa.id,
            taa.file_name,
            taa.audio_group,
            taa.label,
            taa.operator_id,
            taa.route_id,
            taa.next_stop_id,
            taa.created_at
        FROM tram_announcement_audios taa
        WHERE 1 = 1
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = &query.file_name {
            sql.push_str(" AND taa.file_name LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.audio_group {
            sql.push_str(" AND taa.audio_group LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.label {
            sql.push_str(" AND taa.label LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.operator {
            sql.push_str(" AND taa.operator_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.route {
            sql.push_str(" AND taa.route_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.next_stop {
            sql.push_str(" AND taa.next_stop_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.search {
            sql.push_str(
                r#"
            AND (
                taa.file_name LIKE '%' || ? || '%' OR
                taa.audio_group LIKE '%' || ? || '%' OR
                taa.label LIKE '%' || ? || '%'
            )
            "#,
            );

            for _ in 0..3 {
                params.push(Box::new(value.clone()));
            }
        }

        sql.push_str(" ORDER BY taa.audio_group, taa.file_name");
        sql.push_str(" LIMIT ?");
        params.push(Box::new(page_length));

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset));
        }

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_tram_announcement_audio,
            )?
            .collect()
    }


    pub fn create(
        &self,
        audio: &TramAnnouncementAudio,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT OR IGNORE INTO tram_announcement_audios (
                file_name,
                audio_group,
                label,
                operator_id,
                route_id,
                next_stop_id,
                created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &audio.file_name,
                &audio.audio_group,
                &audio.label,
                audio.operator_id,
                audio.route_id,
                audio.next_stop_id,
                &audio.created_at,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn scan_tram_audios<P: AsRef<Path>>(
        db: Arc<Database>,
        directory: P,
    ) -> std::io::Result<()> {
        let mut audios = Vec::new();

        for group_entry in fs::read_dir(directory)? {
            let group_entry = group_entry?;
            let group_path = group_entry.path();

            if !group_path.is_dir() {
                continue;
            }

            let audio_group = group_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();

            for audio_entry in fs::read_dir(&group_path)? {
                let audio_entry = audio_entry?;
                let audio_path = audio_entry.path();

                if !audio_path.is_file() {
                    continue;
                }

                let file_name = audio_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string();

                println!("Found tram announcement: {} {}", file_name.to_string(), audio_group.clone().to_string());

                audios.push(TramAnnouncementAudio {
                    id: 0,
                    file_name,
                    audio_group: audio_group.clone(),
                    label: None,
                    operator_id: None,
                    route_id: None,
                    next_stop_id: None,
                    created_at: Utc::now().to_rfc3339(),
                });
            }
        }

        for audio in audios {
            if let Err(e) = db.tram_announcement_audios.create(&audio) {
                eprintln!("{}", e.to_string());
            }
        }

        Ok(())
    }

    pub fn update(
        &self,
        audio: &TramAnnouncementAudio,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let updated = connection.execute(
            r#"
            UPDATE tram_announcement_audios
            SET
                file_name = ?,
                audio_group = ?,
                label = ?,
                operator_id = ?,
                route_id = ?,
                next_stop_id = ?,
                created_at = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &audio.file_name,
                &audio.audio_group,
                &audio.label,
                audio.operator_id,
                audio.route_id,
                audio.next_stop_id,
                &audio.created_at,
                audio.id,
            ],
        )?;

        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM tram_announcement_audios WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_tram_announcement_audio(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TramAnnouncementAudio> {
        Ok(TramAnnouncementAudio {
            id: row.get("id")?,
            file_name: row.get("file_name")?,
            audio_group: row.get("audio_group")?,
            label: row.get("label")?,
            operator_id: row.get("operator_id")?,
            route_id: row.get("route_id")?,
            next_stop_id: row.get("next_stop_id")?,
            created_at: row.get("created_at")?,
        })
    }
}
