use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::{Deserialize, Serialize};
use crate::db::resources::bus_listing::BusListingResource;

pub static SQL_BUS_LISTING: &str = r#"
CREATE TABLE IF NOT EXISTS bus_listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    external_id TEXT NOT NULL,
    origin TEXT NOT NULL,
    origin_code TEXT NOT NULL,
    fetched_from TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    link TEXT,
    scale TEXT,
    manufacturer TEXT,
    manufacturer_code TEXT,
    price_low REAL NOT NULL,
    price_high REAL NOT NULL,
    currency TEXT,
    image_list TEXT,
    seller_name TEXT,
    seller_link TEXT,
    condition TEXT,
    listed_at TEXT,
    ends_at TEXT,
    fetched_at TEXT NOT NULL,
    category TEXT,
    source_url TEXT,
    total INTEGER,
    lowest INTEGER,
    sold_out BOOLEAN,
    some_sold_out BOOLEAN,

    UNIQUE(external_id, origin_code)
)
"#;

const LISTING_COLUMNS: &str = r#"
    id,
    external_id,
    origin,
    origin_code,
    fetched_from,
    title,
    description,
    link,
    scale,
    manufacturer,
    manufacturer_code,
    price_low,
    price_high,
    currency,
    image_list,
    seller_name,
    seller_link,
    condition,
    listed_at,
    ends_at,
    fetched_at,
    category,
    total,
    lowest,
    sold_out,
    some_sold_out
"#;

#[derive(Deserialize, Serialize)]
pub struct BusListingQuery {
    pub scale: Option<String>,
    pub origin: Option<String>,
    pub title: Option<String>,
    pub manufacturer: Option<String>,
    pub manufacturer_code: Option<String>,
    pub price_low: Option<String>,
    pub price_high: Option<String>,
    pub condition: Option<String>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>
}

#[derive(Serialize)]
pub struct BusListingQueryOptions {
    pub scale: Vec<String>,
    pub origin_code: Vec<String>,
    pub manufacturer: Vec<String>,
    pub manufacturer_code: Vec<String>,
    pub condition: Vec<String>,
}

#[derive(Clone)]
pub struct BusListings {
    pool: Pool<SqliteConnectionManager>,
}

impl BusListings {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn fetch(&self) -> rusqlite::Result<Vec<BusListingResource>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            &format!(
                "SELECT {} FROM bus_listings",
                LISTING_COLUMNS
            )
        )?;

        statement
            .query_map([], BusListings::row_to_listing)?
            .collect()
    }

    pub fn search(
        &self,
        query: &BusListingQuery,
    ) -> rusqlite::Result<Vec<BusListingResource>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut sql = format!(
            "SELECT {} FROM bus_listings WHERE 1 = 1",
            LISTING_COLUMNS
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = &query.scale {
            sql.push_str(" AND scale LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.origin {
            sql.push_str(" AND origin LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.title {
            sql.push_str(" AND title LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.manufacturer {
            sql.push_str(" AND manufacturer LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.manufacturer_code {
            sql.push_str(" AND manufacturer_code LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.condition {
            sql.push_str(" AND condition LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.category {
            sql.push_str(" AND category LIKE '%' || ? || '%'");
            params.push(Box::new(value.clone()));
        }

        if let Some(value) = &query.price_low {
            if let Ok(value) = value.parse::<f64>() {
                sql.push_str(" AND price_low >= ?");
                params.push(Box::new(value));
            }
        }

        if let Some(value) = &query.price_high {
            if let Ok(value) = value.parse::<f64>() {
                sql.push_str(" AND price_high <= ?");
                params.push(Box::new(value));
            }
        }

        if let Some(value) = &query.search {
            sql.push_str(
                r#"
        AND (
            title LIKE '%' || ? || '%' OR
            description LIKE '%' || ? || '%' OR
            manufacturer LIKE '%' || ? || '%' OR
            manufacturer_code LIKE '%' || ? || '%' OR
            scale LIKE '%' || ? || '%' OR
            origin LIKE '%' || ? || '%' OR
            origin_code LIKE '%' || ? || '%' OR
            condition LIKE '%' || ? || '%' OR
            category LIKE '%' || ? || '%' OR
            seller_name LIKE '%' || ? || '%'
        )
        "#,
            );

            for _ in 0..10 {
                params.push(Box::new(value.clone()));
            }
        }

        if let Some(sort) = &query.sort {
            match sort.as_str() {
                "price-low-high" => {
                    sql.push_str(" ORDER BY price_low ASC");
                }
                "price-high-low" => {
                    sql.push_str(" ORDER BY price_low DESC");
                }
                _ => {}
            }
        }

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref())
                ),
                BusListings::row_to_listing,
            )?
            .collect()
    }

    pub fn get_query_options(&self) -> rusqlite::Result<BusListingQueryOptions> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let get_values = |column: &str| -> rusqlite::Result<Vec<String>> {
            let sql = format!(
                r#"
            SELECT DISTINCT {column}
            FROM bus_listings
            WHERE {column} IS NOT NULL
              AND {column} != ''
            ORDER BY {column}
            "#
            );

            let mut statement = connection.prepare(&sql)?;

            let values = statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect();

            values
        };

        Ok(BusListingQueryOptions {
            scale: get_values("scale")?,
            origin_code: get_values("origin_code")?,
            manufacturer: get_values("manufacturer")?,
            manufacturer_code: get_values("manufacturer_code")?,
            condition: get_values("condition")?,
        })
    }

    pub fn create(
        &self,
        listing: &BusListingResource,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO bus_listings (
                external_id,
                origin,
                fetched_from,
                title,
                description,
                link,
                scale,
                manufacturer,
                manufacturer_code,
                price_low,
                price_high,
                currency,
                image_list,
                seller_name,
                seller_link,
                condition,
                listed_at,
                ends_at,
                fetched_at,
                category,
                total,
                lowest,
                sold_out,
                some_sold_out
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &listing.external_id,
                &listing.origin,
                &listing.fetched_from,
                &listing.title,
                &listing.description,
                &listing.link,
                &listing.scale,
                &listing.manufacturer,
                &listing.manufacturer_code,
                listing.price_low,
                listing.price_high,
                &listing.currency,
                &listing.image_list,
                &listing.seller_name,
                &listing.seller_link,
                &listing.condition,
                &listing.listed_at,
                &listing.ends_at,
                &listing.fetched_at,
                &listing.category,
                listing.total,
                listing.lowest,
                listing.sold_out,
                listing.some_sold_out,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(
        &self,
        listing: &BusListingResource,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE bus_listings SET
                external_id = ?,
                origin = ?,
                origin_code = ?,
                fetched_from = ?,
                title = ?,
                description = ?,
                link = ?,
                scale = ?,
                manufacturer = ?,
                manufacturer_code = ?,
                price_low = ?,
                price_high = ?,
                currency = ?,
                image_list = ?,
                seller_name = ?,
                seller_link = ?,
                condition = ?,
                listed_at = ?,
                ends_at = ?,
                fetched_at = ?,
                category = ?,
                total = ?,
                lowest = ?,
                sold_out = ?,
                some_sold_out = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &listing.external_id,
                &listing.origin,
                &listing.origin_code,
                &listing.fetched_from,
                &listing.title,
                &listing.description,
                &listing.link,
                &listing.scale,
                &listing.manufacturer,
                &listing.manufacturer_code,
                listing.price_low,
                listing.price_high,
                &listing.currency,
                &listing.image_list,
                &listing.seller_name,
                &listing.seller_link,
                &listing.condition,
                &listing.listed_at,
                &listing.ends_at,
                &listing.fetched_at,
                &listing.category,
                listing.total,
                listing.lowest,
                listing.sold_out,
                listing.some_sold_out,
                listing.id,
            ],
        )?;

        Ok(())
    }

    pub fn upsert(
        &self,
        listing: &BusListingResource,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
        INSERT INTO bus_listings (
            external_id,
            origin,
            origin_code,
            fetched_from,
            title,
            description,
            link,
            scale,
            manufacturer,
            manufacturer_code,
            price_low,
            price_high,
            currency,
            image_list,
            seller_name,
            seller_link,
            condition,
            listed_at,
            ends_at,
            fetched_at,
            category,
            total,
            lowest,
            sold_out,
            some_sold_out
        )
        VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            ?, ?, ?, ?, ?
        )
        ON CONFLICT (external_id, origin_code) DO UPDATE SET
            origin = excluded.origin,
            fetched_from = excluded.fetched_from,
            title = excluded.title,
            description = excluded.description,
            link = excluded.link,
            scale = excluded.scale,
            manufacturer = excluded.manufacturer,
            manufacturer_code = excluded.manufacturer_code,
            price_low = excluded.price_low,
            price_high = excluded.price_high,
            currency = excluded.currency,
            image_list = excluded.image_list,
            seller_name = excluded.seller_name,
            seller_link = excluded.seller_link,
            condition = excluded.condition,
            listed_at = excluded.listed_at,
            ends_at = excluded.ends_at,
            fetched_at = excluded.fetched_at,
            category = excluded.category,
            total = excluded.total,
            lowest = excluded.lowest,
            sold_out = excluded.sold_out,
            some_sold_out = excluded.some_sold_out
        "#,
            rusqlite::params![
            &listing.external_id,
            &listing.origin,
            &listing.origin_code,
            &listing.fetched_from,
            &listing.title,
            &listing.description,
            &listing.link,
            &listing.scale,
            &listing.manufacturer,
            &listing.manufacturer_code,
            listing.price_low,
            listing.price_high,
            &listing.currency,
            &listing.image_list,
            &listing.seller_name,
            &listing.seller_link,
            &listing.condition,
            &listing.listed_at,
            &listing.ends_at,
            &listing.fetched_at,
            &listing.category,
            listing.total,
            listing.lowest,
            listing.sold_out,
            listing.some_sold_out,
        ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM bus_listings WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_listing(row: &rusqlite::Row) -> rusqlite::Result<BusListingResource> {
        Ok(BusListingResource {
            id: row.get("id")?,
            external_id: row.get("external_id")?,
            origin: row.get("origin")?,
            origin_code: row.get("origin_code")?,
            fetched_from: row.get("fetched_from")?,
            title: row.get("title")?,
            description: row.get("description")?,
            link: row.get("link")?,
            scale: row.get("scale")?,
            manufacturer: row.get("manufacturer")?,
            manufacturer_code: row.get("manufacturer_code")?,
            price_low: row.get("price_low")?,
            price_high: row.get("price_high")?,
            currency: row.get("currency")?,
            image_list: row.get("image_list")?,
            seller_name: row.get("seller_name")?,
            seller_link: row.get("seller_link")?,
            condition: row.get("condition")?,
            listed_at: row.get("listed_at")?,
            ends_at: row.get("ends_at")?,
            fetched_at: row.get("fetched_at")?,
            category: row.get("category")?,
            total: row.get("total")?,
            lowest: row.get("lowest")?,
            sold_out: row.get("sold_out")?,
            some_sold_out: row.get("some_sold_out")?,
        })
    }
}