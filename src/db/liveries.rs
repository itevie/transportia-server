use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::resources::bt_livery::BtLivery;

pub static SQL_BT_LIVERY: &str = r#"
CREATE TABLE IF NOT EXISTS bt_liveries (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    left_css TEXT NOT NULL,
    right_css TEXT NOT NULL,
    white_text INTEGER NOT NULL,
    text_colour TEXT NOT NULL,
    stroke_colour TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct BtLiveries {
    pool: Pool<SqliteConnectionManager>,
}

impl BtLiveries {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<BtLivery>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                name,
                left_css,
                right_css,
                white_text,
                text_colour,
                stroke_colour
            FROM bt_liveries
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_bt_livery(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<BtLivery>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
        SELECT
            id,
            name,
            left_css,
            right_css,
            white_text,
            text_colour,
            stroke_colour
        FROM bt_liveries
        WHERE name LIKE ?
        ORDER BY name
        LIMIT 50
        "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![search],
            Self::row_to_bt_livery,
        )?;

        rows.collect()
    }

    pub fn get_by_name(&self, name: &str) -> rusqlite::Result<Option<BtLivery>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                name,
                left_css,
                right_css,
                white_text,
                text_colour,
                stroke_colour
            FROM bt_liveries
            WHERE name = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_bt_livery(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(&self, livery: &BtLivery) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO bt_liveries (
                id,
                name,
                left_css,
                right_css,
                white_text,
                text_colour,
                stroke_colour
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                livery.id,
                &livery.name,
                &livery.left_css,
                &livery.right_css,
                &livery.white_text,
                &livery.text_colour,
                &livery.stroke_colour,
            ],
        )?;

        Ok(())
    }

    pub fn upsert(&self, livery: &BtLivery) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO bt_liveries (
                id,
                name,
                left_css,
                right_css,
                white_text,
                text_colour,
                stroke_colour
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                left_css = excluded.left_css,
                right_css = excluded.right_css,
                white_text = excluded.white_text,
                text_colour = excluded.text_colour,
                stroke_colour = excluded.stroke_colour
            "#,
            rusqlite::params![
                livery.id,
                &livery.name,
                &livery.left_css,
                &livery.right_css,
                &livery.white_text,
                &livery.text_colour,
                &livery.stroke_colour,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM bt_liveries WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_bt_livery(row: &rusqlite::Row) -> rusqlite::Result<BtLivery> {
        Ok(BtLivery {
            id: row.get("id")?,
            name: row.get("name")?,
            left_css: row.get("left_css")?,
            right_css: row.get("right_css")?,
            white_text: row.get("white_text")?,
            text_colour: row.get("text_colour")?,
            stroke_colour: row.get("stroke_colour")?,
        })
    }
}