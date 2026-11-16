use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::Database;
use crate::db::resources::route_checklist::{RouteChecklist, RouteChecklistFilledOut};
use crate::db::route_checklist_items::RouteChecklistItems;

pub static SQL_ROUTE_CHECKLISTS: &str = r#"
CREATE TABLE IF NOT EXISTS route_checklists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    owner_id INTEGER REFERENCES users(id) NOT NULL,
    created_at TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct RouteChecklists {
    pool: Pool<SqliteConnectionManager>,
}

impl RouteChecklists {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<RouteChecklist>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                title,
                description,
                owner_id,
                created_at
            FROM route_checklists
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_route_checklist(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn count_items(&self, checklist_id: i64) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.query_row(
            "SELECT COUNT(*) FROM route_checklist_items WHERE checklist_id = ?",
            rusqlite::params![checklist_id],
            |row| row.get(0),
        )
    }

    pub fn count_completed_items(&self, checklist_id: i64) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.query_row(
            "SELECT COUNT(*) FROM route_checklist_items WHERE checklist_id = ? AND completed_at IS NOT NULL",
            rusqlite::params![checklist_id],
            |row| row.get(0),
        )
    }

    pub fn get_all(&self, owner_id: i64) -> rusqlite::Result<Vec<RouteChecklist>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
        SELECT
            id,
            title,
            description,
            owner_id,
            created_at
        FROM route_checklists
        WHERE owner_id = ?
        ORDER BY id
        "#,
        )?;

        let rows = statement.query_map([&owner_id], |row| Self::row_to_route_checklist(row))?;

        rows.collect()
    }

    pub fn get_by_owner(
        &self,
        owner_id: i64,
    ) -> rusqlite::Result<Vec<RouteChecklist>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                title,
                description,
                owner_id,
                created_at
            FROM route_checklists
            WHERE owner_id = ?
            ORDER BY created_at DESC
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![owner_id],
            Self::row_to_route_checklist,
        )?;

        rows.collect()
    }

    pub fn create(
        &self,
        title: &str,
        description: Option<&str>,
        owner_id: i64,
        created_at: &str,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO route_checklists (
                title,
                description,
                owner_id,
                created_at
            )
            VALUES (?, ?, ?, ?)
            "#,
            rusqlite::params![
                title,
                description,
                owner_id,
                created_at,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, checklist: &RouteChecklist) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE route_checklists SET
                title = ?,
                description = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &checklist.title,
                &checklist.description,
                checklist.id,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM route_checklists WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }
    
    pub fn fill_out(db: Arc<Database>, checklists: Vec<RouteChecklist>) -> rusqlite::Result<Vec<RouteChecklistFilledOut>> {
        checklists
            .into_iter()
            .map(|checklist| {
                let count = db.route_checklists.count_items(checklist.id)?;
                let completed_count = db.route_checklists.count_completed_items(checklist.id)?;

                Ok(RouteChecklistFilledOut {
                    id: checklist.id,
                    title: checklist.title,
                    description: checklist.description,
                    owner_id: checklist.owner_id,
                    item_count: count,
                    created_at: checklist.created_at,
                    completed_item_count: completed_count
                })
            })
            .collect()
    }

    fn row_to_route_checklist(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<RouteChecklist> {
        Ok(RouteChecklist {
            id: row.get("id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            owner_id: row.get("owner_id")?,
            created_at: row.get("created_at")?,
        })
    }
}