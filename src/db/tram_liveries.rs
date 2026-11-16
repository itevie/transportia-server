use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::db::resources::tram_livery::TramLivery;

pub static SQL_TRAM_LIVERY: &str = r#"
CREATE TABLE IF NOT EXISTS tram_livery (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    css TEXT,
    text_colour TEXT
)
"#;

#[derive(Clone)]
pub struct TramLiveries {
    pool: Pool<SqliteConnectionManager>,
}

impl TramLiveries {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<TramLivery>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, css, text_colour
            FROM tram_livery
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram_livery(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<TramLivery>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, css, text_colour
            FROM tram_livery
            WHERE
                name LIKE ?
                OR css LIKE ?
                OR text_colour LIKE ?
            ORDER BY name
            LIMIT 50
            "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![search, search, search],
            Self::row_to_tram_livery,
        )?;

        rows.collect()
    }

    pub fn create(&self, livery: &TramLivery) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tram_livery (
                name,
                css,
                text_colour
            )
            VALUES (?, ?, ?)
            "#,
            rusqlite::params![
                &livery.name,
                &livery.css,
                &livery.text_colour,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, livery: &TramLivery) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let updated = connection.execute(
            r#"
            UPDATE tram_livery
            SET
                name = ?,
                css = ?,
                text_colour = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &livery.name,
                &livery.css,
                &livery.text_colour,
                livery.id,
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
            "DELETE FROM tram_livery WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_tram_livery(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TramLivery> {
        Ok(TramLivery {
            id: row.get("id")?,
            name: row.get("name")?,
            css: row.get("css")?,
            text_colour: row.get("text_colour")?,
        })
    }
}