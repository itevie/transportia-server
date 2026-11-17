use std::fmt::Debug;
use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::Database;
use crate::db::model_buses::ModelBuses;
use crate::db::photo::PhotoQuery;
use crate::db::resources::bt_vehicle::{BtVehicle, FilledOutBtVehicle};
use crate::db::resources::favourites::FavouriteType;
use crate::db::resources::model_bus::ModelBusFilledOut;
use crate::db::resources::model_bus_ownership::{ModelBusOwnership, ModelBusOwnershipFilledOut};
use crate::db::resources::photo::PhotoType;

pub static SQL_MODEL_BUS_OWNERSHIPS: &str = r#"
CREATE TABLE IF NOT EXISTS model_bus_ownerships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    model_bus_id INTEGER NOT NULL REFERENCES model_buses(id),
    certificate_number INTEGER,
    bought_at TEXT,
    defects TEXT,
    created_at TEXT NOT NULL,

    UNIQUE(model_bus_id, certificate_number)
)
"#;

#[derive(Clone)]
pub struct ModelBusOwnerships {
    pool: Pool<SqliteConnectionManager>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct ModelBusOwnershipQuery {
    pub user_id: Option<i64>,
    pub model_bus_id: Option<i64>,
    pub certificate_number: Option<i64>,
    pub bought_at: Option<String>,

    // Search
    pub search: Option<String>,

    // Model bus filters
    pub product_code: Option<String>,
    pub manufacturer: Option<String>,
    pub scale: Option<i64>,
    pub operator: Option<String>,
    pub livery: Option<String>,
    pub fleet_number: Option<String>,
    pub registration: Option<String>,
    pub route_number: Option<String>,
    pub destination: Option<String>,
    pub vehicle_type: Option<String>,
    pub has_certificate: Option<bool>,
    pub released: Option<String>,

    // Pagination
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl ModelBusOwnerships {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<ModelBusOwnership>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT *
            FROM model_bus_ownerships
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_model_bus_ownership(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(
        &self,
        query: &ModelBusOwnershipQuery,
    ) -> rusqlite::Result<Vec<ModelBusOwnership>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);

        let mut sql = String::from(
            r#"
        SELECT mbo.*
        FROM model_bus_ownerships mbo
        LEFT JOIN model_buses mb ON mb.id = mbo.model_bus_id
        WHERE 1 = 1
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = query.user_id {
            sql.push_str(" AND mbo.user_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.model_bus_id {
            sql.push_str(" AND mbo.model_bus_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.certificate_number {
            sql.push_str(" AND mbo.certificate_number = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.bought_at {
            sql.push_str(" AND mbo.bought_at = ?");
            params.push(Box::new(value.clone()));
        }

        // Model bus filters
        if let Some(value) = &query.product_code {
            sql.push_str(" AND mb.product_code LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = &query.manufacturer {
            sql.push_str(" AND mb.manufacturer LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = query.scale {
            sql.push_str(" AND mb.scale = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.operator {
            sql.push_str(" AND mb.operator LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = &query.livery {
            sql.push_str(" AND mb.livery LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = &query.fleet_number {
            sql.push_str(" AND mb.fleet_number LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = &query.registration {
            sql.push_str(" AND mb.registration LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = &query.route_number {
            sql.push_str(" AND mb.route_number LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = &query.destination {
            sql.push_str(" AND mb.destination LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = &query.vehicle_type {
            sql.push_str(" AND mb.vehicle_type LIKE ?");
            params.push(Box::new(format!("%{value}%")));
        }

        if let Some(value) = query.has_certificate {
            sql.push_str(" AND mb.certificate_max > 0 = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.released {
            sql.push_str(" AND mb.released = ?");
            params.push(Box::new(value.clone()));
        }

        // General search across ownership and model bus fields
        if let Some(value) = &query.search {
            let search = format!("%{value}%");

            sql.push_str(
                r#"
            AND (
                CAST(mbo.certificate_number AS TEXT) LIKE ?
                OR mbo.bought_at LIKE ?
                OR mb.product_code LIKE ?
                OR mb.manufacturer LIKE ?
                OR mb.operator LIKE ?
                OR mb.livery LIKE ?
                OR mb.fleet_number LIKE ?
                OR mb.registration LIKE ?
                OR mb.route_number LIKE ?
                OR mb.destination LIKE ?
                OR mb.vehicle_type LIKE ?
            )
            "#,
            );

            for _ in 0..11 {
                params.push(Box::new(search.clone()));
            }
        }

        sql.push_str(" ORDER BY mbo.created_at DESC");
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
                Self::row_to_model_bus_ownership,
            )?
            .collect()
    }

    pub fn create(
        &self,
        ownership: &ModelBusOwnership,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO model_bus_ownerships (
                user_id,
                model_bus_id,
                certificate_number,
                bought_at,
                defects,
                created_at
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                ownership.user_id,
                ownership.model_bus_id,
                ownership.certificate_number,
                &ownership.bought_at,
                &ownership.defects,
                &ownership.created_at
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn upsert(
        &self,
        ownership: &ModelBusOwnership,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO model_bus_ownerships (
                id,
                user_id,
                model_bus_id,
                certificate_number,
                bought_at,
                defects,
                created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                user_id = excluded.user_id,
                model_bus_id = excluded.model_bus_id,
                certificate_number = excluded.certificate_number,
                bought_at = excluded.bought_at,
                defects = excluded.defects
            "#,
            rusqlite::params![
                ownership.id,
                ownership.user_id,
                ownership.model_bus_id,
                ownership.certificate_number,
                &ownership.bought_at,
                &ownership.defects,
                &ownership.created_at
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM model_bus_ownerships WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(db: Arc<Database>, vehicles: Vec<ModelBusOwnership>, user_id: Option<i64>) -> rusqlite::Result<Vec<ModelBusOwnershipFilledOut>> {
        vehicles
            .into_iter()
            .map(|model| {
                let model_bus = db.model_buses.get(model.model_bus_id)?.unwrap();
                let model_bus_f = ModelBuses::fill_out(db.clone(), vec![model_bus], user_id)?
                    .get(0)
                    .unwrap()
                    .clone();

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
                    mbd_model_id: Some(model.id),
                    mbd_has_matched_model: None,
                    mbd_owner_id: user_id,
                    page_length: None,
                    offset: None,
                    order_by: None,
                })?.first().cloned();

                Ok(ModelBusOwnershipFilledOut {
                    id: model.id,
                    user_id: model.user_id,
                    model_bus_id: model.model_bus_id,
                    model_bus: model_bus_f,
                    certificate_number: model.certificate_number,
                    bought_at: model.bought_at,
                    defects: model.defects,
                    photo,
                    created_at: model.created_at,
                })
            })
            .collect()
    }

    fn row_to_model_bus_ownership(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<ModelBusOwnership> {
        Ok(ModelBusOwnership {
            id: row.get("id")?,
            user_id: row.get("user_id")?,
            model_bus_id: row.get("model_bus_id")?,
            certificate_number: row.get("certificate_number")?,
            bought_at: row.get("bought_at")?,
            defects: row.get("defects")?,
            created_at: row.get("created_at")?,
        })
    }
}