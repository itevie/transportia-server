use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::resources::tx_line::TxLine;

pub static SQL_TX_LINE: &str = r#"
CREATE TABLE IF NOT EXISTS tx_lines (
    tx_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    operator_code TEXT,
    operator_national_code TEXT,
    line_id TEXT NOT NULL,
    service_code TEXT NOT NULL,
    outbound_description TEXT,
    inbound_description TEXT
)
"#;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TxLineQuery {
    pub search: Option<String>,

    pub tx_id: Option<String>,
    pub name: Option<String>,

    pub operator_code: Option<String>,
    pub operator_national_code: Option<String>,

    pub line_id: Option<String>,
    pub service_code: Option<String>,

    pub outbound_description: Option<String>,
    pub inbound_description: Option<String>,

    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone)]
pub struct TxLines {
    pool: Pool<SqliteConnectionManager>,
}

impl TxLines {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, tx_id: &str) -> rusqlite::Result<Option<TxLine>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            FROM tx_lines
            WHERE tx_id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![tx_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_line(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(
        &self,
        query: &TxLineQuery,
    ) -> rusqlite::Result<Vec<TxLine>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;
        
        let page_length = match query.page_length {
            Some(x) => if x > 1000 { 1000 } else { x },
            None => 50,
        };

        let mut sql = String::from(
            r#"
        SELECT
            tx_id,
            name,
            operator_code,
            operator_national_code,
            line_id,
            service_code,
            outbound_description,
            inbound_description
        FROM tx_lines
        WHERE 1 = 1
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = &query.tx_id {
            sql.push_str(" AND tx_id LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.name {
            sql.push_str(" AND name LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.operator_code {
            sql.push_str(" AND operator_code = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.operator_national_code {
            sql.push_str(" AND operator_national_code = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.line_id {
            sql.push_str(" AND line_id LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.service_code {
            sql.push_str(" AND service_code LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.outbound_description {
            sql.push_str(" AND outbound_description LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.inbound_description {
            sql.push_str(" AND inbound_description LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.search {
            let words: Vec<&str> = value.split_whitespace().collect();

            for word in words {
                sql.push_str(
                    r#"
            AND (
                tx_id LIKE '%' || ? || '%' OR
                name LIKE '%' || ? || '%' OR
                operator_code LIKE '%' || ? || '%' OR
                operator_national_code LIKE '%' || ? || '%' OR
                line_id LIKE '%' || ? || '%' OR
                service_code LIKE '%' || ? || '%' OR
                outbound_description LIKE '%' || ? || '%' OR
                inbound_description LIKE '%' || ? || '%'
            )
            "#,
                );

                for _ in 0..8 {
                    params.push(Box::new(word.to_string()));
                }
            }
        }

        sql.push_str(" ORDER BY name");

        sql.push_str(" LIMIT ?");
        params.push(Box::new(page_length));

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset));
        }

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_tx_line,
            )?
            .collect()
    }

    pub fn get_by_line_id(
        &self,
        line_id: &str,
    ) -> rusqlite::Result<Option<TxLine>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            FROM tx_lines
            WHERE line_id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![line_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_line(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_service_code(
        &self,
        service_code: &str,
    ) -> rusqlite::Result<Vec<TxLine>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            FROM tx_lines
            WHERE service_code = ?
            ORDER BY name
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![service_code],
            Self::row_to_tx_line,
        )?;

        rows.collect()
    }

    pub fn get_by_operator_code(
        &self,
        operator_code: &str,
    ) -> rusqlite::Result<Vec<TxLine>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            FROM tx_lines
            WHERE operator_code = ?
            ORDER BY name
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![operator_code],
            Self::row_to_tx_line,
        )?;

        rows.collect()
    }

    pub fn get_by_national_operator_code(
        &self,
        national_code: &str,
    ) -> rusqlite::Result<Vec<TxLine>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            FROM tx_lines
            WHERE operator_national_code = ?
            ORDER BY name
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![national_code],
            Self::row_to_tx_line,
        )?;

        rows.collect()
    }

    pub fn create(&self, line: &TxLine) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_lines (
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &line.tx_id,
                &line.name,
                &line.operator_code,
                &line.operator_national_code,
                &line.line_id,
                &line.service_code,
                &line.outbound_description,
                &line.inbound_description,
            ],
        )?;

        Ok(())
    }

    pub fn upsert(&self, line: &TxLine) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_lines (
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(tx_id) DO UPDATE SET
                name = excluded.name,
                operator_code = excluded.operator_code,
                operator_national_code = excluded.operator_national_code,
                line_id = excluded.line_id,
                service_code = excluded.service_code,
                outbound_description = excluded.outbound_description,
                inbound_description = excluded.inbound_description
            "#,
            rusqlite::params![
                &line.tx_id,
                &line.name,
                &line.operator_code,
                &line.operator_national_code,
                &line.line_id,
                &line.service_code,
                &line.outbound_description,
                &line.inbound_description,
            ],
        )?;

        Ok(())
    }

    pub fn upsert_many(
        &self,
        lines: &[TxLine],
    ) -> rusqlite::Result<()> {
        let mut connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let transaction = connection.transaction()?;

        {
            let mut statement = transaction.prepare(
                r#"
            INSERT INTO tx_lines (
                tx_id,
                name,
                operator_code,
                operator_national_code,
                line_id,
                service_code,
                outbound_description,
                inbound_description
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(tx_id) DO UPDATE SET
                name = excluded.name,
                operator_code = excluded.operator_code,
                operator_national_code = excluded.operator_national_code,
                line_id = excluded.line_id,
                service_code = excluded.service_code,
                outbound_description = excluded.outbound_description,
                inbound_description = excluded.inbound_description
            "#,
            )?;

            for line in lines {
                statement.execute(rusqlite::params![
                &line.tx_id,
                &line.name,
                &line.operator_code,
                &line.operator_national_code,
                &line.line_id,
                &line.service_code,
                &line.outbound_description,
                &line.inbound_description,
            ])?;
            }
        }

        transaction.commit()?;

        Ok(())
    }

    pub fn delete(&self, tx_id: &str) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM tx_lines WHERE tx_id = ?",
            rusqlite::params![tx_id],
        )?;

        Ok(())
    }

    fn row_to_tx_line(row: &rusqlite::Row) -> rusqlite::Result<TxLine> {
        Ok(TxLine {
            tx_id: row.get("tx_id")?,
            name: row.get("name")?,
            operator_code: row.get("operator_code")?,
            operator_national_code: row.get("operator_national_code")?,
            line_id: row.get("line_id")?,
            service_code: row.get("service_code")?,
            outbound_description: row.get("outbound_description")?,
            inbound_description: row.get("inbound_description")?,
        })
    }
}