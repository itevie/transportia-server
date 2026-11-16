use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::db::resources::tx_operator::TxOperator;

pub static SQL_TX_OPERATOR: &str = r#"
CREATE TABLE IF NOT EXISTS tx_operators (
    id TEXT PRIMARY KEY,
    national_code TEXT,
    code TEXT,
    short_name TEXT NOT NULL,
    licence_number TEXT,
    comment_thread INTEGER REFERENCES comment_threads(id)
)
"#;

#[derive(Clone)]
pub struct TxOperators {
    pool: Pool<SqliteConnectionManager>,
}

impl TxOperators {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: &str) -> rusqlite::Result<Option<TxOperator>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_operator(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_or_create_comment_thread(
        &self,
        comment_id: i64,
    ) -> rusqlite::Result<i64> {
        let mut connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let tx = connection.transaction()?;

        let existing: Option<i64> = tx
            .query_row(
                "SELECT comment_thread FROM tx_operators WHERE id = ?1",
                [comment_id],
                |row| row.get(0),
            )?;

        if let Some(thread_id) = existing {
            tx.commit()?;
            return Ok(thread_id);
        }

        let now = chrono::Utc::now().to_rfc3339();

        tx.execute(
            "INSERT INTO comment_threads (created_at) VALUES (?1)",
            [&now],
        )?;

        let thread_id = tx.last_insert_rowid();

        let updated = tx.execute(
            "UPDATE tx_operators
         SET comment_thread = ?1
         WHERE id = ?2 AND comment_thread IS NULL",
            rusqlite::params![thread_id, comment_id],
        )?;

        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        tx.commit()?;

        Ok(thread_id)
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<TxOperator>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE
                short_name LIKE ?
                OR national_code LIKE ?
                OR code LIKE ?
                OR licence_number LIKE ?
            ORDER BY short_name
            LIMIT 50
            "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![
                search,
                search,
                search,
                search,
            ],
            Self::row_to_tx_operator,
        )?;

        rows.collect()
    }

    pub fn get_by_national_code(
        &self,
        national_code: &str,
    ) -> rusqlite::Result<Option<TxOperator>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE national_code = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![national_code])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_operator(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_code(
        &self,
        code: &str,
    ) -> rusqlite::Result<Option<TxOperator>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE code = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![code])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_operator(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_licence_number(
        &self,
        licence_number: &str,
    ) -> rusqlite::Result<Option<TxOperator>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE licence_number = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![licence_number])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_operator(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(&self, operator: &TxOperator) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_operators (
                id,
                national_code,
                code,
                short_name,
                licence_number
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &operator.id,
                &operator.national_code,
                &operator.code,
                &operator.short_name,
                &operator.licence_number,
            ],
        )?;

        Ok(())
    }

    pub fn upsert(&self, operator: &TxOperator) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_operators (
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            )
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                national_code = excluded.national_code,
                code = excluded.code,
                short_name = excluded.short_name,
                licence_number = excluded.licence_number
            "#,
            rusqlite::params![
                &operator.id,
                &operator.national_code,
                &operator.code,
                &operator.short_name,
                &operator.licence_number,
                &operator.comment_thread
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: &str) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM tx_operators WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_tx_operator(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TxOperator> {
        Ok(TxOperator {
            id: row.get("id")?,
            national_code: row.get("national_code")?,
            code: row.get("code")?,
            short_name: row.get("short_name")?,
            licence_number: row.get("licence_number")?,
            comment_thread: row.get("comment_thread")?,
        })
    }
}