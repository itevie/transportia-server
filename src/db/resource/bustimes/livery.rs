use serde::{Deserialize, Serialize};
use crate::db::resource::bustimes::livery_def::BtLivery;
use sqlx::{PgPool, Postgres, QueryBuilder};
use utoipa::{IntoParams, ToSchema};
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use crate::db::util::search_pro_max::{search_query, SearchQuery};

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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct LiveryQuery {
    pub search: Option<String>,

    pub operator_id: Option<String>,
    
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl BtLiveries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: i64) -> sqlx::Result<Option<BtLivery>> {
        sqlx::query_as::<_, BtLivery>(
            r#"
            SELECT *
            FROM bt_liveries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn search(&self, query: &LiveryQuery) -> sqlx::Result<Vec<BtLivery>> {
        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);

        let mut search = SearchQuery::new(
            "*",
            r#"
                FROM bt_liveries
                WHERE TRUE
            "#
        );

        let operator_id = query.operator_id.clone();

        let search = search
            .exists_b(
                operator_id.is_some(),
                "SELECT 1 FROM bt_vehicles v WHERE v.livery_id = bt_liveries.id AND v.operator_id = ",
                move |builder| {
                    builder.push_bind(operator_id.clone());
                },
            )
            .search(
                query.search.as_deref(),
                ["name"],
            )
            .limit(page_length)
            .offset(query.offset);

        let mut builder = QueryBuilder::<Postgres>::new("");

        search_query(&mut builder, &search)?;

        builder
            .build_query_as::<BtLivery>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                println!("SQLx error: {e:#?}");
                e
            })
    }

    pub async fn get_by_name(&self, name: &str) -> sqlx::Result<Option<BtLivery>> {
        sqlx::query_as::<_, BtLivery>(
            r#"
            SELECT *
            FROM bt_liveries
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, livery: &BtLivery) -> sqlx::Result<()> {
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
            "#,
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

    pub async fn upsert(&self, livery: &BtLivery) -> sqlx::Result<()> {
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
            "#,
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
        sqlx::query("DELETE FROM bt_liveries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
