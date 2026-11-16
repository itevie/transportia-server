use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use chrono::Utc;

use crate::db::resources::comments::CommentThread;

pub static SQL_COMMENT_THREAD: &str = r#"
CREATE TABLE IF NOT EXISTS comment_threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct CommentThreads {
    pool: Pool<SqliteConnectionManager>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct CommentThreadQuery {
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl CommentThreads {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<CommentThread>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, created_at
            FROM comment_threads
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_comment_thread(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(&self) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO comment_threads (created_at)
            VALUES (?)
            "#,
            rusqlite::params![Utc::now().to_rfc3339()],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn search(
        &self,
        query: &CommentThreadQuery,
    ) -> rusqlite::Result<Vec<CommentThread>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);
        let offset = query.offset.unwrap_or(0).max(0);

        let mut statement = connection.prepare(
            r#"
            SELECT id, created_at
            FROM comment_threads
            ORDER BY id DESC
            LIMIT ? OFFSET ?
            "#,
        )?;

        statement
            .query_map(
                rusqlite::params![page_length, offset],
                Self::row_to_comment_thread,
            )?
            .collect()
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM comment_threads WHERE id = ?",
            rusqlite::params![id],
        )?;

        connection.execute(
            "DELETE FROM comments WHERE thread = ?",
            rusqlite::params![id]
        )?;

        Ok(())
    }

    fn row_to_comment_thread(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<CommentThread> {
        Ok(CommentThread {
            id: row.get("id")?,
            created_at: row.get("created_at")?,
        })
    }
}