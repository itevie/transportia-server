use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::db::Database;
use crate::db::photo::PhotoQuery;
use crate::db::resources::bt_vehicle::{BtVehicle, FilledOutBtVehicle};
use crate::db::resources::model_bus::{ModelBus, ModelBusFilledOut};
use crate::db::resources::photo::PhotoType;

pub static SQL_MODEL_BUSES: &str = r#"
CREATE TABLE IF NOT EXISTS model_buses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_code TEXT NOT NULL,
    manufacturer TEXT NOT NULL,
    scale INTEGER NOT NULL,
    operator TEXT,
    operator_link TEXT REFERENCES tx_operators(id),
    livery TEXT,
    livery_link INTEGER REFERENCES bt_liveries(id),
    fleet_number TEXT,
    registration TEXT,
    vehicle_link INTEGER REFERENCES bt_vehicles(id),
    route_number TEXT,
    destination TEXT,
    route_link TEXT REFERENCES tx_lines(tx_id),
    vehicle_type TEXT,
    vehicle_type_link INTEGER REFERENCES bt_vehicle_types(id),
    has_certificate INTEGER NOT NULL DEFAULT 0,
    certificate_max INTEGER NOT NULL DEFAULT 0,
    released TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    UNIQUE(product_code, manufacturer)
)
"#;

#[derive(Clone)]
pub struct ModelBuses {
    pool: Pool<SqliteConnectionManager>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct ModelBusQuery {
    pub search: Option<String>,

    pub product_code: Option<String>,
    pub manufacturer: Option<String>,
    pub scale: Option<i64>,

    pub operator: Option<String>,
    pub operator_link: Option<String>,

    pub livery: Option<String>,
    pub livery_link: Option<i64>,

    pub fleet_number: Option<String>,
    pub registration: Option<String>,
    pub vehicle_link: Option<i64>,

    pub route_number: Option<String>,
    pub destination: Option<String>,
    pub route_link: Option<String>,

    pub vehicle_type: Option<String>,
    pub vehicle_type_link: Option<i64>,

    pub has_certificate: Option<bool>,

    pub released: Option<String>,

    pub has_image: Option<bool>,

    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl ModelBuses {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<ModelBus>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT *
            FROM model_buses
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_model_bus(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(
        &self,
        query: &ModelBusQuery,
    ) -> rusqlite::Result<Vec<ModelBus>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);

        let mut sql = String::from(
            r#"
            SELECT
                mb.*
            FROM model_buses mb
            WHERE 1 = 1
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = &query.product_code {
            sql.push_str(" AND mb.product_code LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.manufacturer {
            sql.push_str(" AND mb.manufacturer LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.scale {
            sql.push_str(" AND mb.scale = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.operator {
            sql.push_str(" AND mb.operator LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.operator_link {
            sql.push_str(" AND mb.operator_link = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.livery {
            sql.push_str(" AND mb.livery LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.livery_link {
            sql.push_str(" AND mb.livery_link = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.fleet_number {
            sql.push_str(" AND mb.fleet_number LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.registration {
            sql.push_str(" AND mb.registration LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.vehicle_link {
            sql.push_str(" AND mb.vehicle_link = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.route_number {
            sql.push_str(" AND mb.route_number LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.destination {
            sql.push_str(" AND mb.destination LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.route_link {
            sql.push_str(" AND mb.route_link = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.vehicle_type {
            sql.push_str(" AND mb.vehicle_type LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.vehicle_type_link {
            sql.push_str(" AND mb.vehicle_type_link = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.has_certificate {
            sql.push_str(" AND mb.has_certificate = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.released {
            sql.push_str(" AND mb.released = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.has_image {
            if value {
                sql.push_str(
                    r#"
                    AND EXISTS (
                        SELECT 1
                        FROM model_bus_photo_details mbpd
                        WHERE mbpd.model_bus_link = mb.id
                    )
                    "#,
                );
            } else {
                sql.push_str(
                    r#"
                    AND NOT EXISTS (
                        SELECT 1
                        FROM model_bus_photo_details mbpd
                        WHERE mbpd.model_bus_link = mb.id
                    )
                    "#,
                );
            }
        }

        if let Some(value) = &query.search {
            let search = value
                .to_lowercase()
                .replace(' ', "");

            sql.push_str(
                r#"
        AND (
            -- Model bus fields
            REPLACE(LOWER(mb.product_code), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.manufacturer), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.operator), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.livery), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.fleet_number), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.registration), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.route_number), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.destination), ' ', '') LIKE '%' || ? || '%' OR
            REPLACE(LOWER(mb.vehicle_type), ' ', '') LIKE '%' || ? || '%' OR

            -- Linked vehicle
            EXISTS (
                SELECT 1
                FROM bt_vehicles bv
                WHERE bv.id = mb.vehicle_link
                AND (
                    REPLACE(LOWER(bv.reg), ' ', '') LIKE '%' || ? || '%' OR
                    REPLACE(LOWER(bv.fleet), ' ', '') LIKE '%' || ? || '%' OR
                    REPLACE(LOWER(bv.previous_reg), ' ', '') LIKE '%' || ? || '%' OR
                    REPLACE(LOWER(bv.name), ' ', '') LIKE '%' || ? || '%' OR
                    REPLACE(LOWER(bv.branding), ' ', '') LIKE '%' || ? || '%' OR
                    REPLACE(LOWER(bv.operator_id), ' ', '') LIKE '%' || ? || '%'
                )
            ) OR

            -- Linked vehicle type
            EXISTS (
                SELECT 1
                FROM bt_vehicle_types bvt
                WHERE bvt.id = mb.vehicle_type_link
                AND (
                    REPLACE(LOWER(bvt.name), ' ', '') LIKE '%' || ? || '%' OR
                    REPLACE(LOWER(bvt.style), ' ', '') LIKE '%' || ? || '%' OR
                    REPLACE(LOWER(bvt.fuel), ' ', '') LIKE '%' || ? || '%'
                )
            )
        )
        "#
            );

            for _ in 0..18 {
                params.push(Box::new(search.clone()));
            }
        }

        sql.push_str(" ORDER BY mb.manufacturer, mb.product_code");
        sql.push_str(" LIMIT ?");
        params.push(Box::new(page_length));

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset.max(0)));
        }

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_model_bus,
            )?
            .collect()
    }

    pub fn create(&self, model_bus: &ModelBus) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO model_buses (
                product_code,
                manufacturer,
                scale,
                operator,
                operator_link,
                livery,
                livery_link,
                fleet_number,
                registration,
                vehicle_link,
                route_number,
                destination,
                route_link,
                vehicle_type,
                vehicle_type_link,
                has_certificate,
                certificate_max,
                released,
                created_at,
                updated_at
            )
            VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            "#,
            rusqlite::params![
                &model_bus.product_code,
                &model_bus.manufacturer,
                model_bus.scale,
                &model_bus.operator,
                &model_bus.operator_link,
                &model_bus.livery,
                model_bus.livery_link,
                &model_bus.fleet_number,
                &model_bus.registration,
                model_bus.vehicle_link,
                &model_bus.route_number,
                &model_bus.destination,
                &model_bus.route_link,
                &model_bus.vehicle_type,
                model_bus.vehicle_type_link,
                model_bus.has_certificate,
                model_bus.certificate_max,
                &model_bus.released,
                &model_bus.created_at,
                &model_bus.updated_at
            ],
        )?;
        
        Ok(connection.last_insert_rowid())
    }

    pub fn upsert(&self, model_bus: &ModelBus) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO model_buses (
                id,
                product_code,
                manufacturer,
                scale,
                operator,
                operator_link,
                livery,
                livery_link,
                fleet_number,
                registration,
                vehicle_link,
                route_number,
                destination,
                route_link,
                vehicle_type,
                vehicle_type_link,
                has_certificate,
                certificate_max,
                released,
                created_at,
                updated_at
            )
            VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            ON CONFLICT(id) DO UPDATE SET
                product_code = excluded.product_code,
                manufacturer = excluded.manufacturer,
                scale = excluded.scale,
                operator = excluded.operator,
                operator_link = excluded.operator_link,
                livery = excluded.livery,
                livery_link = excluded.livery_link,
                fleet_number = excluded.fleet_number,
                registration = excluded.registration,
                vehicle_link = excluded.vehicle_link,
                route_number = excluded.route_number,
                destination = excluded.destination,
                route_link = excluded.route_link,
                vehicle_type = excluded.vehicle_type,
                vehicle_type_link = excluded.vehicle_type_link,
                has_certificate = excluded.has_certificate,
                certificate_max = excluded.certificate_max,
                released = excluded.released,
                updated_at = excluded.updated_at
            "#,
            rusqlite::params![
                model_bus.id,
                &model_bus.product_code,
                &model_bus.manufacturer,
                model_bus.scale,
                &model_bus.operator,
                &model_bus.operator_link,
                &model_bus.livery,
                model_bus.livery_link,
                &model_bus.fleet_number,
                &model_bus.registration,
                model_bus.vehicle_link,
                &model_bus.route_number,
                &model_bus.destination,
                &model_bus.route_link,
                &model_bus.vehicle_type,
                model_bus.vehicle_type_link,
                model_bus.has_certificate,
                model_bus.certificate_max,
                &model_bus.released,
                &model_bus.created_at,
                &model_bus.updated_at
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM model_buses WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(db: Arc<Database>, model_buses: Vec<ModelBus>, user_id: Option<i64>) -> rusqlite::Result<Vec<ModelBusFilledOut>> {
        model_buses
            .into_iter()
            .map(|m| {
                let operator = m.operator_link.clone()
                    .map(|id| db.tx_operators.get(&id))
                    .transpose()?
                    .flatten();

                let livery = m.livery_link.clone()
                    .map(|id| db.liveries.get(id))
                    .transpose()?
                    .flatten();

                let vehicle = m.vehicle_link.clone()
                    .map(|id| db.vehicles.get(id))
                    .transpose()?
                    .flatten();

                let vehicle_type = m.vehicle_type_link.clone()
                    .map(|id| db.vehicle_types.get(id))
                    .transpose()?
                    .flatten();

                let route = m.route_link.clone()
                    .map(|id| db.tx_lines.get(&id))
                    .transpose()?
                    .flatten();

                let photo = db.photos.search(&PhotoQuery {
                    id: None,
                    author_id: None,
                    photo_type: Some(PhotoType::ModelBus),
                    title: None,
                    description: None,
                    search: None,
                    bd_reg: None,
                    bd_fleet: None,
                    bd_livery_id: None,
                    bd_vehicle_type_id: None,
                    bd_vehicle_id: None,
                    bd_has_matched_vehicle: None,
                    bd_route: None,
                    bd_operator_id: None,
                    td_fleet: None,
                    td_livery_id: None,
                    td_vehicle_type_id: None,
                    td_operator_id: None,
                    td_vehicle_id: None,
                    td_has_matched_vehicle: None,
                    mbd_manufacturer: None,
                    mbd_product_code: None,
                    mbd_scale: None,
                    mbd_operator: None,
                    mbd_livery: None,
                    mbd_fleet_number: None,
                    mbd_registration: None,
                    mbd_route_number: None,
                    mbd_destination: None,
                    mbd_vehicle_type: None,
                    mbd_has_certificate: None,
                    mbd_released: None,
                    mbd_model_id: Some(m.id),
                    mbd_has_matched_model: None,
                    mbd_owner_id: None,
                    page_length: None,
                    offset: None,
                    order_by: None,
                })?.first().cloned();
                
                Ok(ModelBusFilledOut {
                    id: m.id,
                    product_code: m.product_code,
                    manufacturer: m.manufacturer,
                    scale: m.scale,
                    operator: m.operator,
                    operator_link: m.operator_link,
                    operator_obj: operator,
                    livery: m.livery,
                    livery_link: m.livery_link,
                    livery_obj: livery,
                    fleet_number: m.fleet_number,
                    registration: m.registration,
                    vehicle_link: m.vehicle_link,
                    vehicle_obj: vehicle,
                    route_number: m.route_number,
                    destination: m.destination,
                    route_link: m.route_link,
                    route_obj: route,
                    vehicle_type: m.vehicle_type,
                    vehicle_type_link: m.vehicle_type_link,
                    vehicle_type_obj: vehicle_type,
                    has_certificate: m.has_certificate,
                    certificate_max: m.certificate_max,
                    released: m.released,
                    photo,
                    created_at: m.created_at,
                    updated_at: m.updated_at,
                })
            })
            .collect()
    }
    
    fn row_to_model_bus(row: &rusqlite::Row) -> rusqlite::Result<ModelBus> {
        Ok(ModelBus {
            id: row.get("id")?,

            product_code: row.get("product_code")?,
            manufacturer: row.get("manufacturer")?,
            scale: row.get("scale")?,

            operator: row.get("operator")?,
            operator_link: row.get("operator_link")?,

            livery: row.get("livery")?,
            livery_link: row.get("livery_link")?,

            fleet_number: row.get("fleet_number")?,
            registration: row.get("registration")?,
            vehicle_link: row.get("vehicle_link")?,

            route_number: row.get("route_number")?,
            destination: row.get("destination")?,
            route_link: row.get("route_link")?,

            vehicle_type: row.get("vehicle_type")?,
            vehicle_type_link: row.get("vehicle_type_link")?,

            has_certificate: row.get("has_certificate")?,
            certificate_max: row.get("certificate_max")?,

            released: row.get("released")?,

            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
