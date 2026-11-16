use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::Database;
use crate::db::resources::route_checklist_item::{RouteChecklistItem, RouteChecklistItemFilledOut};

pub static SQL_ROUTE_CHECKLIST_ITEMS: &str = r#"
CREATE TABLE IF NOT EXISTS route_checklist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    checklist_id INTEGER REFERENCES route_checklists(id) NOT NULL,
    line_id TEXT REFERENCES tx_lines(tx_id) NOT NULL,
    added_at TEXT NOT NULL,
    completed_at TEXT,

    UNIQUE(checklist_id, line_id)
)
"#;

#[derive(Clone)]
pub struct RouteChecklistItems {
    pool: Pool<SqliteConnectionManager>,
}

impl RouteChecklistItems {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<RouteChecklistItem>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                checklist_id,
                line_id,
                added_at,
                completed_at
            FROM route_checklist_items
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_route_checklist_item(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_checklist(
        &self,
        checklist_id: i64,
    ) -> rusqlite::Result<Vec<RouteChecklistItem>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                checklist_id,
                line_id,
                added_at,
                completed_at
            FROM route_checklist_items
            WHERE checklist_id = ?
            ORDER BY added_at ASC
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![checklist_id],
            Self::row_to_route_checklist_item,
        )?;

        rows.collect()
    }

    pub fn get_by_line(
        &self,
        checklist_id: i64,
        line_id: &str,
    ) -> rusqlite::Result<Option<RouteChecklistItem>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                checklist_id,
                line_id,
                added_at,
                completed_at
            FROM route_checklist_items
            WHERE checklist_id = ? AND line_id = ?
            "#,
        )?;

        let mut rows = statement.query(
            rusqlite::params![checklist_id, line_id],
        )?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_route_checklist_item(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(
        &self,
        checklist_id: i64,
        line_id: &str,
        added_at: &str,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT OR IGNORE INTO route_checklist_items (
                checklist_id,
                line_id,
                added_at
            )
            VALUES (?, ?, ?)
            "#,
            rusqlite::params![
                checklist_id,
                line_id,
                added_at,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(
        &self,
        item: &RouteChecklistItem,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE route_checklist_items SET
                checklist_id = ?,
                line_id = ?,
                added_at = ?,
                completed_at = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                item.checklist_id,
                &item.line_id,
                &item.added_at,
                &item.completed_at,
                item.id,
            ],
        )?;

        Ok(())
    }

    pub fn set_completed(
        &self,
        id: i64,
        completed_at: Option<&str>,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE route_checklist_items
            SET completed_at = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                completed_at,
                id,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM route_checklist_items WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(db: Arc<Database>, items: Vec<RouteChecklistItem>) -> rusqlite::Result<Vec<RouteChecklistItemFilledOut>> {
        items
            .into_iter()
            .map(|item| {
                let tx_line = db.tx_lines.get_by_line_id(&item.line_id)?;

                Ok(RouteChecklistItemFilledOut {
                    id: item.id,
                    checklist_id: item.checklist_id,
                    line_id: item.line_id,
                    line: tx_line.unwrap_or_default(),
                    added_at: item.added_at,
                    completed_at: item.completed_at,
                })
            })
            .collect()
    }

    fn row_to_route_checklist_item(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<RouteChecklistItem> {
        Ok(RouteChecklistItem {
            id: row.get("id")?,
            checklist_id: row.get("checklist_id")?,
            line_id: row.get("line_id")?,
            added_at: row.get("added_at")?,
            completed_at: row.get("completed_at")?,
        })
    }
}