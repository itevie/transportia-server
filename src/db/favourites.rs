use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use rusqlite::OptionalExtension;
use crate::db::comments::Comments;
use crate::db::Database;
use crate::db::model_buses::ModelBuses;
use crate::db::photo::Photos;
use crate::db::resources::favourites::{Favourite, FavouriteFilled, FavouriteObject, FavouriteType};
use crate::db::trams::Trams;
use crate::db::vehicles::BtVehicles;

pub static SQL_FAVOURITE: &str = r#"
CREATE TABLE IF NOT EXISTS favourites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    target_id TEXT NOT NULL,
    target_type INTEGER NOT NULL,
    group_id INTEGER REFERENCES favourites_groups(id),
    created_at TEXT NOT NULL,

    UNIQUE(user_id, target_id, target_type)
)
"#;

#[derive(Clone)]
pub struct Favourites {
    pool: Pool<SqliteConnectionManager>,
}

impl Favourites {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get_by_type(
        &self,
        user_id: i64,
        target_type: FavouriteType,
    ) -> rusqlite::Result<Vec<Favourite>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;



        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                user_id,
                target_id,
                target_type,
                created_at,
                group_id
            FROM favourites
            WHERE user_id = ?
              AND target_type = ?
            ORDER BY created_at DESC
            "#,
        )?;

        statement
            .query_map(
                rusqlite::params![
                    user_id,
                    target_type as i64,
                ],
                Self::row_to_favourite,
            )?
            .collect()
    }

    pub fn get(
        &self,
        user_id: i64,
        target_type: FavouriteType,
        photo_id: i64,
    ) -> rusqlite::Result<Option<Favourite>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
        SELECT
            id,
            user_id,
            target_id,
            target_type,
            created_at,
            group_id
        FROM favourites
        WHERE user_id = ?
          AND target_type = ?
          AND target_id = ?
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        )?;

        statement
            .query_row(
                rusqlite::params![
                user_id,
                target_type as i64,
                photo_id,
            ],
                Self::row_to_favourite,
            )
            .optional()
    }

    pub fn create(
        &self,
        user_id: i64,
        target_id: i64,
        target_type: FavouriteType,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let created_at = chrono::Utc::now().to_rfc3339();

        connection.execute(
            r#"
            INSERT OR IGNORE INTO favourites (
                user_id,
                target_id,
                target_type,
                created_at
            )
            VALUES (?, ?, ?, ?)
            "#,
            rusqlite::params![
                user_id,
                target_id,
                target_type as i64,
                created_at,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn get_by_group_id(
        &self,
        user_id: i64,
        group_id: i64,
    ) -> rusqlite::Result<Vec<Favourite>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
        SELECT
            id,
            user_id,
            target_id,
            target_type,
            created_at,
            group_id
        FROM favourites
        WHERE user_id = ?
          AND group_id = ?
        ORDER BY created_at DESC
        "#,
        )?;

        statement
            .query_map(
                rusqlite::params![user_id, group_id],
                Self::row_to_favourite,
            )?
            .collect()
    }

    pub fn update_group_id(
        &self,
        user_id: i64,
        target_id: i64,
        target_type: FavouriteType,
        group_id: Option<i64>,
    ) -> rusqlite::Result<bool> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let affected = connection.execute(
            r#"
        UPDATE favourites
        SET group_id = ?
        WHERE user_id = ?
          AND target_id = ?
          AND target_type = ?
        "#,
            rusqlite::params![
            group_id,
            user_id,
            target_id,
            target_type as i64,
        ],
        )?;

        Ok(affected > 0)
    }

    pub fn delete(
        &self,
        user_id: i64,
        target_id: i64,
        target_type: FavouriteType,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            DELETE FROM favourites
            WHERE user_id = ?
              AND target_id = ?
              AND target_type = ?
            "#,
            rusqlite::params![
                user_id,
                target_id,
                target_type as i64,
            ],
        )?;

        Ok(())
    }

    pub fn fill(
        &self,
        db: Arc<Database>,
        favourites: Vec<Favourite>,
        user_id: Option<i64>
    ) -> rusqlite::Result<Vec<FavouriteFilled>> {
        favourites
            .into_iter()
            .map(|favourite| {
                let object = match favourite.target_type {
                    FavouriteType::Photo => {
                        let val =
                            db.photos
                                .get(favourite.target_id.parse().unwrap())?
                                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;
                        let filled_out_r = Photos::fill_out(db.clone(), vec![val], user_id)?;
                        let filled_out = filled_out_r.get(0).unwrap();
                        FavouriteObject::BusPhoto(
                            filled_out.clone()
                        )
                    },

                    FavouriteType::Comment => {
                        let val = db
                            .comments
                            .get(favourite.target_id.parse().unwrap())?
                            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;
                        let filled_out = Comments::fill_out(db.clone(), val, user_id)?;

                        FavouriteObject::Comment(filled_out)
                    }

                    FavouriteType::Tram => {
                        let val = db
                            .trams
                            .get(favourite.target_id.parse().unwrap())?
                            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;
                        let filled_out = Trams::fill_out(db.clone(), vec![val], user_id)?;

                        FavouriteObject::Tram(filled_out.get(0).unwrap().clone())
                    }

                    FavouriteType::ModelBus => {
                        let val = db
                            .model_buses
                            .get(favourite.target_id.parse().unwrap())?
                            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;
                        let filled_out = ModelBuses::fill_out(db.clone(), vec![val], user_id)?;

                        FavouriteObject::ModelBus(filled_out.get(0).unwrap().clone())
                    }

                    FavouriteType::Operator => {
                        FavouriteObject::Operator(
                            db.tx_operators
                                .get(&*favourite.target_id)?
                                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?,
                        )
                    }

                    FavouriteType::Vehicle => {
                        let val = db.vehicles
                            .get(favourite.target_id.parse().unwrap())?
                            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;
                        let filled_out_r = BtVehicles::fill_out(db.clone(), vec![val], user_id)?;
                        let filled_out = filled_out_r.get(0).unwrap();
                        FavouriteObject::Vehicle(
                            filled_out.clone()
                        )
                    }

                    FavouriteType::VehicleType => {
                        FavouriteObject::VehicleType(
                            db.vehicle_types
                                .get(favourite.target_id.parse().unwrap())?
                                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?,
                        )
                    }

                    FavouriteType::Livery => {
                        FavouriteObject::Livery(
                            db.liveries
                                .get(favourite.target_id.parse().unwrap())?
                                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?,
                        )
                    }

                    FavouriteType::Service => {
                        FavouriteObject::Service(
                            db.tx_lines
                                .get(&*favourite.target_id)?
                                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?,
                        )
                    }
                };

                Ok(FavouriteFilled {
                    id: favourite.id,
                    user_id: favourite.user_id,
                    target_id: favourite.target_id,
                    target_type: favourite.target_type,
                    created_at: favourite.created_at,
                    object,
                    group: favourite.group,
                })
            })
            .collect()
    }

    fn row_to_favourite(row: &rusqlite::Row) -> rusqlite::Result<Favourite> {
        Ok(Favourite {
            id: row.get("id")?,
            user_id: row.get("user_id")?,
            target_id: row.get("target_id")?,
            target_type: Self::favourite_type_from_i64(
                row.get("target_type")?
            )?,
            created_at: row.get("created_at")?,
            group: row.get("group_id")?,
        })
    }

    fn favourite_type_from_i64(value: i64) -> rusqlite::Result<FavouriteType> {
        match value {
            0 => Ok(FavouriteType::Photo),
            1 => Ok(FavouriteType::Operator),
            2 => Ok(FavouriteType::Vehicle),
            3 => Ok(FavouriteType::VehicleType),
            4 => Ok(FavouriteType::Livery),
            5 => Ok(FavouriteType::Service),
            6 => Ok(FavouriteType::Comment),
            7 => Ok(FavouriteType::Tram),
            _ => Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Integer,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid FavouriteType: {}", value),
                )),
            )),
        }
    }
}