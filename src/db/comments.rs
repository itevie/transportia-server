use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use chrono::Utc;
use crate::db::Database;
use crate::db::resources::comments::{Comment, CommentFilledOut};
use crate::db::resources::favourites::FavouriteType;

pub static SQL_COMMENT: &str = r#"
CREATE TABLE IF NOT EXISTS comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    thread INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    body TEXT NOT NULL,
    edited INTEGER NOT NULL DEFAULT 0,
    edited_at TEXT,
    parent_comment_id INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY (thread) REFERENCES comment_threads(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_comment_id) REFERENCES comments(id) ON DELETE CASCADE
)
"#;

#[derive(Clone)]
pub struct Comments {
    pool: Pool<SqliteConnectionManager>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct CommentQuery {
    pub thread: Option<i64>,
    pub author_id: Option<i64>,
    pub parent_comment_id: Option<i64>,
    pub search: Option<String>,
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl Comments {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<Comment>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                thread,
                author_id,
                body,
                edited,
                edited_at,
                parent_comment_id,
                created_at
            FROM comments
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_comment(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_thread(
        &self,
        thread: i64,
    ) -> rusqlite::Result<Vec<Comment>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                thread,
                author_id,
                body,
                edited,
                edited_at,
                parent_comment_id,
                created_at
            FROM comments
            WHERE thread = ?
            ORDER BY created_at DESC, id DESC
            "#,
        )?;

        statement
            .query_map(
                rusqlite::params![thread],
                Self::row_to_comment,
            )?
            .collect()
    }

    pub fn get_replies(
        &self,
        parent_comment_id: i64,
    ) -> rusqlite::Result<Vec<Comment>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                thread,
                author_id,
                body,
                edited,
                edited_at,
                parent_comment_id,
                created_at
            FROM comments
            WHERE parent_comment_id = ?
            ORDER BY created_at ASC, id ASC
            "#,
        )?;

        statement
            .query_map(
                rusqlite::params![parent_comment_id],
                Self::row_to_comment,
            )?
            .collect()
    }

    pub fn create(
        &self,
        thread: i64,
        author_id: i64,
        body: &str,
        parent_comment_id: Option<i64>,
    ) -> rusqlite::Result<Comment> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
        INSERT INTO comments (
            thread,
            author_id,
            body,
            edited,
            edited_at,
            parent_comment_id,
            created_at
        )
        VALUES (?, ?, ?, 0, NULL, ?, ?)
        "#,
            rusqlite::params![
            thread,
            author_id,
            body,
            parent_comment_id,
            Utc::now().to_rfc3339(),
        ],
        )?;

        let id = connection.last_insert_rowid();

        self.get(id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn update(&self, comment: &Comment) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE comments SET
                body = ?,
                edited = 1,
                edited_at = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &comment.body,
                Utc::now().to_rfc3339(),
                comment.id,
            ],
        )?;

        Ok(())
    }

    pub fn search(
        &self,
        query: &CommentQuery,
    ) -> rusqlite::Result<Vec<Comment>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);

        let mut sql = String::from(
            r#"
            SELECT
                id,
                thread,
                author_id,
                body,
                edited,
                edited_at,
                parent_comment_id,
                created_at
            FROM comments
            WHERE 1 = 1
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(thread) = query.thread {
            sql.push_str(" AND thread = ?");
            params.push(Box::new(thread));
        }

        if let Some(author_id) = query.author_id {
            sql.push_str(" AND author_id = ?");
            params.push(Box::new(author_id));
        }

        if let Some(parent_comment_id) = query.parent_comment_id {
            sql.push_str(" AND parent_comment_id = ?");
            params.push(Box::new(parent_comment_id));
        }

        if let Some(search) = &query.search {
            sql.push_str(" AND body LIKE '%' || ? || '%' ");
            params.push(Box::new(search.clone()));
        }

        sql.push_str(" ORDER BY created_at ASC, id ASC LIMIT ?");
        params.push(Box::new(page_length));

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset.max(0)));
        }

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_comment,
            )?
            .collect()
    }

    pub fn fill_out(db: Arc<Database>, comment: Comment, user_id: Option<i64>) -> rusqlite::Result<CommentFilledOut> {
        let user = db
            .users
            .get(comment.author_id, false)?
            .unwrap();

        let favourite = match user_id {
            Some(user) => {
                db.favourites.get(user, FavouriteType::Comment, comment.id)?
            },
            None => None
        };

        Ok(CommentFilledOut {
            id: comment.id,
            thread: comment.thread,
            author_id: comment.author_id,
            author: user,
            body: comment.body,
            edited: comment.edited,
            edited_at: comment.edited_at,
            parent_comment_id: comment.parent_comment_id,
            created_at: comment.created_at,

            is_favourited: if let Some(_) = user_id {
                Some(favourite.is_some())
            } else {
                None
            }
        })
    }

    pub fn fill_out_many(db: Arc<Database>, comments: Vec<Comment>, user_id: Option<i64>) -> rusqlite::Result<Vec<CommentFilledOut>> {
        comments
            .into_iter()
            .map(|x| Self::fill_out(db.clone(), x, user_id))
            .collect()
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM comments WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_comment(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<Comment> {
        Ok(Comment {
            id: row.get("id")?,
            thread: row.get("thread")?,
            author_id: row.get("author_id")?,
            body: row.get("body")?,
            edited: row.get::<_, i64>("edited")? != 0,
            edited_at: row.get("edited_at")?,
            parent_comment_id: row.get("parent_comment_id")?,
            created_at: row.get("created_at")?,
        })
    }
}