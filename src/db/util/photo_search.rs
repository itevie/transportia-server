use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::db::resource::photo::photo::{PhotoOrderBy, PhotoQuery, Photos};
use crate::db::resource::photo::photo_def::{Photo, PhotoType};

impl Photos {
    pub fn query_only_search(query: String) -> PhotoQuery {
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
            order_by: None,
        }
    }
    pub async fn search(&self, query: &PhotoQuery) -> Result<Vec<Photo>, sqlx::Error> {
        match query.photo_type {
            Some(PhotoType::Bus) => self.search_buses(query).await,
            Some(PhotoType::ModelBus) => self.search_model_buses(query).await,
            Some(PhotoType::Tram) => self.search_trams(query).await,
            Some(PhotoType::Train) => self.search_photos(query).await,
            None => self.search_all(query).await,
        }
    }

    fn photo_select() -> QueryBuilder<Postgres> {
        QueryBuilder::new(
            r#"
            SELECT p.*
            FROM photos p
            "#,
        )
    }

    fn add_common_filters(builder: &mut QueryBuilder<Postgres>, query: &PhotoQuery) {
        builder.push(" WHERE 1 = 1");

        if let Some(value) = query.id {
            builder.push(" AND p.id = ").push_bind(value);
        }

        if let Some(value) = query.author_id {
            builder.push(" AND p.author_id = ").push_bind(value);
        }

        if let Some(value) = query.photo_type {
            builder
                .push(" AND p.photo_type = ")
                .push_bind(crate::db::resource::photo::photo::photo_type_to_i64(value));
        }

        if let Some(value) = &query.title {
            builder
                .push(" AND p.title ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.description {
            builder
                .push(" AND p.description ILIKE ")
                .push_bind(format!("%{}%", value));
        }
    }

    fn add_general_search(
        builder: &mut QueryBuilder<Postgres>,
        query: &PhotoQuery,
        fields: &[&str],
    ) {
        let Some(value) = &query.search else {
            return;
        };

        builder.push(" AND (");

        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                builder.push(" OR ");
            }

            // These field names are internal constants, never user input.
            builder
                .push(*field)
                .push(" ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        builder.push(")");
    }

    async fn finish_search(
        &self,
        mut builder: QueryBuilder<Postgres>,
        query: &PhotoQuery,
    ) -> Result<Vec<Photo>, sqlx::Error> {
        let page_length = query.page_length.unwrap_or(50).min(1000);

        match query.order_by.unwrap_or_default() {
            PhotoOrderBy::Newest => {
                builder.push(" ORDER BY p.created_at DESC");
            }
            PhotoOrderBy::Oldest => {
                builder.push(" ORDER BY p.created_at ASC");
            }
            PhotoOrderBy::HighestRated => {
                builder.push(
                    r#"
                    ORDER BY
                        (
                            SELECT AVG(r.rating)
                            FROM photo_ratings r
                            WHERE r.photo_id = p.id
                        ) DESC NULLS LAST,
                        (
                            SELECT COUNT(*)
                            FROM photo_ratings r
                            WHERE r.photo_id = p.id
                        ) DESC
                    "#,
                );
            }
            PhotoOrderBy::LowestRated => {
                builder.push(
                    r#"
                    ORDER BY
                        (
                            SELECT AVG(r.rating)
                            FROM photo_ratings r
                            WHERE r.photo_id = p.id
                        ) ASC NULLS LAST,
                        (
                            SELECT COUNT(*)
                            FROM photo_ratings r
                            WHERE r.photo_id = p.id
                        ) DESC
                    "#,
                );
            }
        }

        builder.push(" LIMIT ").push_bind(page_length);

        if let Some(offset) = query.offset {
            builder.push(" OFFSET ").push_bind(offset);
        }

        builder
            .build_query_as::<Photo>()
            .fetch_all(&self.pool)
            .await
    }

    async fn search_buses(&self, query: &PhotoQuery) -> Result<Vec<Photo>, sqlx::Error> {
        let mut builder = Self::photo_select();

        builder.push(
            r#"
            LEFT JOIN bus_photo_details bd
                ON bd.id = p.obj_details_id

            LEFT JOIN bt_vehicles v
                ON v.id = bd.matched_vehicle_id
            "#,
        );

        Self::add_common_filters(&mut builder, query);

        Self::add_general_search(
            &mut builder,
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

        Self::add_bus_filters(&mut builder, query);

        self.finish_search(builder, query).await
    }

    fn add_bus_filters(builder: &mut QueryBuilder<Postgres>, query: &PhotoQuery) {
        if let Some(value) = &query.bd_reg {
            let search = format!("%{}%", value);

            builder
                .push(" AND (bd.reg ILIKE ")
                .push_bind(search.clone())
                .push(" OR v.reg ILIKE ")
                .push_bind(search)
                .push(")");
        }

        if let Some(value) = &query.bd_fleet {
            let search = format!("%{}%", value);

            builder
                .push(" AND (bd.fleet ILIKE ")
                .push_bind(search.clone())
                .push(" OR v.fleet ILIKE ")
                .push_bind(search)
                .push(")");
        }

        if let Some(value) = query.bd_livery_id {
            builder
                .push(" AND (bd.livery_id = ")
                .push_bind(value)
                .push(" OR v.livery_id = ")
                .push_bind(value)
                .push(")");
        }

        if let Some(value) = query.bd_vehicle_type_id {
            builder
                .push(" AND (bd.vehicle_type_id = ")
                .push_bind(value)
                .push(" OR v.vehicle_type_id = ")
                .push_bind(value)
                .push(")");
        }

        if let Some(value) = &query.bd_operator_id {
            builder
                .push(" AND v.operator_id = ")
                .push_bind(value.clone());
        }

        if let Some(value) = query.bd_vehicle_id {
            builder
                .push(" AND bd.matched_vehicle_id = ")
                .push_bind(value);
        }

        if let Some(value) = &query.bd_route {
            builder.push(" AND bd.route_id = ").push_bind(value.clone());
        }

        if let Some(value) = query.bd_has_matched_vehicle {
            if value {
                builder.push(" AND bd.matched_vehicle_id IS NOT NULL");
            } else {
                builder.push(" AND bd.matched_vehicle_id IS NULL");
            }
        }
    }

    async fn search_trams(&self, query: &PhotoQuery) -> Result<Vec<Photo>, sqlx::Error> {
        let mut builder = Self::photo_select();

        builder.push(
            r#"
            LEFT JOIN tram_photo_details td
                ON td.id = p.obj_details_id

            LEFT JOIN trams t
                ON t.id = td.matched_vehicle_id
            "#,
        );

        Self::add_common_filters(&mut builder, query);

        Self::add_general_search(
            &mut builder,
            query,
            &[
                "p.title",
                "p.description",
                "p.file_name",
                "td.fleet",
                "t.fleet",
            ],
        );

        Self::add_tram_filters(&mut builder, query);

        self.finish_search(builder, query).await
    }

    fn add_tram_filters(builder: &mut QueryBuilder<Postgres>, query: &PhotoQuery) {
        if let Some(value) = &query.td_fleet {
            let search = format!("%{}%", value);

            builder
                .push(" AND (td.fleet ILIKE ")
                .push_bind(search.clone())
                .push(" OR t.fleet ILIKE ")
                .push_bind(search)
                .push(")");
        }

        if let Some(value) = query.td_livery_id {
            builder
                .push(" AND (td.livery_id = ")
                .push_bind(value)
                .push(" OR t.livery_id = ")
                .push_bind(value)
                .push(")");
        }

        if let Some(value) = query.td_vehicle_type_id {
            builder
                .push(" AND (td.vehicle_type_id = ")
                .push_bind(value)
                .push(" OR t.tram_type_id = ")
                .push_bind(value)
                .push(")");
        }

        if let Some(value) = query.td_operator_id {
            builder
                .push(" AND (td.operator_id = ")
                .push_bind(value)
                .push(" OR t.operator_id = ")
                .push_bind(value)
                .push(")");
        }

        if let Some(value) = query.td_vehicle_id {
            builder
                .push(" AND td.matched_vehicle_id = ")
                .push_bind(value);
        }

        if let Some(value) = query.td_has_matched_vehicle {
            if value {
                builder.push(" AND td.matched_vehicle_id IS NOT NULL");
            } else {
                builder.push(" AND td.matched_vehicle_id IS NULL");
            }
        }
    }

    async fn search_model_buses(&self, query: &PhotoQuery) -> Result<Vec<Photo>, sqlx::Error> {
        let mut builder = Self::photo_select();

        builder.push(
            r#"
            LEFT JOIN model_bus_photo_details mbd
                ON mbd.id = p.obj_details_id

            LEFT JOIN model_buses mb
                ON mb.id = mbd.matched_model_id

            LEFT JOIN model_bus_ownerships mbo
                ON mbo.id = mbd.ownership
            "#,
        );

        Self::add_common_filters(&mut builder, query);

        Self::add_general_search(
            &mut builder,
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

        Self::add_model_bus_filters(&mut builder, query);

        self.finish_search(builder, query).await
    }

    fn add_model_bus_filters(builder: &mut QueryBuilder<Postgres>, query: &PhotoQuery) {
        if let Some(value) = &query.mbd_manufacturer {
            builder
                .push(" AND mb.manufacturer ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.mbd_product_code {
            builder
                .push(" AND mb.product_code ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = query.mbd_scale {
            builder.push(" AND mb.scale = ").push_bind(value);
        }

        if let Some(value) = &query.mbd_operator {
            builder
                .push(" AND mb.operator ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.mbd_livery {
            builder
                .push(" AND mb.livery ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.mbd_fleet_number {
            builder
                .push(" AND mb.fleet_number ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.mbd_registration {
            builder
                .push(" AND mb.registration ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.mbd_route_number {
            builder
                .push(" AND mb.route_number ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.mbd_destination {
            builder
                .push(" AND mb.destination ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = &query.mbd_vehicle_type {
            builder
                .push(" AND mb.vehicle_type ILIKE ")
                .push_bind(format!("%{}%", value));
        }

        if let Some(value) = query.mbd_has_certificate {
            builder.push(" AND mb.has_certificate = ").push_bind(value);
        }

        if let Some(value) = query.mbd_released {
            builder.push(" AND mb.released = ").push_bind(value);
        }

        if let Some(value) = query.mbd_model_id {
            builder
                .push(" AND mbd.matched_model_id = ")
                .push_bind(value);
        }

        if let Some(value) = query.mbd_has_matched_model {
            if value {
                builder.push(" AND mbd.matched_model_id IS NOT NULL");
            } else {
                builder.push(" AND mbd.matched_model_id IS NULL");
            }
        }

        if let Some(value) = query.mbd_owner_id {
            builder.push(" AND mbo.user_id = ").push_bind(value);
        }
    }

    async fn search_photos(&self, query: &PhotoQuery) -> Result<Vec<Photo>, sqlx::Error> {
        let mut builder = Self::photo_select();

        Self::add_common_filters(&mut builder, query);

        Self::add_general_search(
            &mut builder,
            query,
            &["p.title", "p.description", "p.file_name"],
        );

        self.finish_search(builder, query).await
    }

    async fn search_all(&self, query: &PhotoQuery) -> Result<Vec<Photo>, sqlx::Error> {
        let mut builder = Self::photo_select();

        builder.push(
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
            "#,
        );

        Self::add_common_filters(&mut builder, query);

        Self::add_general_search(
            &mut builder,
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

        Self::add_bus_filters(&mut builder, query);
        Self::add_tram_filters(&mut builder, query);
        Self::add_model_bus_filters(&mut builder, query);

        self.finish_search(builder, query).await
    }
}
