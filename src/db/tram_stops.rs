
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::db::resources::tram_stop::TramStop;

pub static SQL_TRAM_STOP: &str = r#"
CREATE TABLE IF NOT EXISTS tram_stops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    operator_id INTEGER REFERENCES tram_operators(id),
    created_at TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct TramStops {
    pool: Pool<SqliteConnectionManager>,
}

impl TramStops {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<TramStop>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, operator_id, created_at
            FROM tram_stops
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram_stop(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<TramStop>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, operator_id, created_at
            FROM tram_stops
            WHERE name LIKE ?
            ORDER BY name
            LIMIT 50
            "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![search],
            Self::row_to_tram_stop,
        )?;

        rows.collect()
    }

    pub fn create(&self, stop: &TramStop) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tram_stops (
                name,
                operator_id,
                created_at
            )
            VALUES (?, ?, ?)
            "#,
            rusqlite::params![
                &stop.name,
                &stop.operator_id,
                &stop.created_at,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, stop: &TramStop) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let updated = connection.execute(
            r#"
            UPDATE tram_stops
            SET
                name = ?,
                operator_id = ?,
                created_at = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &stop.name,
                &stop.operator_id,
                &stop.created_at,
                stop.id,
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
            "DELETE FROM tram_stops WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_tram_stop(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TramStop> {
        Ok(TramStop {
            id: row.get("id")?,
            name: row.get("name")?,
            operator_id: row.get("operator_id")?,
            created_at: row.get("created_at")?,
        })
    }
}
