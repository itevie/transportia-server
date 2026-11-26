use std::sync::Arc;

use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use utoipa::{IntoParams, ToSchema};

use crate::db::Database;
use crate::db::resource::bustimes::vehicle_def::{BtVehicle, BtVehicleFilleOut, BtVehicleSearchResult};
use crate::db::resource::favourite::favourite_def::FavouriteType;
use crate::db::resource::photo::photo::Photos;
use crate::db::resource::transxchange::line_def::TxLine;
use crate::{distinct_on, filter_bool, filter_eq, filter_ilike, filter_search, search_query};
use crate::db::util::pagination::PaginatedResponse;
use crate::db::util::search_pro_max::{search_query, SearchQuery};
use crate::db::util::unique_macro::ALLOWED_UNIQUE_FILTERS;

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
    pub unique: Option<String>
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

    pub async fn count(&self, id: i64) -> sqlx::Result<Option<BtVehicle>> {
        sqlx::query_as::<_, BtVehicle>(
            r#"
            SELECT COUNT(*)
            FROM bt_vehicles
            WHERE id = $1
            "#,
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_by_national_operator_code(
        &self,
        national_code: &str,
    ) -> sqlx::Result<Vec<BtVehicle>> {
        sqlx::query_as::<_, BtVehicle>(
            r#"
            SELECT
                *
            FROM bt_vehicles
            WHERE operator_id = $1
            "#,
        )
        .bind(national_code)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn search(&self, query: &BtVehicleQuery) -> sqlx::Result<PaginatedResponse<BtVehicle>> {
        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);

        let allowed_unique = ALLOWED_UNIQUE_FILTERS
            .get("vehicles")
            .unwrap();

        let mut search = SearchQuery::new(
            "v.*, COUNT(*) OVER() AS total_count",
            r#"
                FROM bt_vehicles v
                LEFT JOIN bt_vehicle_types vt
                    ON vt.id = v.vehicle_type_id
                LEFT JOIN tx_operators o
                    ON o.id = v.operator_id
                WHERE TRUE
            "#,
        );

        if let Some(unique) = query.unique.as_deref() {
            search = search.distinct(unique, &allowed_unique);
        }

        let search = search
            .ilike("v.slug", query.slug.as_deref())
            .ilike("v.fleet", query.fleet.as_deref())
            .ilike("v.reg", query.reg.as_deref())
            .ilike("v.previous_reg", query.previous_reg.as_deref())
            .ilike("v.name", query.name.as_deref())

            .eq("v.vehicle_type_id", query.vehicle_type_id)
            .eq("v.livery_id", query.livery_id)
            .eq("v.operator_id", query.operator_id.clone())
            .eq("v.withdrawn", query.withdrawn)
            .eq("vt.electric", query.electric)
            .eq("vt.double_decker", query.double_decker)
            .eq("vt.coach", query.coach)
            .eq("v.training_vehicle", query.training_vehicle)
            .eq("v.demonstrator", query.demonstrator)
            .eq("v.open_top", query.open_top)

            .exists(
                query.has_image,
                r#"
            SELECT 1
            FROM bus_photo_details bpd
            WHERE bpd.matched_vehicle_id = v.id
        "#,
            )

            .search(
                query.search.as_deref(),
                [
                    "v.slug",
                    "v.fleet",
                    "v.reg",
                    "v.previous_reg",
                    "v.name",
                    "o.short_name",
                    "o.code",
                    "o.national_code",
                ],
            )

            .order("v.name ASC, v.id ASC")
            .limit(page_length)
            .offset(query.offset);

        let mut builder = QueryBuilder::<Postgres>::new("");

        search_query(&mut builder, &search)?;

        let results = builder
            .build_query_as::<BtVehicleSearchResult>()
            .fetch_all(&self.pool)
            .await?;

        let total_count = results
            .first()
            .map(|result| result.total_count)
            .unwrap_or(0);

        Ok(PaginatedResponse {
            current_page: query.offset.unwrap_or(0) / page_length + 1,
            total_count,
            items: results
                .into_iter()
                .map(|result| result.vehicle)
                .collect(),
        })
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
    ) -> sqlx::Result<Vec<BtVehicleFilleOut>> {
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

            filled_vehicles.push(BtVehicleFilleOut {
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
