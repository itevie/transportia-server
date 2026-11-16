use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::resources::photo_rating::PhotoRating;

pub static SQL_PHOTO_RATING: &str = r#"
CREATE TABLE IF NOT EXISTS photo_ratings (
    id INTEGER PRIMARY KEY,
    photo_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    created_at TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct PhotoRatings {
    pool: Pool<SqliteConnectionManager>,
}

impl PhotoRatings {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<PhotoRating>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                photo_id,
                user_id,
                rating,
                created_at
            FROM photo_ratings
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_photo_rating(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_photo_id(
        &self,
        photo_id: i64,
    ) -> rusqlite::Result<Vec<PhotoRating>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                photo_id,
                user_id,
                rating,
                created_at
            FROM photo_ratings
            WHERE photo_id = ?
            ORDER BY created_at DESC
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![photo_id],
            Self::row_to_photo_rating,
        )?;

        rows.collect()
    }

    pub fn get_by_user_and_photo(
        &self,
        user_id: i64,
        photo_id: i64,
    ) -> rusqlite::Result<Option<PhotoRating>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                photo_id,
                user_id,
                rating,
                created_at
            FROM photo_ratings
            WHERE user_id = ?
              AND photo_id = ?
            "#,
        )?;

        let mut rows = statement.query(
            rusqlite::params![user_id, photo_id],
        )?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_photo_rating(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn average_rating(&self, photo_id: i64) -> rusqlite::Result<f64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.query_row(
            r#"
                SELECT COALESCE(AVG(rating), 0.0)
                FROM photo_ratings
                WHERE photo_id = ?
            "#,
            rusqlite::params![photo_id],
            |row| row.get(0),
        )
    }

    pub fn create(&self, rating: &PhotoRating) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
        INSERT INTO photo_ratings (
            photo_id,
            user_id,
            rating,
            created_at
        )
        VALUES (?, ?, ?, ?)
        "#,
            rusqlite::params![
            rating.photo_id,
            rating.user_id,
            rating.rating,
            &rating.created_at,
        ],
        )?;

        Ok(())
    }

    pub fn upsert(&self, rating: &PhotoRating) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO photo_ratings (
                id,
                photo_id,
                user_id,
                rating,
                created_at
            )
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                photo_id = excluded.photo_id,
                user_id = excluded.user_id,
                rating = excluded.rating,
                created_at = excluded.created_at
            "#,
            rusqlite::params![
                rating.id,
                rating.photo_id,
                rating.user_id,
                rating.rating,
                &rating.created_at,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM photo_ratings WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn delete_by_user_and_photo(
        &self,
        user_id: i64,
        photo_id: i64,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            DELETE FROM photo_ratings
            WHERE user_id = ?
              AND photo_id = ?
            "#,
            rusqlite::params![user_id, photo_id],
        )?;

        Ok(())
    }

    fn row_to_photo_rating(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<PhotoRating> {
        Ok(PhotoRating {
            id: row.get("id")?,
            photo_id: row.get("photo_id")?,
            user_id: row.get("user_id")?,
            rating: row.get("rating")?,
            created_at: row.get("created_at")?,
        })
    }
}