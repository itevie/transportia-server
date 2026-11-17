use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::Database;
use crate::db::photo::PhotoQuery;
use crate::db::resources::bt_vehicle::{BtVehicle, FilledOutBtVehicle};
use crate::db::resources::favourites::FavouriteType;
use crate::db::resources::photo::PhotoType;
use crate::db::resources::tram::{Tram, TramFilledOut};

pub static SQL_TRAM: &str = r#"
CREATE TABLE IF NOT EXISTS trams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operator_id INTEGER REFERENCES tram_operators(id),
    fleet TEXT NOT NULL,
    livery_id INTEGER REFERENCES tram_livery(id),
    tram_type_id INTEGER REFERENCES tram_type(id),
    garage TEXT,
    withdrawn INTEGER NOT NULL DEFAULT 0,
    name TEXT,
    branding TEXT
)
"#;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TramQuery {
    pub search: Option<String>,

    pub operator: Option<i64>,
    pub fleet: Option<String>,
    pub livery: Option<i64>,
    pub tram_type: Option<i64>,
    pub garage: Option<String>,
    pub withdrawn: Option<bool>,

    pub name: Option<String>,
    pub branding: Option<String>,

    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone)]
pub struct Trams {
    pool: Pool<SqliteConnectionManager>,
}

impl Trams {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<Tram>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                operator_id,
                fleet,
                livery_id,
                tram_type_id,
                garage,
                withdrawn,
                name,
                branding
            FROM trams
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &TramQuery) -> rusqlite::Result<Vec<Tram>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).min(1000);

        let mut sql = String::from(
            r#"
            SELECT
                t.id,
                t.operator_id,
                t.fleet,
                t.livery_id,
                t.tram_type_id,
                t.garage,
                t.withdrawn,
                t.name,
                t.branding
            FROM trams t
            WHERE 1 = 1
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = query.operator {
            sql.push_str(" AND t.operator_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.fleet {
            sql.push_str(" AND t.fleet LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.livery {
            sql.push_str(" AND t.livery_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.tram_type {
            sql.push_str(" AND t.tram_type_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.garage {
            sql.push_str(" AND t.garage LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.withdrawn {
            sql.push_str(" AND t.withdrawn = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.name {
            sql.push_str(" AND t.name LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.branding {
            sql.push_str(" AND t.branding LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.search {
            sql.push_str(
                r#"
                AND (
                    t.fleet LIKE '%' || ? || '%' OR
                    t.garage LIKE '%' || ? || '%' OR
                    t.name LIKE '%' || ? || '%' OR
                    t.branding LIKE '%' || ? || '%'
                )
                "#,
            );

            for _ in 0..4 {
                params.push(Box::new(value.clone()));
            }
        }

        sql.push_str(" ORDER BY t.fleet");
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
                Self::row_to_tram,
            )?
            .collect()
    }

    pub fn create(&self, tram: &Tram) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO trams (
                operator_id,
                fleet,
                livery_id,
                tram_type_id,
                garage,
                withdrawn,
                name,
                branding
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &tram.operator_id,
                &tram.fleet,
                &tram.livery_id,
                &tram.tram_type_id,
                &tram.garage,
                &tram.withdrawn,
                &tram.name,
                &tram.branding,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, tram: &Tram) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let updated = connection.execute(
            r#"
            UPDATE trams
            SET
                operator_id = ?,
                fleet = ?,
                livery_id = ?,
                tram_type_id = ?,
                garage = ?,
                withdrawn = ?,
                name = ?,
                branding = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &tram.operator_id,
                &tram.fleet,
                &tram.livery_id,
                &tram.tram_type_id,
                &tram.garage,
                &tram.withdrawn,
                &tram.name,
                &tram.branding,
                tram.id,
            ],
        )?;

        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM trams WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn fill_out(db: Arc<Database>, trams: Vec<Tram>, user_id: Option<i64>) -> rusqlite::Result<Vec<TramFilledOut>> {
        trams
            .into_iter()
            .map(|x| {
                // TODO: Maybe make it ignore errors and turn into None
                let operator = x.operator_id
                    .map(|id| db.tram_operators.get(id))
                    .transpose()?
                    .flatten();

                let tram_type = x.tram_type_id
                    .map(|id| db.tram_types.get(id))
                    .transpose()?
                    .flatten();

                let livery = x.livery_id
                    .map(|id| db.tram_liveries.get(id))
                    .transpose()?
                    .flatten();

                let photo = db.photos.search(&PhotoQuery {
                    id: None,
                    author_id: None,
                    photo_type: Some(PhotoType::Tram),
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
                    td_vehicle_id: Some(x.id),
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
                    mbd_model_id: None,
                    mbd_has_matched_model: None,
                    mbd_owner_id: user_id,
                    page_length: None,
                    offset: None,
                    order_by: None,
                })?.first().cloned();

                let favourite = match user_id {
                    Some(user) => {
                        db.favourites.get(user, FavouriteType::Vehicle, x.id)?
                    },
                    None => None
                };

                Ok(TramFilledOut {
                    id: x.id,
                    operator_id: x.operator_id,
                    operator,
                    fleet: x.fleet,
                    livery_id: x.livery_id,
                    livery,
                    tram_type_id: x.tram_type_id,
                    tram_type,
                    name: x.name,
                    garage: x.garage,
                    withdrawn: x.withdrawn,
                    branding: x.branding,
                    photo,
                    is_favourited: if let Some(_) = user_id {
                        Some(favourite.is_some())
                    } else {
                        None
                    },
                })
            })
            .collect()
    }

    fn row_to_tram(row: &rusqlite::Row) -> rusqlite::Result<Tram> {
        Ok(Tram {
            id: row.get("id")?,
            operator_id: row.get("operator_id")?,
            fleet: row.get("fleet")?,
            livery_id: row.get("livery_id")?,
            tram_type_id: row.get("tram_type_id")?,
            garage: row.get("garage")?,
            withdrawn: row.get("withdrawn")?,
            name: row.get("name")?,
            branding: row.get("branding")?,
        })
    }
}
