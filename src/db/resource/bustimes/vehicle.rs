use std::sync::Arc;

use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use utoipa::{IntoParams, ToSchema};

use crate::db::Database;
use crate::db::resource::bustimes::vehicle_def::{BtVehicle, FilledOutBtVehicle};
use crate::db::resource::favourite::favourite_def::FavouriteType;
use crate::db::resource::photo::photo::Photos;

pub static SQL_BT_VEHICLE: &str = r#"
CREATE TABLE IF NOT EXISTS bt_vehicles (
    id BIGINT PRIMARY KEY,
    slug TEXT NOT NULL,
    fleet TEXT NOT NULL,
    reg TEXT NOT NULL,
    previous_reg TEXT NOT NULL,
    vehicle_type_id BIGINT REFERENCES bt_vehicle_types(id),
    livery_id BIGINT REFERENCES bt_liveries(id),
    branding TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    garage_id TEXT,
    name TEXT NOT NULL,
    notes TEXT NOT NULL,
    withdrawn BOOLEAN NOT NULL,
    training_vehicle BOOLEAN NOT NULL DEFAULT FALSE,
    open_top BOOLEAN NOT NULL DEFAULT FALSE,
    demonstrator BOOLEAN NOT NULL DEFAULT FALSE,
    comment_thread BIGINT REFERENCES comment_threads(id)
)
"#;

#[derive(Clone)]
pub struct BtVehicles {
    pool: PgPool,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct BtVehicleQuery {
    pub search: Option<String>,

    pub slug: Option<String>,
    pub fleet: Option<String>,
    pub reg: Option<String>,
    pub previous_reg: Option<String>,

    pub vehicle_type_id: Option<i64>,
    pub livery_id: Option<i64>,
    pub operator_id: Option<String>,

    pub name: Option<String>,
    pub withdrawn: Option<bool>,

    pub electric: Option<bool>,
    pub double_decker: Option<bool>,
    pub coach: Option<bool>,

    pub training_vehicle: Option<bool>,
    pub demonstrator: Option<bool>,
    pub open_top: Option<bool>,

    pub has_image: Option<bool>,

    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl BtVehicles {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: i64) -> sqlx::Result<Option<BtVehicle>> {
        sqlx::query_as::<_, BtVehicle>(
            r#"
            SELECT *
            FROM bt_vehicles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn search(&self, query: &BtVehicleQuery) -> sqlx::Result<Vec<BtVehicle>> {
        let page_length = query.page_length.unwrap_or(50).min(1000);

        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT v.*
            FROM bt_vehicles v
            LEFT JOIN bt_vehicle_types vt
                ON vt.id = v.vehicle_type_id
            LEFT JOIN tx_operators o
                ON o.id = v.operator_id
            WHERE TRUE
            "#,
        );

        if let Some(value) = query.slug.as_deref() {
            builder
                .push(" AND v.slug ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = query.fleet.as_deref() {
            builder
                .push(" AND v.fleet ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = query.reg.as_deref() {
            builder
                .push(" AND v.reg ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = query.previous_reg.as_deref() {
            builder
                .push(" AND v.previous_reg ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = query.vehicle_type_id {
            builder.push(" AND v.vehicle_type_id = ").push_bind(value);
        }

        if let Some(value) = query.livery_id {
            builder.push(" AND v.livery_id = ").push_bind(value);
        }

        if let Some(value) = query.operator_id.as_deref() {
            builder.push(" AND v.operator_id = ").push_bind(value);
        }

        if let Some(value) = query.name.as_deref() {
            builder
                .push(" AND v.name ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = query.withdrawn {
            builder.push(" AND v.withdrawn = ").push_bind(value);
        }

        if let Some(value) = query.electric {
            builder.push(" AND vt.electric = ").push_bind(value);
        }

        if let Some(value) = query.double_decker {
            builder.push(" AND vt.double_decker = ").push_bind(value);
        }

        if let Some(value) = query.coach {
            builder.push(" AND vt.coach = ").push_bind(value);
        }

        if let Some(value) = query.training_vehicle {
            builder.push(" AND v.training_vehicle = ").push_bind(value);
        }

        if let Some(value) = query.demonstrator {
            builder.push(" AND v.demonstrator = ").push_bind(value);
        }

        if let Some(value) = query.open_top {
            builder.push(" AND v.open_top = ").push_bind(value);
        }

        if let Some(value) = query.has_image {
            if value {
                builder.push(
                    r#"
                    AND EXISTS (
                        SELECT 1
                        FROM bus_photo_details bpd
                        WHERE bpd.matched_vehicle_id = v.id
                    )
                    "#,
                );
            } else {
                builder.push(
                    r#"
                    AND NOT EXISTS (
                        SELECT 1
                        FROM bus_photo_details bpd
                        WHERE bpd.matched_vehicle_id = v.id
                    )
                    "#,
                );
            }
        }

        if let Some(value) = query.search.as_deref() {
            let search = format!("%{}%", value);

            builder
                .push(
                    r#"
                AND (
                    v.slug ILIKE
                "#,
                )
                .push_bind(search.clone());

            builder.push(" OR v.fleet ILIKE ").push_bind(search.clone());

            builder.push(" OR v.reg ILIKE ").push_bind(search.clone());

            builder
                .push(" OR v.previous_reg ILIKE ")
                .push_bind(search.clone());

            builder.push(" OR v.name ILIKE ").push_bind(search.clone());

            builder
                .push(" OR o.short_name ILIKE ")
                .push_bind(search.clone());

            builder.push(" OR o.code ILIKE ").push_bind(search.clone());

            builder.push(" OR o.national_code ILIKE ").push_bind(search);

            builder.push(")");
        }

        builder.push(" ORDER BY v.name");

        builder.push(" LIMIT ").push_bind(page_length);

        if let Some(offset) = query.offset {
            builder.push(" OFFSET ").push_bind(offset);
        }

        builder
            .build_query_as::<BtVehicle>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create(&self, vehicle: &BtVehicle) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bt_vehicles (
                id,
                slug,
                fleet,
                reg,
                previous_reg,
                vehicle_type_id,
                livery_id,
                branding,
                operator_id,
                garage_id,
                name,
                notes,
                withdrawn,
                training_vehicle,
                demonstrator,
                open_top,
                comment_thread
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15, $16, $17
            )
            "#,
        )
        .bind(vehicle.id)
        .bind(&vehicle.slug)
        .bind(&vehicle.fleet)
        .bind(&vehicle.reg)
        .bind(&vehicle.previous_reg)
        .bind(vehicle.vehicle_type_id)
        .bind(vehicle.livery_id)
        .bind(&vehicle.branding)
        .bind(&vehicle.operator_id)
        .bind(&vehicle.garage_id)
        .bind(&vehicle.name)
        .bind(&vehicle.notes)
        .bind(vehicle.withdrawn)
        .bind(vehicle.training_vehicle)
        .bind(vehicle.demonstrator)
        .bind(vehicle.open_top)
        .bind(vehicle.comment_thread)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, vehicle: &BtVehicle) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bt_vehicles (
                id,
                slug,
                fleet,
                reg,
                previous_reg,
                vehicle_type_id,
                livery_id,
                branding,
                operator_id,
                garage_id,
                name,
                notes,
                withdrawn,
                training_vehicle,
                demonstrator,
                open_top,
                comment_thread
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15, $16, $17
            )
            ON CONFLICT(id) DO UPDATE SET
                slug = EXCLUDED.slug,
                fleet = EXCLUDED.fleet,
                reg = EXCLUDED.reg,
                previous_reg = EXCLUDED.previous_reg,
                vehicle_type_id = EXCLUDED.vehicle_type_id,
                livery_id = EXCLUDED.livery_id,
                branding = EXCLUDED.branding,
                operator_id = EXCLUDED.operator_id,
                garage_id = EXCLUDED.garage_id,
                name = EXCLUDED.name,
                notes = EXCLUDED.notes,
                withdrawn = EXCLUDED.withdrawn,
                training_vehicle = EXCLUDED.training_vehicle,
                demonstrator = EXCLUDED.demonstrator,
                open_top = EXCLUDED.open_top,
                comment_thread = EXCLUDED.comment_thread
            "#,
        )
        .bind(vehicle.id)
        .bind(&vehicle.slug)
        .bind(&vehicle.fleet)
        .bind(&vehicle.reg)
        .bind(&vehicle.previous_reg)
        .bind(vehicle.vehicle_type_id)
        .bind(vehicle.livery_id)
        .bind(&vehicle.branding)
        .bind(&vehicle.operator_id)
        .bind(&vehicle.garage_id)
        .bind(&vehicle.name)
        .bind(&vehicle.notes)
        .bind(vehicle.withdrawn)
        .bind(vehicle.training_vehicle)
        .bind(vehicle.demonstrator)
        .bind(vehicle.open_top)
        .bind(vehicle.comment_thread)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Upserts only BusTimes data, preserving Transportia-specific fields.
    pub async fn upsert_bustimes(&self, vehicle: &BtVehicle) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO bt_vehicles (
                id,
                slug,
                fleet,
                reg,
                previous_reg,
                vehicle_type_id,
                livery_id,
                branding,
                operator_id,
                garage_id,
                name,
                notes,
                withdrawn
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10, $11, $12, $13
            )
            ON CONFLICT(id) DO UPDATE SET
                slug = EXCLUDED.slug,
                fleet = EXCLUDED.fleet,
                reg = EXCLUDED.reg,
                previous_reg = EXCLUDED.previous_reg,
                vehicle_type_id = EXCLUDED.vehicle_type_id,
                livery_id = EXCLUDED.livery_id,
                branding = EXCLUDED.branding,
                operator_id = EXCLUDED.operator_id,
                garage_id = EXCLUDED.garage_id,
                name = EXCLUDED.name,
                notes = EXCLUDED.notes,
                withdrawn = EXCLUDED.withdrawn
            "#,
        )
        .bind(vehicle.id)
        .bind(&vehicle.slug)
        .bind(&vehicle.fleet)
        .bind(&vehicle.reg)
        .bind(&vehicle.previous_reg)
        .bind(vehicle.vehicle_type_id)
        .bind(vehicle.livery_id)
        .bind(&vehicle.branding)
        .bind(&vehicle.operator_id)
        .bind(&vehicle.garage_id)
        .bind(&vehicle.name)
        .bind(&vehicle.notes)
        .bind(vehicle.withdrawn)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM bt_vehicles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn fill_out(
        db: Arc<Database>,
        vehicles: Vec<BtVehicle>,
        user_id: Option<i64>,
    ) -> sqlx::Result<Vec<FilledOutBtVehicle>> {
        let mut filled_vehicles = Vec::with_capacity(vehicles.len());

        for vehicle in vehicles {
            let vehicle_type = match vehicle.vehicle_type_id {
                Some(id) => db.vehicle_types.get(id).await?,
                None => None,
            };

            let livery = match vehicle.livery_id {
                Some(id) => db.liveries.get(id).await?,
                None => None,
            };

            let operator = db.tx_operators.search(&vehicle.operator_id).await?;

            let op = operator.into_iter().next();

            let favourite = match user_id {
                Some(user) => {
                    db.favourites
                        .get(user, FavouriteType::Vehicle, vehicle.id)
                        .await?
                }
                None => None,
            };

            let photos = db
                .photos
                .search(&Photos::query_only_search(vehicle.reg.clone()))
                .await?;

            filled_vehicles.push(FilledOutBtVehicle {
                id: vehicle.id,
                slug: vehicle.slug,
                fleet: vehicle.fleet,
                reg: vehicle.reg,
                previous_reg: vehicle.previous_reg,
                vehicle_type_id: vehicle.vehicle_type_id,
                vehicle_type,
                livery_id: vehicle.livery_id,
                livery,
                branding: vehicle.branding,
                operator_id: vehicle.operator_id,
                operator: op,
                garage_id: vehicle.garage_id,
                name: vehicle.name,
                notes: vehicle.notes,
                withdrawn: vehicle.withdrawn,
                training_vehicle: vehicle.training_vehicle,
                demonstrator: vehicle.demonstrator,
                open_top: vehicle.open_top,
                photo: photos.first().cloned(),
                is_favourited: user_id.map(|_| favourite.is_some()),
                comment_thread: vehicle.comment_thread,
            });
        }

        Ok(filled_vehicles)
    }
}
