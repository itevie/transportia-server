use serde::{Deserialize, Serialize};
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use sqlx::{PgPool, Postgres, QueryBuilder};
use utoipa::{IntoParams, ToSchema};
use crate::db::resource::bustimes::vehicle_def::BtVehicle;
use crate::db::util::search_pro_max::{search_query, SearchQuery};

pub static SQL_BT_VEHICLE_TYPE: &str = r#"
CREATE TABLE IF NOT EXISTS bt_vehicle_types (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    style TEXT NOT NULL,
    fuel TEXT NOT NULL,
    double_decker BOOLEAN NOT NULL,
    coach BOOLEAN NOT NULL,
    electric BOOLEAN NOT NULL
)
"#;

#[derive(Clone)]
pub struct BtVehicleTypes {
    pool: PgPool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct VehicleTypeQuery {
    pub search: Option<String>,

    pub operator_id: Option<String>,

    pub double_decker: Option<bool>,
    pub coach: Option<bool>,
    pub electric: Option<bool>,

    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl BtVehicleTypes {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: i64) -> sqlx::Result<Option<BtVehicleType>> {
        sqlx::query_as::<_, BtVehicleType>(
            r#"
            SELECT *
            FROM bt_vehicle_types
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn search(&self, query: &VehicleTypeQuery) -> sqlx::Result<Vec<BtVehicleType>> {
        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);

        let mut search = SearchQuery::new(
            "*",
            r#"
                FROM bt_vehicle_types
                WHERE TRUE
            "#
        );

        let operator_id = query.operator_id.clone();

        let search = search
            .exists_b(
                operator_id.is_some(),
                "SELECT 1 FROM bt_vehicles v WHERE v.vehicle_type_id = bt_vehicle_types.id AND v.operator_id = ",
                move |builder| {
                    builder.push_bind(operator_id.clone());
                },
            )
            .eq("double_decker", query.double_decker)
            .eq("electric", query.electric)
            .eq("coach", query.coach)
            .search(
                query.search.as_deref(),
                ["name"],
            )
            .limit(page_length)
            .offset(query.offset);

        let mut builder = QueryBuilder::<Postgres>::new("");

        search_query(&mut builder, &search)?;

        println!("{:?}", builder.sql().as_str());

        builder
            .build_query_as::<BtVehicleType>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                println!("SQLx error: {e:#?}");
                e
            })
    }

    pub async fn get_by_name(&self, name: &str) -> sqlx::Result<Option<BtVehicleType>> {
        sqlx::query_as::<_, BtVehicleType>(
            r#"
            SELECT *
            FROM bt_vehicle_types
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, vehicle_type: &BtVehicleType) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bt_vehicle_types (
                id,
                name,
                style,
                fuel,
                double_decker,
                coach,
                electric
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(vehicle_type.id)
        .bind(&vehicle_type.name)
        .bind(&vehicle_type.style)
        .bind(&vehicle_type.fuel)
        .bind(vehicle_type.double_decker)
        .bind(vehicle_type.coach)
        .bind(vehicle_type.electric)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, vehicle_type: &BtVehicleType) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bt_vehicle_types (
                id,
                name,
                style,
                fuel,
                double_decker,
                coach,
                electric
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(id) DO UPDATE SET
                name = EXCLUDED.name,
                style = EXCLUDED.style,
                fuel = EXCLUDED.fuel,
                double_decker = EXCLUDED.double_decker,
                coach = EXCLUDED.coach,
                electric = EXCLUDED.electric
            "#,
        )
        .bind(vehicle_type.id)
        .bind(&vehicle_type.name)
        .bind(&vehicle_type.style)
        .bind(&vehicle_type.fuel)
        .bind(vehicle_type.double_decker)
        .bind(vehicle_type.coach)
        .bind(vehicle_type.electric)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM bt_vehicle_types WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
