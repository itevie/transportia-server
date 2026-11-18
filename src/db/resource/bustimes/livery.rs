
use sqlx::PgPool;
use crate::db::resource::bustimes::livery_def::BtLivery;

pub static SQL_BT_LIVERY: &str = r#"
CREATE TABLE IF NOT EXISTS bt_liveries (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    left_css TEXT NOT NULL,
    right_css TEXT NOT NULL,
    white_text BOOLEAN NOT NULL,
    text_colour TEXT NOT NULL,
    stroke_colour TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct BtLiveries {
    pool: PgPool,
}

impl BtLiveries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        id: i64,
    ) -> sqlx::Result<Option<BtLivery>> {
        sqlx::query_as::<_, BtLivery>(
            r#"
            SELECT *
            FROM bt_liveries
            WHERE id = $1
            "#
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn search(
        &self,
        query: &str,
    ) -> sqlx::Result<Vec<BtLivery>> {
        let search = format!("%{}%", query);

        sqlx::query_as::<_, BtLivery>(
            r#"
            SELECT *
            FROM bt_liveries
            WHERE name ILIKE $1
            ORDER BY name
            LIMIT 50
            "#
        )
            .bind(search)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_by_name(
        &self,
        name: &str,
    ) -> sqlx::Result<Option<BtLivery>> {
        sqlx::query_as::<_, BtLivery>(
            r#"
            SELECT *
            FROM bt_liveries
            WHERE name = $1
            "#
        )
            .bind(name)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(
        &self,
        livery: &BtLivery,
    ) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bt_liveries (
                id,
                name,
                left_css,
                right_css,
                white_text,
                text_colour,
                stroke_colour
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
            .bind(livery.id)
            .bind(&livery.name)
            .bind(&livery.left_css)
            .bind(&livery.right_css)
            .bind(livery.white_text)
            .bind(&livery.text_colour)
            .bind(&livery.stroke_colour)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn upsert(
        &self,
        livery: &BtLivery,
    ) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bt_liveries (
                id,
                name,
                left_css,
                right_css,
                white_text,
                text_colour,
                stroke_colour
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(id) DO UPDATE SET
                name = EXCLUDED.name,
                left_css = EXCLUDED.left_css,
                right_css = EXCLUDED.right_css,
                white_text = EXCLUDED.white_text,
                text_colour = EXCLUDED.text_colour,
                stroke_colour = EXCLUDED.stroke_colour
            "#
        )
            .bind(livery.id)
            .bind(&livery.name)
            .bind(&livery.left_css)
            .bind(&livery.right_css)
            .bind(livery.white_text)
            .bind(&livery.text_colour)
            .bind(&livery.stroke_colour)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> sqlx::Result<()> {
        sqlx::query(
            "DELETE FROM bt_liveries WHERE id = $1"
        )
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
