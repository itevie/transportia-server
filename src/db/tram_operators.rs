use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::db::resources::tram_operator::TramOperator;

pub static SQL_TRAM_OPERATOR: &str = r#"
CREATE TABLE IF NOT EXISTS tram_operators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    location TEXT,
    website TEXT
)
"#;

#[derive(Clone)]
pub struct TramOperators {
    pool: Pool<SqliteConnectionManager>,
}

impl TramOperators {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<TramOperator>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, location, website
            FROM tram_operators
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram_operator(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<TramOperator>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, location, website
            FROM tram_operators
            WHERE
                name LIKE ?
                OR location LIKE ?
            ORDER BY name
            LIMIT 50
            "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![search, search],
            Self::row_to_tram_operator,
        )?;

        rows.collect()
    }

    pub fn create(&self, operator: &TramOperator) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tram_operators (
                name,
                location,
                website
            )
            VALUES (?, ?, ?)
            "#,
            rusqlite::params![
                &operator.name,
                &operator.location,
                &operator.website,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, operator: &TramOperator) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let updated = connection.execute(
            r#"
            UPDATE tram_operators
            SET
                name = ?,
                location = ?,
                website = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &operator.name,
                &operator.location,
                &operator.website,
                operator.id,
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
            "DELETE FROM tram_operators WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_tram_operator(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TramOperator> {
        Ok(TramOperator {
            id: row.get("id")?,
            name: row.get("name")?,
            location: row.get("location")?,
            website: row.get("website")?,
        })
    }
}