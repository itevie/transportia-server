use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::db::Database;
use crate::db::model_bus_ownerships::ModelBusOwnerships;
use crate::db::resources::model_bus::{ModelBusFilledOut};
use crate::db::resources::model_bus_photo_details::{
    ModelBusPhotoDetails,
    ModelBusPhotoDetailsFilledOut,
    ModelBusPhotoDetailsUpdate,
};
use crate::db::model_buses::ModelBuses;

pub static SQL_MODEL_BUS_PHOTO_DETAILS: &str = r#"
CREATE TABLE IF NOT EXISTS model_bus_photo_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    matched_model_id INTEGER REFERENCES bt_vehicles(id),
    ownership INTEGER REFERENCES model_bus_ownerships(id),
    override_match INTEGER DEFAULT 0 NOT NULL
);
"#;

#[derive(Clone)]
pub struct ModelBusPhotoDetailsManager {
    pool: Pool<SqliteConnectionManager>,
}

impl ModelBusPhotoDetailsManager {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(
        &self,
        id: i64,
    ) -> rusqlite::Result<Option<ModelBusPhotoDetails>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                matched_model_id,
                ownership,
                override_match
            FROM model_bus_photo_details
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_model_bus_photo_details(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create(
        &self,
        details: &ModelBusPhotoDetails,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO model_bus_photo_details (
                matched_model_id,
                ownership,
                override_match
            )
            VALUES (?, ?, ?)
            "#,
            rusqlite::params![
                details.matched_model_id,
                details.ownership,
                details.override_match,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(
        &self,
        id: i64,
        update: &ModelBusPhotoDetailsUpdate,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE model_bus_photo_details SET
                matched_model_id = ?,
                ownership = ?,
                override_match = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                update.matched_model_id,
                update.ownership,
                update.override_match,
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
            "DELETE FROM model_bus_photo_details WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(
        db: Arc<Database>,
        details: ModelBusPhotoDetails,
    ) -> rusqlite::Result<ModelBusPhotoDetailsFilledOut> {
        let ownership = match details.ownership {
            Some(id) => db.model_bus_ownerships.get(id)?,
            None => None,
        };

        let ownership_f = ownership.map(|x|
            ModelBusOwnerships::fill_out(db.clone(), vec![x], None)
        ).transpose()?.map(|x| x.get(0).unwrap().clone());

        let matched: Option<ModelBusFilledOut> = match details.matched_model_id {
            Some(id) => {
                match db.model_buses.get(id)? {
                    Some(model_bus) => {
                        Some(ModelBuses::fill_out(
                            db.clone(),
                            vec![model_bus],
                            None,
                        )?[0].clone())
                    }
                    None => None,
                }
            }
            None => None,
        };

        Ok(ModelBusPhotoDetailsFilledOut {
            id: details.id,
            matched_model_id: details.matched_model_id,
            matched_model: matched,
            ownership: details.ownership,
            ownership_details: ownership_f,
            override_match: details.override_match,
        })
    }

    pub fn fill_out_many(db: Arc<Database>, models: Vec<ModelBusPhotoDetails>) -> rusqlite::Result<Vec<ModelBusPhotoDetailsFilledOut>> {
        models
            .into_iter()
            .map(|model| {
                ModelBusPhotoDetailsManager::fill_out(db.clone(), model)
            })
            .collect()
    }

    fn row_to_model_bus_photo_details(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<ModelBusPhotoDetails> {
        Ok(ModelBusPhotoDetails {
            id: row.get("id")?,
            matched_model_id: row.get("matched_model_id")?,
            ownership: row.get("ownership")?,
            override_match: row.get::<_, i64>("override_match")? != 0,
        })
    }
}