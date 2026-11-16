use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::Database;
use crate::db::resources::bt_vehicle::FilledOutBtVehicle;
use crate::db::resources::bus_photo_details::{BusPhotoDetails, BusPhotoDetailsFilledOut, BusPhotoDetailsUpdate};
use crate::db::vehicles::BtVehicles;

// TODO: Add manual operator
pub static SQL_BUS_PHOTO_DETAILS: &str = r#"
CREATE TABLE IF NOT EXISTS bus_photo_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matched_vehicle_id INTEGER REFERENCES bt_vehicles(id),
    override_match INTEGER DEFAULT 0 NOT NULL, -- Whether or not to prefer this details over matched ones
    reg TEXT,
    fleet TEXT,
    livery_id INTEGER REFERENCES bt_liveries(id),
    vehicle_type_id INTEGER REFERENCES bt_vehicle_types(id),
    route_id TEXT REFERENCES tx_lines(tx_id)
);
"#;

#[derive(Clone)]
pub struct BusPhotoDetailsManager {
    pool: Pool<SqliteConnectionManager>,
}

impl BusPhotoDetailsManager {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(
        &self,
        id: i64,
    ) -> rusqlite::Result<Option<BusPhotoDetails>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                matched_vehicle_id,
                override_match,
                reg,
                fleet,
                livery_id,
                vehicle_type_id,
                route_id
            FROM bus_photo_details
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_bus_photo_details(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(
        &self,
        details: &BusPhotoDetails,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO bus_photo_details (
                matched_vehicle_id,
                override_match,
                reg,
                fleet,
                livery_id,
                vehicle_type_id,
                route_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                details.matched_vehicle_id,
                details.override_match,
                details.reg,
                details.fleet,
                details.livery_id,
                details.vehicle_type_id,
                details.route_id,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(
        &self,
        id: i64,
        update: &BusPhotoDetailsUpdate,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
        UPDATE bus_photo_details SET
            matched_vehicle_id = ?,
            override_match = ?,
            reg = ?,
            fleet = ?,
            livery_id = ?,
            vehicle_type_id = ?,
            route_id = ?
        WHERE id = ?
        "#,
            rusqlite::params![
            update.matched_vehicle_id,
            update.override_match,
            update.reg,
            update.fleet,
            update.livery_id,
            update.vehicle_type_id,
            update.route_id,
            id,
        ],
        )?;

        Ok(())
    }

    pub fn delete(
        &self,
        id: i64,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM bus_photo_details WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(db: Arc<Database>, details: BusPhotoDetails) -> rusqlite::Result<BusPhotoDetailsFilledOut> {
        let livery = match details.livery_id {
            Some(id) => db.liveries.get(id)?,
            None => None,
        };

        let vt = match details.vehicle_type_id {
            Some(id) => db.vehicle_types.get(id)?,
            None => None,
        };

        let route = match details.route_id {
            Some(ref id) => db.tx_lines.get(&id.clone())?,
            None => None
        };

        let matched: Option<FilledOutBtVehicle> = match details.matched_vehicle_id {
            Some(id) => {
                match db.vehicles.get(id)? {
                    Some(vehicle) => {
                        Some(BtVehicles::fill_out(db.clone(), vec![vehicle], None)?[0].clone())
                    },
                    None => None,
                }

            },
            None => None
        };

        Ok(BusPhotoDetailsFilledOut {
            id: details.id,
            matched_vehicle_id: details.matched_vehicle_id,
            matched_vehicle: matched,
            override_match: details.override_match,
            reg: details.reg,
            fleet: details.fleet,
            livery_id: details.livery_id,
            livery,
            vehicle_type_id: details.vehicle_type_id,
            vehicle_type: vt,
            route_id: details.route_id,
            route,
        })
    }

    fn row_to_bus_photo_details(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<BusPhotoDetails> {
        Ok(BusPhotoDetails {
            id: row.get("id")?,
            matched_vehicle_id: row.get("matched_vehicle_id")?,
            override_match: row.get::<_, i64>("override_match")? != 0,
            reg: row.get("reg")?,
            fleet: row.get("fleet")?,
            livery_id: row.get("livery_id")?,
            vehicle_type_id: row.get("vehicle_type_id")?,
            route_id: row.get("route_id")?
        })
    }
}