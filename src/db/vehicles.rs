use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::Database;
use crate::db::photo::PhotoQuery;
use crate::db::resources::bt_vehicle::{BtVehicle, FilledOutBtVehicle};
use crate::db::resources::favourites::FavouriteType;
use crate::db::util::photo_search::query_only_photo_search;

pub static SQL_BT_VEHICLE: &str = r#"
CREATE TABLE IF NOT EXISTS bt_vehicles (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL,
    fleet TEXT NOT NULL,
    reg TEXT NOT NULL,
    previous_reg TEXT NOT NULL,
    vehicle_type_id INTEGER,
    livery_id INTEGER,
    branding TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    garage_id TEXT,
    name TEXT NOT NULL,
    notes TEXT NOT NULL,
    withdrawn INTEGER NOT NULL,
    training_vehicle INTEGER NOT NULL DEFAULT 0,
    open_top INTEGER NOT NULL DEFAULT 0,
    demonstrator INTEGER NOT NULL DEFAULT 0,
    comment_thread INTEGER REFERENCES comment_threads(id)
)
"#;

#[derive(Clone)]
pub struct BtVehicles {
    pool: Pool<SqliteConnectionManager>,
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
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<BtVehicle>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
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
            FROM bt_vehicles
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_bt_vehicle(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(
        &self,
        query: &BtVehicleQuery,
    ) -> rusqlite::Result<Vec<BtVehicle>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).min(1000);

        let mut sql = String::from(
            r#"
        SELECT
            v.id,
            v.slug,
            v.fleet,
            v.reg,
            v.previous_reg,
            v.vehicle_type_id,
            v.livery_id,
            v.branding,
            v.operator_id,
            v.garage_id,
            v.name,
            v.notes,
            v.withdrawn,
            v.training_vehicle,
            v.demonstrator,
            v.open_top,
            v.comment_thread
        FROM bt_vehicles v
        LEFT JOIN bt_vehicle_types vt
            ON vt.id = v.vehicle_type_id
        LEFT JOIN tx_operators o
            ON o.id = v.operator_id
        WHERE 1 = 1
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = &query.slug {
            sql.push_str(" AND v.slug LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.fleet {
            sql.push_str(" AND v.fleet LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.reg {
            sql.push_str(" AND v.reg LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.previous_reg {
            sql.push_str(" AND v.previous_reg LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.vehicle_type_id {
            sql.push_str(" AND v.vehicle_type_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.livery_id {
            sql.push_str(" AND v.livery_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.operator_id {
            sql.push_str(" AND v.operator_id = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.name {
            sql.push_str(" AND v.name LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.withdrawn {
            sql.push_str(" AND v.withdrawn = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.electric {
            sql.push_str(" AND vt.electric = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.double_decker {
            sql.push_str(" AND vt.double_decker = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.coach {
            sql.push_str(" AND vt.coach = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.training_vehicle {
            sql.push_str(" AND v.training_vehicle = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.demonstrator {
            sql.push_str(" AND v.demonstrator = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.open_top {
            sql.push_str(" AND v.open_top = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.has_image {
            if value {
                sql.push_str(
                    r#"
                AND EXISTS (
                    SELECT 1
                    FROM bus_photo_details bpd
                    WHERE bpd.matched_vehicle_id = v.id
                )
                "#,
                );
            } else {
                sql.push_str(
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

        if let Some(value) = &query.search {
            sql.push_str(
                r#"
                AND (
                    v.slug LIKE '%' || ? || '%' OR
                    v.fleet LIKE '%' || ? || '%' OR
                    v.reg LIKE '%' || ? || '%' OR
                    v.previous_reg LIKE '%' || ? || '%' OR
                    v.name LIKE '%' || ? || '%' OR
                    o.short_name LIKE '%' || ? || '%' OR
                    o.code LIKE '%' || ? || '%' OR
                    o.national_code LIKE '%' || ? || '%'
                )
                "#,
            );

            for _ in 0..8 {
                params.push(Box::new(value.clone()));
            }
        }

        sql.push_str(" ORDER BY v.name");
        sql.push_str(" LIMIT ?");
        params.push(Box::new(page_length));

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset));
        }

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_bt_vehicle,
            )?
            .collect()
    }

    pub fn create(&self, vehicle: &BtVehicle) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
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
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                vehicle.id,
                &vehicle.slug,
                &vehicle.fleet,
                &vehicle.reg,
                &vehicle.previous_reg,
                vehicle.vehicle_type_id,
                vehicle.livery_id,
                &vehicle.branding,
                &vehicle.operator_id,
                &vehicle.garage_id,
                &vehicle.name,
                &vehicle.notes,
                &vehicle.withdrawn,
                &vehicle.training_vehicle,
                &vehicle.demonstrator,
                &vehicle.open_top,
                &vehicle.comment_thread
            ],
        )?;

        Ok(())
    }

    pub fn upsert(&self, vehicle: &BtVehicle) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
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
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                slug = excluded.slug,
                fleet = excluded.fleet,
                reg = excluded.reg,
                previous_reg = excluded.previous_reg,
                vehicle_type_id = excluded.vehicle_type_id,
                livery_id = excluded.livery_id,
                branding = excluded.branding,
                operator_id = excluded.operator_id,
                garage_id = excluded.garage_id,
                name = excluded.name,
                notes = excluded.notes,
                withdrawn = excluded.withdrawn,
                training_vehicle = excluded.training_vehicle,
                demonstrator = excluded.demonstrator,
                open_top = excluded.open_top,
                comment_thread = excluded.comment_thread
            "#,
            rusqlite::params![
                vehicle.id,
                &vehicle.slug,
                &vehicle.fleet,
                &vehicle.reg,
                &vehicle.previous_reg,
                vehicle.vehicle_type_id,
                vehicle.livery_id,
                &vehicle.branding,
                &vehicle.operator_id,
                &vehicle.garage_id,
                &vehicle.name,
                &vehicle.notes,
                &vehicle.withdrawn,
                &vehicle.training_vehicle,
                &vehicle.demonstrator,
                &vehicle.open_top,
                &vehicle.comment_thread
            ],
        )?;

        Ok(())
    }

    /// Upserts only bustimes data, does not include transportia data so it doesn't overwrite
    /// with None.
    pub fn upsert_bustimes(&self, vehicle: &BtVehicle) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
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
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                slug = excluded.slug,
                fleet = excluded.fleet,
                reg = excluded.reg,
                previous_reg = excluded.previous_reg,
                vehicle_type_id = excluded.vehicle_type_id,
                livery_id = excluded.livery_id,
                branding = excluded.branding,
                operator_id = excluded.operator_id,
                garage_id = excluded.garage_id,
                name = excluded.name,
                notes = excluded.notes,
                withdrawn = excluded.withdrawn
            "#,
            rusqlite::params![
                vehicle.id,
                &vehicle.slug,
                &vehicle.fleet,
                &vehicle.reg,
                &vehicle.previous_reg,
                vehicle.vehicle_type_id,
                vehicle.livery_id,
                &vehicle.branding,
                &vehicle.operator_id,
                &vehicle.garage_id,
                &vehicle.name,
                &vehicle.notes,
                &vehicle.withdrawn
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM bt_vehicles WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(db: Arc<Database>, vehicles: Vec<BtVehicle>, user_id: Option<i64>) -> rusqlite::Result<Vec<FilledOutBtVehicle>> {
        vehicles
            .into_iter()
            .map(|vehicle| {
                let vehicle_type = match vehicle.vehicle_type_id {
                    Some(id) => db.vehicle_types.get(id)?,
                    None => None,
                };

                let livery = match vehicle.livery_id {
                    Some(id) => db.liveries.get(id)?,
                    None => None,
                };

                let operator = db.tx_operators.search(&vehicle.operator_id)?;
                let op = if operator.len() == 0 {
                    None
                } else {
                    Some(operator[0].clone())
                };

                let favourite = match user_id {
                    Some(user) => {
                        db.favourites.get(user, FavouriteType::Vehicle, vehicle.id)?
                    },
                    None => None
                };
                
                let photos = db.photos.search(&query_only_photo_search(vehicle.reg.clone()))?;

                Ok(FilledOutBtVehicle {
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
                    photo: photos.get(0).cloned(),
                    is_favourited: if let Some(_) = user_id {
                        Some(favourite.is_some())
                    } else {
                        None
                    },
                    comment_thread: vehicle.comment_thread
                })
            })
            .collect()
    }

    fn row_to_bt_vehicle(row: &rusqlite::Row) -> rusqlite::Result<BtVehicle> {
        Ok(BtVehicle {
            id: row.get("id")?,
            slug: row.get("slug")?,
            fleet: row.get("fleet")?,
            reg: row.get("reg")?,
            previous_reg: row.get("previous_reg")?,
            vehicle_type_id: row.get("vehicle_type_id")?,
            livery_id: row.get("livery_id")?,
            branding: row.get("branding")?,
            operator_id: row.get("operator_id")?,
            garage_id: row.get("garage_id")?,
            name: row.get("name")?,
            notes: row.get("notes")?,
            withdrawn: row.get("withdrawn")?,
            training_vehicle: row.get("training_vehicle")?,
            demonstrator: row.get("demonstrator")?,
            open_top: row.get("open_top")?,
            comment_thread: row.get("comment_thread")?,
        })
    }
}