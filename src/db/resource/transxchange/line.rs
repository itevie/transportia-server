use crate::db::resource::transxchange::line_def::TxLine;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use utoipa::{IntoParams, ToSchema};

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
    pool: PgPool,
}

impl TxLines {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, tx_id: &str) -> sqlx::Result<Option<TxLine>> {
        sqlx::query_as::<_, TxLine>(
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
            WHERE tx_id = $1
            "#,
        )
        .bind(tx_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn search(&self, query: &TxLineQuery) -> sqlx::Result<Vec<TxLine>> {
        let mut builder = QueryBuilder::<Postgres>::new(
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
            WHERE TRUE
            "#,
        );

        if let Some(value) = &query.tx_id {
            builder
                .push(" AND tx_id ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.name {
            builder
                .push(" AND name ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.operator_code {
            builder.push(" AND operator_code = ").push_bind(value);
        }

        if let Some(value) = &query.operator_national_code {
            builder
                .push(" AND operator_national_code = ")
                .push_bind(value);
        }

        if let Some(value) = &query.line_id {
            builder
                .push(" AND line_id ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.service_code {
            builder
                .push(" AND service_code ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.outbound_description {
            builder
                .push(" AND outbound_description ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.inbound_description {
            builder
                .push(" AND inbound_description ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.search {
            for word in value.split_whitespace() {
                let pattern = format!("%{}%", word);

                builder.push(
                    r#"
                    AND (
                        tx_id ILIKE "#,
                );
                builder.push_bind(pattern.clone());

                builder.push(" OR name ILIKE ");
                builder.push_bind(pattern.clone());

                builder.push(" OR operator_code ILIKE ");
                builder.push_bind(pattern.clone());

                builder.push(" OR operator_national_code ILIKE ");
                builder.push_bind(pattern.clone());

                builder.push(" OR line_id ILIKE ");
                builder.push_bind(pattern.clone());

                builder.push(" OR service_code ILIKE ");
                builder.push_bind(pattern.clone());

                builder.push(" OR outbound_description ILIKE ");
                builder.push_bind(pattern.clone());

                builder.push(" OR inbound_description ILIKE ");
                builder.push_bind(pattern);

                builder.push(")");
            }
        }

        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);
        let offset = query.offset.unwrap_or(0).max(0);

        builder
            .push(" ORDER BY name LIMIT ")
            .push_bind(page_length)
            .push(" OFFSET ")
            .push_bind(offset);

        builder
            .build_query_as::<TxLine>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_by_line_id(&self, line_id: &str) -> sqlx::Result<Option<TxLine>> {
        sqlx::query_as::<_, TxLine>(
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
            WHERE line_id = $1
            "#,
        )
        .bind(line_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_by_service_code(&self, service_code: &str) -> sqlx::Result<Vec<TxLine>> {
        sqlx::query_as::<_, TxLine>(
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
            WHERE service_code = $1
            ORDER BY name
            "#,
        )
        .bind(service_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_operator_code(&self, operator_code: &str) -> sqlx::Result<Vec<TxLine>> {
        sqlx::query_as::<_, TxLine>(
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
            WHERE operator_code = $1
            ORDER BY name
            "#,
        )
        .bind(operator_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_national_operator_code(
        &self,
        national_code: &str,
    ) -> sqlx::Result<Vec<TxLine>> {
        sqlx::query_as::<_, TxLine>(
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
            WHERE operator_national_code = $1
            ORDER BY name
            "#,
        )
        .bind(national_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, line: &TxLine) -> sqlx::Result<()> {
        sqlx::query(
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&line.tx_id)
        .bind(&line.name)
        .bind(&line.operator_code)
        .bind(&line.operator_national_code)
        .bind(&line.line_id)
        .bind(&line.service_code)
        .bind(&line.outbound_description)
        .bind(&line.inbound_description)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, line: &TxLine) -> sqlx::Result<()> {
        sqlx::query(
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(tx_id) DO UPDATE SET
                name = EXCLUDED.name,
                operator_code = EXCLUDED.operator_code,
                operator_national_code = EXCLUDED.operator_national_code,
                line_id = EXCLUDED.line_id,
                service_code = EXCLUDED.service_code,
                outbound_description = EXCLUDED.outbound_description,
                inbound_description = EXCLUDED.inbound_description
            "#,
        )
        .bind(&line.tx_id)
        .bind(&line.name)
        .bind(&line.operator_code)
        .bind(&line.operator_national_code)
        .bind(&line.line_id)
        .bind(&line.service_code)
        .bind(&line.outbound_description)
        .bind(&line.inbound_description)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_many(&self, lines: &[TxLine]) -> sqlx::Result<()> {
        let mut transaction = self.pool.begin().await?;

        for line in lines {
            sqlx::query(
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
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT(tx_id) DO UPDATE SET
                    name = EXCLUDED.name,
                    operator_code = EXCLUDED.operator_code,
                    operator_national_code = EXCLUDED.operator_national_code,
                    line_id = EXCLUDED.line_id,
                    service_code = EXCLUDED.service_code,
                    outbound_description = EXCLUDED.outbound_description,
                    inbound_description = EXCLUDED.inbound_description
                "#,
            )
            .bind(&line.tx_id)
            .bind(&line.name)
            .bind(&line.operator_code)
            .bind(&line.operator_national_code)
            .bind(&line.line_id)
            .bind(&line.service_code)
            .bind(&line.outbound_description)
            .bind(&line.inbound_description)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn delete(&self, tx_id: &str) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM tx_lines WHERE tx_id = $1")
            .bind(tx_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
