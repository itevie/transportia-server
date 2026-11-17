
use std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

pub async fn init_db() -> Result<Database, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url).await?;

    let db = Database {
        pool: pool.clone(),
    };

    Ok(db)
}

pub async fn create_missing_comment_threads(
    db: Arc<Database>,
    table_name: &str,
) -> Result<u64, sqlx::Error> {
    const ALLOWED_TABLES: &[&str] = &[
        "tx_operators",
        "photos",
        "tx_routes",
        "bt_vehicles",
    ];

    if !ALLOWED_TABLES.contains(&table_name) {
        return Err(sqlx::Error::Protocol(
            format!("Invalid table name: {table_name}"),
        ));
    }

    let mut tx = db.pool.begin().await?;

    let sql = format!(
        r#"SELECT id FROM "{table_name}" WHERE comment_thread IS NULL"#
    );

    let ids: Vec<i64> = sqlx::query_scalar(&sql)
        .fetch_all(&mut *tx)
        .await?;

    let mut created = 0;

    for id in ids {
        let thread_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO comment_threads (created_at)
            VALUES (NOW())
            RETURNING id
            "#
        )
            .fetch_one(&mut *tx)
            .await?;

        let sql = format!(
            r#"
            UPDATE "{table_name}"
            SET comment_thread = $1
            WHERE id = $2 AND comment_thread IS NULL
            "#
        );

        created += sqlx::query(&sql)
            .bind(thread_id)
            .bind(id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    }

    tx.commit().await?;

    Ok(created)
}
