use crate::db::resource::transxchange::operator_def::TxOperator;
use sqlx::PgPool;

pub static SQL_TX_OPERATOR: &str = r#"
CREATE TABLE IF NOT EXISTS tx_operators (
    id TEXT PRIMARY KEY,
    national_code TEXT,
    code TEXT,
    short_name TEXT NOT NULL,
    licence_number TEXT,
    comment_thread BIGINT REFERENCES comment_threads(id)
)
"#;

#[derive(Clone)]
pub struct TxOperators {
    pool: PgPool,
}

impl TxOperators {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: &str) -> sqlx::Result<Option<TxOperator>> {
        sqlx::query_as::<_, TxOperator>(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_or_create_comment_thread(&self, operator_id: &str) -> sqlx::Result<i64> {
        let mut tx = self.pool.begin().await?;

        // Lock the operator row to prevent concurrent thread creation.
        let existing: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT comment_thread
            FROM tx_operators
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(operator_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

        if let Some(thread_id) = existing {
            tx.commit().await?;
            return Ok(thread_id);
        }

        let now = chrono::Utc::now().to_rfc3339();

        let thread_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO comment_threads (created_at)
            VALUES ($1)
            RETURNING id
            "#,
        )
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE tx_operators
            SET comment_thread = $1
            WHERE id = $2
            "#,
        )
        .bind(thread_id)
        .bind(operator_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(thread_id)
    }

    pub async fn search(&self, query: &str) -> sqlx::Result<Vec<TxOperator>> {
        let search = format!("%{}%", query);

        sqlx::query_as::<_, TxOperator>(
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
                short_name ILIKE $1
                OR national_code ILIKE $1
                OR code ILIKE $1
                OR licence_number ILIKE $1
            ORDER BY short_name
            LIMIT 50
            "#,
        )
        .bind(search)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_by_national_code(
        &self,
        national_code: &str,
    ) -> sqlx::Result<Option<TxOperator>> {
        sqlx::query_as::<_, TxOperator>(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE national_code = $1
            "#,
        )
        .bind(national_code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_by_code(&self, code: &str) -> sqlx::Result<Option<TxOperator>> {
        sqlx::query_as::<_, TxOperator>(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_by_licence_number(
        &self,
        licence_number: &str,
    ) -> sqlx::Result<Option<TxOperator>> {
        sqlx::query_as::<_, TxOperator>(
            r#"
            SELECT
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            FROM tx_operators
            WHERE licence_number = $1
            "#,
        )
        .bind(licence_number)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, operator: &TxOperator) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tx_operators (
                id,
                national_code,
                code,
                short_name,
                licence_number
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&operator.id)
        .bind(&operator.national_code)
        .bind(&operator.code)
        .bind(&operator.short_name)
        .bind(&operator.licence_number)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, operator: &TxOperator) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tx_operators (
                id,
                national_code,
                code,
                short_name,
                licence_number,
                comment_thread
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(id) DO UPDATE SET
                national_code = EXCLUDED.national_code,
                code = EXCLUDED.code,
                short_name = EXCLUDED.short_name,
                licence_number = EXCLUDED.licence_number
            "#,
        )
        .bind(&operator.id)
        .bind(&operator.national_code)
        .bind(&operator.code)
        .bind(&operator.short_name)
        .bind(&operator.licence_number)
        .bind(operator.comment_thread)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM tx_operators WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
