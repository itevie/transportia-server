use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::db::resources::bt_vehicle_type::BtVehicleType;

pub static SQL_BT_VEHICLE_TYPE: &str = r#"
CREATE TABLE IF NOT EXISTS bt_vehicle_types (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    style TEXT NOT NULL,
    fuel TEXT NOT NULL,
    double_decker INTEGER NOT NULL,
    coach INTEGER NOT NULL,
    electric INTEGER NOT NULL
)
"#;

#[derive(Clone)]
pub struct BtVehicleTypes {
    pool: Pool<SqliteConnectionManager>,
}

impl BtVehicleTypes {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<BtVehicleType>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                name,
                style,
                fuel,
                double_decker,
                coach,
                electric
            FROM bt_vehicle_types
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_bt_vehicle_type(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<BtVehicleType>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
        SELECT
            id,
            name,
            style,
            fuel,
            double_decker,
            coach,
            electric
        FROM bt_vehicle_types
        WHERE name LIKE ?
        ORDER BY name
        LIMIT 50
        "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![search],
            Self::row_to_bt_vehicle_type,
        )?;

        rows.collect()
    }

    pub fn get_by_name(&self, name: &str) -> rusqlite::Result<Option<BtVehicleType>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                name,
                style,
                fuel,
                double_decker,
                coach,
                electric
            FROM bt_vehicle_types
            WHERE name = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_bt_vehicle_type(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(&self, vehicle_type: &BtVehicleType) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
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
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                vehicle_type.id,
                &vehicle_type.name,
                &vehicle_type.style,
                &vehicle_type.fuel,
                &vehicle_type.double_decker,
                &vehicle_type.coach,
                &vehicle_type.electric,
            ],
        )?;

        Ok(())
    }

    pub fn upsert(&self, vehicle_type: &BtVehicleType) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
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
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                style = excluded.style,
                fuel = excluded.fuel,
                double_decker = excluded.double_decker,
                coach = excluded.coach,
                electric = excluded.electric
            "#,
            rusqlite::params![
                vehicle_type.id,
                &vehicle_type.name,
                &vehicle_type.style,
                &vehicle_type.fuel,
                &vehicle_type.double_decker,
                &vehicle_type.coach,
                &vehicle_type.electric,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM bt_vehicle_types WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_bt_vehicle_type(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<BtVehicleType> {
        Ok(BtVehicleType {
            id: row.get("id")?,
            name: row.get("name")?,
            style: row.get("style")?,
            fuel: row.get("fuel")?,
            double_decker: row.get("double_decker")?,
            coach: row.get("coach")?,
            electric: row.get("electric")?,
        })
    }
}