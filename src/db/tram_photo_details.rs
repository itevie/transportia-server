
use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::db::Database;
use crate::db::resources::tram::{TramFilledOut};
use crate::db::resources::tram_photo_details::{
    TramPhotoDetails,
    TramPhotoDetailsFilledOut,
    TramPhotoDetailsUpdate,
};
use crate::db::trams::Trams;

pub static SQL_TRAM_PHOTO_DETAILS: &str = r#"
CREATE TABLE IF NOT EXISTS tram_photo_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matched_vehicle_id INTEGER REFERENCES trams(id),
    override_match INTEGER DEFAULT 0 NOT NULL,
    fleet TEXT,
    livery_id INTEGER REFERENCES tram_liveries(id),
    vehicle_type_id INTEGER REFERENCES tram_types(id),
    operator_id INTEGER REFERENCES tram_operators(id)
);
"#;

#[derive(Clone)]
pub struct TramPhotoDetailsManager {
    pool: Pool<SqliteConnectionManager>,
}

impl TramPhotoDetailsManager {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(
        &self,
        id: i64,
    ) -> rusqlite::Result<Option<TramPhotoDetails>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                matched_vehicle_id,
                override_match,
                fleet,
                livery_id,
                vehicle_type_id,
                operator_id
            FROM tram_photo_details
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram_photo_details(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(
        &self,
        details: &TramPhotoDetails,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tram_photo_details (
                matched_vehicle_id,
                override_match,
                fleet,
                livery_id,
                vehicle_type_id,
                operator_id
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                details.matched_vehicle_id,
                details.override_match,
                details.fleet,
                details.livery_id,
                details.vehicle_type_id,
                details.operator_id,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(
        &self,
        id: i64,
        update: &TramPhotoDetailsUpdate,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE tram_photo_details SET
                matched_vehicle_id = ?,
                override_match = ?,
                fleet = ?,
                livery_id = ?,
                vehicle_type_id = ?,
                operator_id = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                update.matched_vehicle_id,
                update.override_match,
                update.fleet,
                update.livery_id,
                update.vehicle_type_id,
                update.operator_id,
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
            "DELETE FROM tram_photo_details WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(
        db: Arc<Database>,
        details: TramPhotoDetails,
    ) -> rusqlite::Result<TramPhotoDetailsFilledOut> {
        let livery = match details.livery_id {
            Some(id) => db.tram_liveries.get(id)?,
            None => None,
        };

        let vt = match details.vehicle_type_id {
            Some(id) => db.tram_types.get(id)?,
            None => None,
        };

        let operator = match details.operator_id {
            Some(id) => db.tram_operators.get(id)?,
            None => None,
        };

        let matched: Option<TramFilledOut> = match details.matched_vehicle_id {
            Some(id) => {
                match db.trams.get(id)? {
                    Some(tram) => {
                        Some(Trams::fill_out(
                            db.clone(),
                            vec![tram],
                            None,
                        )?[0].clone())
                    }
                    None => None,
                }
            }
            None => None,
        };

        Ok(TramPhotoDetailsFilledOut {
            id: details.id,
            matched_vehicle_id: details.matched_vehicle_id,
            matched_vehicle: matched,
            override_match: details.override_match,
            fleet: details.fleet,
            livery_id: details.livery_id,
            livery,
            vehicle_type_id: details.vehicle_type_id,
            vehicle_type: vt,
            operator_id: details.operator_id,
            operator,
        })
    }

    fn row_to_tram_photo_details(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TramPhotoDetails> {
        Ok(TramPhotoDetails {
            id: row.get("id")?,
            matched_vehicle_id: row.get("matched_vehicle_id")?,
            override_match: row.get::<_, i64>("override_match")? != 0,
            fleet: row.get("fleet")?,
            livery_id: row.get("livery_id")?,
            vehicle_type_id: row.get("vehicle_type_id")?,
            operator_id: row.get("operator_id")?,
        })
    }
}
