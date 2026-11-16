use crate::db::photo::{PhotoQuery, Photos};
use crate::db::resources::photo::{Photo, PhotoType};

pub fn query_only_photo_search(query: String) -> PhotoQuery {
    PhotoQuery {
        id: None,
        author_id: None,
        photo_type: None,
        title: None,
        description: None,
        search: Some(query),
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
        mbd_model_id: None,
        mbd_has_matched_model: None,
        mbd_owner_id: None,
        page_length: None,
        offset: None,
    }
}

impl Photos {
    pub fn search(
        &self,
        query: &PhotoQuery,
    ) -> rusqlite::Result<Vec<Photo>> {
        match query.photo_type {
            Some(PhotoType::Bus) => self.search_buses(query),
            Some(PhotoType::ModelBus) => self.search_model_buses(query),
            Some(PhotoType::Tram) => self.search_trams(query),
            Some(PhotoType::Train) => self.search_photos(query),
            None => self.search_all(query),
        }
    }

    fn photo_select() -> String {
        String::from(
            r#"
        SELECT
            p.id,
            p.author_id,
            p.photo_type,
            p.obj_details_id,
            p.comment_thread,
            p.file_name,
            p.uploaded_id,
            p.title,
            p.description,
            p.created_at
        FROM photos p
        "#
        )
    }

    fn add_common_filters(
        sql: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
        query: &PhotoQuery,
    ) {
        sql.push_str(" WHERE 1 = 1");

        if let Some(value) = query.id {
            sql.push_str(" AND p.id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.author_id {
            sql.push_str(" AND p.author_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.photo_type {
            sql.push_str(" AND p.photo_type = ?");
            params.push(Box::new(crate::db::photo::photo_type_to_i64(value)));
        }

        if let Some(value) = &query.title {
            sql.push_str(" AND p.title LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.description {
            sql.push_str(" AND p.description LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }
    }

    fn add_general_search(
        sql: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
        query: &PhotoQuery,
        fields: &[&str],
    ) {
        let Some(value) = &query.search else {
            return;
        };

        sql.push_str(" AND (");

        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                sql.push_str(" OR ");
            }

            sql.push_str(field);
            sql.push_str(" LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        sql.push(')');
    }

    fn finish_search(
        &self,
        mut sql: String,
        mut params: Vec<Box<dyn rusqlite::ToSql>>,
        query: &PhotoQuery,
    ) -> rusqlite::Result<Vec<Photo>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = match query.page_length {
            Some(x) => if x > 1000 { 1000 } else { x },
            None => 50,
        };

        sql.push_str(" ORDER BY p.created_at DESC");
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
                Self::row_to_photo,
            )?
            .collect()
    }

    fn search_buses(
        &self,
        query: &PhotoQuery,
    ) -> rusqlite::Result<Vec<Photo>> {
        let mut sql = Self::photo_select();

        sql.push_str(
            r#"
        LEFT JOIN bus_photo_details bd
            ON bd.id = p.obj_details_id

        LEFT JOIN bt_vehicles v
            ON v.id = bd.matched_vehicle_id
        "#
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        Self::add_common_filters(&mut sql, &mut params, query);

        Self::add_general_search(
            &mut sql,
            &mut params,
            query,
            &[
                "p.title",
                "p.description",
                "p.file_name",
                "bd.reg",
                "bd.fleet",
                "v.reg",
                "v.fleet",
            ],
        );

        Self::add_bus_filters(&mut sql, &mut params, query);

        self.finish_search(sql, params, query)
    }

    fn add_bus_filters(
        sql: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
        query: &PhotoQuery,
    ) {
        if let Some(value) = &query.bd_reg {
            sql.push_str(
                " AND (bd.reg LIKE '%' || ? || '%' OR v.reg LIKE '%' || ? || '%')"
            );

            params.push(Box::new(value.clone()));
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.bd_fleet {
            sql.push_str(
                " AND (bd.fleet LIKE '%' || ? || '%' OR v.fleet LIKE '%' || ? || '%')"
            );

            params.push(Box::new(value.clone()));
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.bd_livery_id {
            sql.push_str(" AND (bd.livery_id = ? OR v.livery_id = ?)");

            params.push(Box::new(value));
            params.push(Box::new(value));
        }

        if let Some(value) = query.bd_vehicle_type_id {
            sql.push_str(" AND (bd.vehicle_type_id = ? OR v.vehicle_type_id = ?)");

            params.push(Box::new(value));
            params.push(Box::new(value));
        }

        if let Some(value) = query.bd_operator_id.clone() {
            sql.push_str(" AND (v.operator_id = ?)");

            params.push(Box::new(value));
        }

        if let Some(value) = query.bd_vehicle_id {
            sql.push_str(" AND bd.matched_vehicle_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.bd_route {
            sql.push_str(" AND bd.route_id = ?");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.bd_has_matched_vehicle {
            if value {
                sql.push_str(" AND bd.matched_vehicle_id IS NOT NULL");
            } else {
                sql.push_str(" AND bd.matched_vehicle_id IS NULL");
            }
        }
    }

    fn search_trams(
        &self,
        query: &PhotoQuery,
    ) -> rusqlite::Result<Vec<Photo>> {
        let mut sql = Self::photo_select();

        sql.push_str(
            r#"
        LEFT JOIN tram_photo_details td
            ON td.id = p.obj_details_id

        LEFT JOIN trams t
            ON t.id = td.matched_vehicle_id
        "#
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        Self::add_common_filters(&mut sql, &mut params, query);

        Self::add_general_search(
            &mut sql,
            &mut params,
            query,
            &[
                "p.title",
                "p.description",
                "p.file_name",
                "td.fleet",
                "t.fleet",
            ],
        );

        Self::add_tram_filters(&mut sql, &mut params, query);

        self.finish_search(sql, params, query)
    }

    fn add_tram_filters(
        sql: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
        query: &PhotoQuery,
    ) {
        if let Some(value) = &query.td_fleet {
            sql.push_str(
                " AND (td.fleet LIKE '%' || ? || '%' OR t.fleet LIKE '%' || ? || '%')"
            );

            params.push(Box::new(value.clone()));
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.td_livery_id {
            sql.push_str(" AND (td.livery_id = ? OR t.livery_id = ?)");

            params.push(Box::new(value));
            params.push(Box::new(value));
        }

        if let Some(value) = query.td_vehicle_type_id {
            sql.push_str(" AND (td.vehicle_type_id = ? OR t.tram_type_id = ?)");

            params.push(Box::new(value));
            params.push(Box::new(value));
        }

        if let Some(value) = query.td_operator_id {
            sql.push_str(" AND (td.operator_id = ? OR t.operator_id = ?)");

            params.push(Box::new(value));
            params.push(Box::new(value));
        }

        if let Some(value) = query.td_vehicle_id {
            sql.push_str(" AND td.matched_vehicle_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.td_has_matched_vehicle {
            if value {
                sql.push_str(" AND td.matched_vehicle_id IS NOT NULL");
            } else {
                sql.push_str(" AND td.matched_vehicle_id IS NULL");
            }
        }
    }

    fn search_model_buses(
        &self,
        query: &PhotoQuery,
    ) -> rusqlite::Result<Vec<Photo>> {
        let mut sql = Self::photo_select();

        sql.push_str(
            r#"
        LEFT JOIN model_bus_photo_details mbd
            ON mbd.id = p.obj_details_id

        LEFT JOIN model_buses mb
            ON mb.id = mbd.matched_model_id

        LEFT JOIN model_bus_ownerships mbo
            ON mbo.id = mbd.ownership
        "#
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        Self::add_common_filters(&mut sql, &mut params, query);

        Self::add_general_search(
            &mut sql,
            &mut params,
            query,
            &[
                "p.title",
                "p.description",
                "p.file_name",
                "mb.manufacturer",
                "mb.product_code",
                "mb.fleet_number",
                "mb.registration",
                "mb.operator",
                "mb.livery",
                "mb.route_number",
                "mb.destination",
                "mb.vehicle_type",
            ],
        );

        Self::add_model_bus_filters(&mut sql, &mut params, query);

        self.finish_search(sql, params, query)
    }

    fn add_model_bus_filters(
        sql: &mut String,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
        query: &PhotoQuery,
    ) {
        if let Some(value) = &query.mbd_manufacturer {
            sql.push_str(" AND mb.manufacturer LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.mbd_product_code {
            sql.push_str(" AND mb.product_code LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.mbd_scale {
            sql.push_str(" AND mb.scale = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = &query.mbd_operator {
            sql.push_str(" AND mb.operator LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.mbd_livery {
            sql.push_str(" AND mb.livery LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.mbd_fleet_number {
            sql.push_str(" AND mb.fleet_number LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.mbd_registration {
            sql.push_str(" AND mb.registration LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.mbd_route_number {
            sql.push_str(" AND mb.route_number LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.mbd_destination {
            sql.push_str(" AND mb.destination LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.mbd_vehicle_type {
            sql.push_str(" AND mb.vehicle_type LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = query.mbd_has_certificate {
            sql.push_str(" AND mb.has_certificate = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.mbd_released {
            sql.push_str(" AND mb.released = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.mbd_model_id {
            sql.push_str(" AND mbd.matched_model_id = ?");
            params.push(Box::new(value));
        }

        if let Some(value) = query.mbd_has_matched_model {
            if value {
                sql.push_str(" AND mbd.matched_model_id IS NOT NULL");
            } else {
                sql.push_str(" AND mbd.matched_model_id IS NULL");
            }
        }

        if let Some(value) = query.mbd_owner_id {
            sql.push_str(" AND mbo.user_id = ?");
            params.push(Box::new(value));
        }
    }

    fn search_photos(
        &self,
        query: &PhotoQuery,
    ) -> rusqlite::Result<Vec<Photo>> {
        let mut sql = Self::photo_select();

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        Self::add_common_filters(&mut sql, &mut params, query);

        Self::add_general_search(
            &mut sql,
            &mut params,
            query,
            &[
                "p.title",
                "p.description",
                "p.file_name",
            ],
        );

        self.finish_search(sql, params, query)
    }

    fn search_all(
        &self,
        query: &PhotoQuery,
    ) -> rusqlite::Result<Vec<Photo>> {
        let mut sql = Self::photo_select();

        sql.push_str(
            r#"
        LEFT JOIN bus_photo_details bd
            ON bd.id = p.obj_details_id
            AND p.photo_type = 0

        LEFT JOIN bt_vehicles v
            ON v.id = bd.matched_vehicle_id

        LEFT JOIN tram_photo_details td
            ON td.id = p.obj_details_id
            AND p.photo_type = 3

        LEFT JOIN trams t
            ON t.id = td.matched_vehicle_id

        LEFT JOIN model_bus_photo_details mbd
            ON mbd.id = p.obj_details_id
            AND p.photo_type = 1

        LEFT JOIN model_buses mb
            ON mb.id = mbd.matched_model_id

        LEFT JOIN model_bus_ownerships mbo
            ON mbo.id = mbd.ownership
        "#
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        Self::add_common_filters(&mut sql, &mut params, query);

        Self::add_general_search(
            &mut sql,
            &mut params,
            query,
            &[
                "p.title",
                "p.description",
                "p.file_name",

                // Bus
                "bd.reg",
                "bd.fleet",
                "v.reg",
                "v.fleet",

                // Tram
                "td.fleet",
                "t.fleet",

                // Model bus
                "mb.manufacturer",
                "mb.product_code",
                "mb.fleet_number",
                "mb.registration",
                "mb.operator",
                "mb.livery",
                "mb.route_number",
                "mb.destination",
                "mb.vehicle_type",
            ],
        );

        Self::add_bus_filters(&mut sql, &mut params, query);
        Self::add_tram_filters(&mut sql, &mut params, query);
        Self::add_model_bus_filters(&mut sql, &mut params, query);

        self.finish_search(sql, params, query)
    }
}