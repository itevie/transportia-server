use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::resources::tram_type::TramType;

pub static SQL_TRAM_TYPE: &str = r#"
CREATE TABLE IF NOT EXISTS tram_type (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct TramTypes {
    pool: Pool<SqliteConnectionManager>,
}

impl TramTypes {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<TramType>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name
            FROM tram_type
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram_type(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<TramType>> {
        println!("{}", query);
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name
            FROM tram_type
            WHERE name LIKE ?
            ORDER BY name
            LIMIT 50
            "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![search],
            Self::row_to_tram_type,
        )?;

        rows.collect()
    }

    pub fn create(&self, tram_type: &TramType) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tram_type (name)
            VALUES (?)
            "#,
            rusqlite::params![&tram_type.name],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, tram_type: &TramType) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let updated = connection.execute(
            r#"
            UPDATE tram_type
            SET name = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &tram_type.name,
                tram_type.id,
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
            "DELETE FROM tram_type WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_tram_type(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TramType> {
        Ok(TramType {
            id: row.get("id")?,
            name: row.get("name")?,
        })
    }
}