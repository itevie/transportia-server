use std::sync::Arc;
use chrono::Utc;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::{Deserialize, Serialize};
use crate::db::bus_photo_details::BusPhotoDetailsManager;
use crate::db::Database;
use crate::db::model_bus_photo_details::ModelBusPhotoDetailsManager;
use crate::db::resources::bus_photo_details::BusPhotoDetails;
use crate::db::resources::favourites::FavouriteType;
use crate::db::resources::model_bus_photo_details::ModelBusPhotoDetails;
use crate::db::resources::photo::{Photo, PhotoDetails, PhotoFilledOut, PhotoType, PhotoUpdate};
use crate::db::resources::tram_photo_details::TramPhotoDetails;
use crate::db::tram_photo_details::TramPhotoDetailsManager;

pub static SQL_PHOTOS: &str = r#"
CREATE TABLE IF NOT EXISTS photos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    author_id INTEGER REFERENCES users(id) NOT NULL,
    photo_type INTEGER NOT NULL, -- 0 = Bus Photo, 1 = Model Bus Photo, 2 = Train Photo, etc.
    obj_details_id INTEGER, -- If photo_type is 0 use bus_photo_details
    comment_thread INTEGER REFERENCES comment_threads(id),
    file_name TEXT NOT NULL,
    uploaded_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL
);
"#;

#[derive(Serialize, Deserialize, utoipa::IntoParams)]
pub struct PhotoQuery {
    pub id: Option<i64>,
    pub author_id: Option<i64>,
    pub photo_type: Option<PhotoType>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub search: Option<String>,

    // Bus details
    pub bd_reg: Option<String>,
    pub bd_fleet: Option<String>,
    pub bd_livery_id: Option<i64>,
    pub bd_vehicle_type_id: Option<i64>,
    pub bd_vehicle_id: Option<i64>,
    pub bd_has_matched_vehicle: Option<bool>,
    pub bd_route: Option<String>,
    pub bd_operator_id: Option<String>,

    // Tram details
    pub td_fleet: Option<String>,
    pub td_livery_id: Option<i64>,
    pub td_vehicle_type_id: Option<i64>,
    pub td_operator_id: Option<i64>,
    pub td_vehicle_id: Option<i64>,
    pub td_has_matched_vehicle: Option<bool>,

    // Model bus details
    pub mbd_manufacturer: Option<String>,
    pub mbd_product_code: Option<String>,
    pub mbd_scale: Option<i64>,
    pub mbd_operator: Option<String>,
    pub mbd_livery: Option<String>,
    pub mbd_fleet_number: Option<String>,
    pub mbd_registration: Option<String>,
    pub mbd_route_number: Option<String>,
    pub mbd_destination: Option<String>,
    pub mbd_vehicle_type: Option<String>,
    pub mbd_has_certificate: Option<bool>,
    pub mbd_released: Option<bool>,
    pub mbd_model_id: Option<i64>,
    pub mbd_has_matched_model: Option<bool>,
    pub mbd_owner_id: Option<i64>,

    // Pagination
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}


#[derive(Clone)]
pub struct Photos {
    pub(crate) pool: Pool<SqliteConnectionManager>,
}

impl Photos {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(
        &self,
        id: i64,
    ) -> rusqlite::Result<Option<Photo>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
        SELECT
            id,
            author_id,
            photo_type,
            obj_details_id,
            comment_thread,
            file_name,
            uploaded_id,
            title,
            description,
            created_at
        FROM photos
        WHERE id = ?
        "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_photo(row)?))
        } else {
            Ok(None)
        }
    }

    // pub fn search(
    //     &self,
    //     query: &PhotoQuery
    // ) -> rusqlite::Result<Vec<Photo>> {
    //     let connection = self.pool.get().map_err(|err| {
    //         rusqlite::Error::ToSqlConversionFailure(Box::new(err))
    //     })?;
    //
    //     let page_length = match query.page_length {
    //         Some(x) => if x > 1000 { 1000 } else { x },
    //         None => 50,
    //     };
    //
    //     let mut sql = String::from(
    //         r#"
    //     SELECT
    //         p.id,
    //         p.author_id,
    //         p.photo_type,
    //         p.obj_details_id,
    //         p.comment_thread,
    //         p.file_name,
    //         p.uploaded_id,
    //         p.title,
    //         p.description,
    //         p.created_at
    //     FROM photos p
    //     LEFT JOIN bus_photo_details bd
    //         ON bd.id = p.obj_details_id
    //         AND p.photo_type = 0
    //     LEFT JOIN bt_vehicles v
    //         ON v.id = bd.matched_vehicle_id
    //     WHERE 1 = 1
    //     "#,
    //     );
    //
    //     let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    //
    //     if let Some(value) = query.id {
    //         sql.push_str(" AND p.id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.author_id {
    //         sql.push_str(" AND p.author_id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.photo_type {
    //         sql.push_str(" AND p.photo_type = ?");
    //         params.push(Box::new(photo_type_to_i64(value)));
    //     }
    //
    //     if let Some(value) = &query.title {
    //         sql.push_str(" AND p.title LIKE '%' || ? || '%'");
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = &query.description {
    //         sql.push_str(" AND p.description LIKE '%' || ? || '%'");
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     // General search
    //     if let Some(value) = &query.search {
    //         sql.push_str(
    //             r#"
    //     AND (
    //         p.title LIKE '%' || ? || '%' OR
    //         p.description LIKE '%' || ? || '%' OR
    //         p.file_name LIKE '%' || ? || '%' OR
    //         bd.reg LIKE '%' || ? || '%' OR
    //         bd.fleet LIKE '%' || ? || '%' OR
    //         v.reg LIKE '%' || ? || '%' OR
    //         v.fleet LIKE '%' || ? || '%'
    //     )
    //     "#,
    //         );
    //
    //         for _ in 0..7 {
    //             params.push(Box::new(value.clone()));
    //         }
    //     }
    //
    //     // Bus details
    //     if let Some(value) = &query.bd_reg {
    //         sql.push_str(
    //             " AND (bd.reg LIKE '%' || ? || '%' OR v.reg LIKE '%' || ? || '%')"
    //         );
    //
    //         params.push(Box::new(value.clone()));
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = &query.bd_fleet {
    //         sql.push_str(
    //             " AND (bd.fleet LIKE '%' || ? || '%' OR v.fleet LIKE '%' || ? || '%')"
    //         );
    //
    //         params.push(Box::new(value.clone()));
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = query.bd_livery_id {
    //         sql.push_str(
    //             " AND (bd.livery_id = ? OR v.livery_id = ?)"
    //         );
    //
    //         params.push(Box::new(value));
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.bd_vehicle_type_id {
    //         sql.push_str(
    //             " AND (bd.vehicle_type_id = ? OR v.vehicle_type_id = ?)"
    //         );
    //
    //         params.push(Box::new(value));
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.bd_vehicle_id {
    //         sql.push_str(" AND bd.matched_vehicle_id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = &query.bd_route {
    //         sql.push_str(" AND bd.route_id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.bd_has_matched_vehicle {
    //         if value {
    //             sql.push_str(" AND bd.matched_vehicle_id IS NOT NULL");
    //         } else {
    //             sql.push_str(" AND bd.matched_vehicle_id IS NULL");
    //         }
    //     }
    //
    //     sql.push_str(" ORDER BY p.created_at DESC");
    //
    //     sql.push_str(" LIMIT ?");
    //     params.push(Box::new(page_length));
    //
    //     if let Some(offset) = query.offset {
    //         sql.push_str(" OFFSET ?");
    //         params.push(Box::new(offset));
    //     }
    //
    //     let mut statement = connection.prepare(&sql)?;
    //
    //     statement
    //         .query_map(
    //             rusqlite::params_from_iter(
    //                 params.iter().map(|param| param.as_ref()),
    //             ),
    //             Self::row_to_photo,
    //         )?
    //         .collect()
    // }


    // pub fn search(
    //     &self,
    //     query: &PhotoQuery
    // ) -> rusqlite::Result<Vec<Photo>> {
    //     let connection = self.pool.get().map_err(|err| {
    //         rusqlite::Error::ToSqlConversionFailure(Box::new(err))
    //     })?;
    //
    //     let page_length = match query.page_length {
    //         Some(x) => if x > 1000 { 1000 } else { x },
    //         None => 50,
    //     };
    //
    //     let mut sql = String::from(
    //         r#"
    //     SELECT
    //         p.id,
    //         p.author_id,
    //         p.photo_type,
    //         p.obj_details_id,
    //         p.comment_thread,
    //         p.file_name,
    //         p.uploaded_id,
    //         p.title,
    //         p.description,
    //         p.created_at
    //     FROM photos p
    //
    //     LEFT JOIN bus_photo_details bd
    //         ON bd.id = p.obj_details_id
    //         AND p.photo_type = 0
    //
    //     LEFT JOIN bt_vehicles v
    //         ON v.id = bd.matched_vehicle_id
    //
    //     LEFT JOIN tram_photo_details td
    //         ON td.id = p.obj_details_id
    //         AND p.photo_type = 3
    //
    //     LEFT JOIN trams t
    //         ON t.id = td.matched_vehicle_id
    //
    //     WHERE 1 = 1
    //     "#,
    //     );
    //
    //     let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    //
    //     if let Some(value) = query.id {
    //         sql.push_str(" AND p.id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.author_id {
    //         sql.push_str(" AND p.author_id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.photo_type {
    //         sql.push_str(" AND p.photo_type = ?");
    //         params.push(Box::new(photo_type_to_i64(value)));
    //     }
    //
    //     if let Some(value) = &query.title {
    //         sql.push_str(" AND p.title LIKE '%' || ? || '%'");
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = &query.description {
    //         sql.push_str(" AND p.description LIKE '%' || ? || '%'");
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     // General search
    //     if let Some(value) = &query.search {
    //         sql.push_str(
    //             r#"
    //         AND (
    //             p.title LIKE '%' || ? || '%' OR
    //             p.description LIKE '%' || ? || '%' OR
    //             p.file_name LIKE '%' || ? || '%' OR
    //             bd.reg LIKE '%' || ? || '%' OR
    //             bd.fleet LIKE '%' || ? || '%' OR
    //             v.reg LIKE '%' || ? || '%' OR
    //             v.fleet LIKE '%' || ? || '%' OR
    //             td.fleet LIKE '%' || ? || '%' OR
    //             t.fleet LIKE '%' || ? || '%'
    //         )
    //         "#,
    //         );
    //
    //         for _ in 0..9 {
    //             params.push(Box::new(value.clone()));
    //         }
    //     }
    //
    //     // Bus details
    //     if let Some(value) = &query.bd_reg {
    //         sql.push_str(
    //             " AND (bd.reg LIKE '%' || ? || '%' OR v.reg LIKE '%' || ? || '%')"
    //         );
    //
    //         params.push(Box::new(value.clone()));
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = &query.bd_fleet {
    //         sql.push_str(
    //             " AND (bd.fleet LIKE '%' || ? || '%' OR v.fleet LIKE '%' || ? || '%')"
    //         );
    //
    //         params.push(Box::new(value.clone()));
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = query.bd_livery_id {
    //         sql.push_str(
    //             " AND (bd.livery_id = ? OR v.livery_id = ?)"
    //         );
    //
    //         params.push(Box::new(value));
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.bd_vehicle_type_id {
    //         sql.push_str(
    //             " AND (bd.vehicle_type_id = ? OR v.vehicle_type_id = ?)"
    //         );
    //
    //         params.push(Box::new(value));
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.bd_vehicle_id {
    //         sql.push_str(" AND bd.matched_vehicle_id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = &query.bd_route {
    //         sql.push_str(" AND bd.route_id = ?");
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = query.bd_has_matched_vehicle {
    //         if value {
    //             sql.push_str(" AND bd.matched_vehicle_id IS NOT NULL");
    //         } else {
    //             sql.push_str(" AND bd.matched_vehicle_id IS NULL");
    //         }
    //     }
    //
    //     // Tram details
    //     if let Some(value) = &query.td_fleet {
    //         sql.push_str(
    //             " AND (td.fleet LIKE '%' || ? || '%' OR t.fleet LIKE '%' || ? || '%')"
    //         );
    //
    //         params.push(Box::new(value.clone()));
    //         params.push(Box::new(value.clone()));
    //     }
    //
    //     if let Some(value) = query.td_livery_id {
    //         sql.push_str(
    //             " AND (td.livery_id = ? OR t.livery_id = ?)"
    //         );
    //
    //         params.push(Box::new(value));
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.td_vehicle_type_id {
    //         sql.push_str(
    //             " AND (td.vehicle_type_id = ? OR t.tram_type_id = ?)"
    //         );
    //
    //         params.push(Box::new(value));
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.td_operator_id {
    //         sql.push_str(
    //             " AND (td.operator_id = ? OR t.operator_id = ?)"
    //         );
    //
    //         params.push(Box::new(value));
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.td_vehicle_id {
    //         sql.push_str(" AND td.matched_vehicle_id = ?");
    //         params.push(Box::new(value));
    //     }
    //
    //     if let Some(value) = query.td_has_matched_vehicle {
    //         if value {
    //             sql.push_str(" AND td.matched_vehicle_id IS NOT NULL");
    //         } else {
    //             sql.push_str(" AND td.matched_vehicle_id IS NULL");
    //         }
    //     }
    //
    //     sql.push_str(" ORDER BY p.created_at DESC");
    //
    //     sql.push_str(" LIMIT ?");
    //     params.push(Box::new(page_length));
    //
    //     if let Some(offset) = query.offset {
    //         sql.push_str(" OFFSET ?");
    //         params.push(Box::new(offset));
    //     }
    //
    //     let mut statement = connection.prepare(&sql)?;
    //
    //     statement
    //         .query_map(
    //             rusqlite::params_from_iter(
    //                 params.iter().map(|param| param.as_ref()),
    //             ),
    //             Self::row_to_photo,
    //         )?
    //         .collect()
    // }

    

    pub fn create(
        &self,
        photo: &Photo,
    ) -> rusqlite::Result<Photo> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
        INSERT INTO photos (
            author_id,
            photo_type,
            obj_details_id,
            comment_thread,
            file_name,
            uploaded_id,
            title,
            description,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
            rusqlite::params![
            photo.author_id,
            photo_type_to_i64(photo.photo_type),
            photo.obj_details_id,
            photo.comment_thread,
            photo.file_name,
            photo.uploaded_id,
            photo.title,
            photo.description,
            photo.created_at,
        ],
        )?;

        let mut created_photo = photo.clone();
        created_photo.id = connection.last_insert_rowid();

        Ok(created_photo)
    }

    pub fn update(
        &self,
        photo: &Photo,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE photos
            SET
                author_id = ?,
                photo_type = ?,
                obj_details_id = ?,
                comment_thread = ?,
                file_name = ?,
                uploaded_id = ?,
                title = ?,
                description = ?,
                created_at = ?
            WHERE id = ?
            "#,
                    rusqlite::params![
                photo.author_id,
                photo_type_to_i64(photo.photo_type),
                photo.obj_details_id,
                photo.comment_thread,
                photo.file_name,
                photo.uploaded_id,
                photo.title,
                photo.description,
                photo.created_at,
                photo.id,
            ],
        )?;

        Ok(())
    }

    pub fn patch(
        &self,
        id: i64,
        update: &PhotoUpdate,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut sql = String::from("UPDATE photos SET ");
        let mut fields = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = &update.title {
            fields.push("title = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &update.description {
            fields.push("description = ?");
            params.push(Box::new(value.clone()));
        }

        if fields.is_empty() {
            return Ok(());
        }

        sql.push_str(&fields.join(", "));
        sql.push_str(" WHERE id = ?");
        params.push(Box::new(id));

        connection.execute(
            &sql,
            rusqlite::params_from_iter(
                params.iter().map(|param| param.as_ref()),
            ),
        )?;

        Ok(())
    }

    pub fn delete(
        &self,
        db: Arc<Database>,
        id: i64,
        user_id: i64,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let photo = db.photos.get(id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;

        connection.execute(
            "DELETE FROM photos WHERE id = ?",
            rusqlite::params![id],
        )?;

        db.favourites.delete(user_id, id, FavouriteType::Photo)?;
        if let Some(comment_thread) = photo.comment_thread {
            db.comment_threads.delete(comment_thread)?;
        }

        if let Some(details) = photo.obj_details_id {
            if let PhotoType::Bus = photo.photo_type {
                db.bus_photo_details.delete(details)?;
            }
        }

        Ok(())
    }

    pub fn get_or_create_comment_thread(
        &self,
        photo_id: i64,
    ) -> rusqlite::Result<i64> {
        let mut connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let tx = connection.transaction()?;

        let existing: Option<i64> = tx
            .query_row(
                "SELECT comment_thread FROM photos WHERE id = ?1",
                [photo_id],
                |row| row.get(0),
            )?;

        if let Some(thread_id) = existing {
            tx.commit()?;
            return Ok(thread_id);
        }

        let now = chrono::Utc::now().to_rfc3339();

        tx.execute(
            "INSERT INTO comment_threads (created_at) VALUES (?1)",
            [&now],
        )?;

        let thread_id = tx.last_insert_rowid();

        let updated = tx.execute(
            "UPDATE photos
         SET comment_thread = ?1
         WHERE id = ?2 AND comment_thread IS NULL",
            rusqlite::params![thread_id, photo_id],
        )?;

        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        tx.commit()?;

        Ok(thread_id)
    }

    pub fn get_bus_details(db: Arc<Database>, mut photo: Photo) -> rusqlite::Result<BusPhotoDetails> {
        if !matches!(photo.photo_type, PhotoType::Bus) {
            panic!("Photo's type is not PhotoType::Bus in get_bus_details");
        }

        if let Some(id) = photo.obj_details_id {
            let details = db.bus_photo_details.get(id)?.unwrap();

            Ok(details)
        } else {
            let details_id = db.bus_photo_details.create(&BusPhotoDetails {
                id: 0,
                matched_vehicle_id: None,
                override_match: false,
                reg: None,
                fleet: None,
                livery_id: None,
                vehicle_type_id: None,
                route_id: None,
            })?;

            let details = db.bus_photo_details.get(details_id)?.unwrap();

            photo.obj_details_id = Some(details.id);
            db.photos.update(&photo)?;

            Ok(details)
        }
    }

    pub fn get_tram_details(db: Arc<Database>, mut photo: Photo) -> rusqlite::Result<TramPhotoDetails> {
        if !matches!(photo.photo_type, PhotoType::Tram) {
            panic!("Photo's type is not PhotoType::Tram in get_tram_details");
        }

        if let Some(id) = photo.obj_details_id {
            let details = db.tram_photo_details.get(id)?.unwrap();

            Ok(details)
        } else {
            let details_id = db.tram_photo_details.create(&TramPhotoDetails {
                id: 0,
                matched_vehicle_id: None,
                override_match: false,
                fleet: None,
                livery_id: None,
                vehicle_type_id: None,
                operator_id: None,
            })?;

            let details = db.tram_photo_details.get(details_id)?.unwrap();

            photo.obj_details_id = Some(details.id);
            db.photos.update(&photo)?;

            Ok(details)
        }
    }

    pub fn get_model_bus_details(db: Arc<Database>, mut photo: Photo) -> rusqlite::Result<ModelBusPhotoDetails> {
        if !matches!(photo.photo_type, PhotoType::ModelBus) {
            panic!("Photo's type is not PhotoType::Tram in get_tram_details");
        }

        if let Some(id) = photo.obj_details_id {
            let details = db.model_bus_photo_details.get(id)?.unwrap();

            Ok(details)
        } else {
            let details_id = db.model_bus_photo_details.create(&ModelBusPhotoDetails {
                id: 0,
                matched_model_id: None,
                ownership: None,
                override_match: false,
            })?;

            let details = db.model_bus_photo_details.get(details_id)?.unwrap();

            photo.obj_details_id = Some(details.id);
            db.photos.update(&photo)?;

            Ok(details)
        }
    }

    pub fn fill_out(db: Arc<Database>, photos: Vec<Photo>, user_id: Option<i64>) -> rusqlite::Result<Vec<PhotoFilledOut>> {
        photos
            .into_iter()
            .map(|photo| {
                let details = match photo.photo_type {
                    PhotoType::Bus => {
                        let details = Self::get_bus_details(db.clone(), photo.clone())?;

                        let filled_out_details = BusPhotoDetailsManager::fill_out(db.clone(), details)?;
                        
                        Some(PhotoDetails::Bus(filled_out_details))
                    },
                    PhotoType::Tram => {
                        let details = Self::get_tram_details(db.clone(), photo.clone())?;

                        let filled_out_details = TramPhotoDetailsManager::fill_out(db.clone(), details)?;

                        Some(PhotoDetails::Tram(filled_out_details))
                    },
                    PhotoType::ModelBus => {
                        let details = Self::get_model_bus_details(db.clone(), photo.clone())?;

                        let filled_out_details = ModelBusPhotoDetailsManager::fill_out(db.clone(), details)?;

                        Some(PhotoDetails::ModelBus(filled_out_details))
                    }
                    _ => None,
                };

                let user = db.users.get(photo.author_id, false)?.unwrap();

                let favourite = match user_id {
                    Some(user) => {
                        db.favourites.get(user, FavouriteType::Photo, photo.id)?
                    },
                    None => None
                };

                Ok(PhotoFilledOut {
                    id: photo.id,
                    author_id: photo.author_id,
                    author: user,
                    photo_type: photo.photo_type,
                    obj_details_id: photo.obj_details_id,
                    obj_details: details,
                    comment_thread: Some(db.photos.get_or_create_comment_thread(photo.id)?),
                    file_name: photo.file_name,
                    uploaded_id: photo.uploaded_id,
                    title: photo.title,
                    description: photo.description,
                    created_at: photo.created_at,
                    
                    is_favourited: if let Some(_) = user_id {
                        Some(favourite.is_some())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub(crate) fn row_to_photo(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<Photo> {
        let photo_type: i64 = row.get("photo_type")?;

        let photo_type = match photo_type {
            0 => PhotoType::Bus,
            1 => PhotoType::ModelBus,
            2 => PhotoType::Train,
            3 => PhotoType::Tram,
            _ => {
                return Err(rusqlite::Error::InvalidColumnType(
                    2,
                    "photo_type".to_string(),
                    rusqlite::types::Type::Integer,
                ));
            }
        };

        Ok(Photo {
            id: row.get("id")?,
            author_id: row.get("author_id")?,
            photo_type,
            obj_details_id: row.get("obj_details_id")?,
            comment_thread: row.get("comment_thread")?,
            file_name: row.get("file_name")?,
            uploaded_id: row.get("uploaded_id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub(crate) fn photo_type_to_i64(photo_type: PhotoType) -> i64 {
    match photo_type {
        PhotoType::Bus => 0,
        PhotoType::ModelBus => 1,
        PhotoType::Train => 2,
        PhotoType::Tram => 3
    }
}