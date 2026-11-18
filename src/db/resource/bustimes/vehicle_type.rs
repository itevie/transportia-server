
use sqlx::PgPool;
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;

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

impl BtVehicleTypes {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(
        &self,
        id: i64,
    ) -> sqlx::Result<Option<BtVehicleType>> {
        sqlx::query_as::<_, BtVehicleType>(
            r#"
            SELECT *
            FROM bt_vehicle_types
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
    ) -> sqlx::Result<Vec<BtVehicleType>> {
        let search = format!("%{}%", query);

        sqlx::query_as::<_, BtVehicleType>(
            r#"
            SELECT *
            FROM bt_vehicle_types
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
    ) -> sqlx::Result<Option<BtVehicleType>> {
        sqlx::query_as::<_, BtVehicleType>(
            r#"
            SELECT *
            FROM bt_vehicle_types
            WHERE name = $1
            "#
        )
            .bind(name)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create(
        &self,
        vehicle_type: &BtVehicleType,
    ) -> sqlx::Result<()> {
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
            "#
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

    pub async fn upsert(
        &self,
        vehicle_type: &BtVehicleType,
    ) -> sqlx::Result<()> {
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
            "#
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
        sqlx::query(
            "DELETE FROM bt_vehicle_types WHERE id = $1"
        )
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
