use chrono::{Datelike, Local, Weekday};
use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::db::resources::tx_vehicle_journey::TxVehicleJourney;

pub static SQL_TX_VEHICLE_JOURNEY: &str = r#"
CREATE TABLE IF NOT EXISTS tx_vehicle_journeys (
    id TEXT PRIMARY KEY,
    operator_ref TEXT,
    service_ref TEXT,
    line_ref TEXT,
    journey_pattern_ref TEXT,
    departure_time TEXT,
    garage_ref TEXT,
    block_description TEXT,
    block_number TEXT,
    ticket_machine_service_code TEXT,
    ticket_machine_journey_code TEXT,
    monday INTEGER NOT NULL DEFAULT 0,
    tuesday INTEGER NOT NULL DEFAULT 0,
    wednesday INTEGER NOT NULL DEFAULT 0,
    thursday INTEGER NOT NULL DEFAULT 0,
    friday INTEGER NOT NULL DEFAULT 0,
    saturday INTEGER NOT NULL DEFAULT 0,
    sunday INTEGER NOT NULL DEFAULT 0
)
"#;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TxVehicleJourneyQuery {
    pub search: Option<String>,

    pub id: Option<String>,
    pub operator_ref: Option<String>,
    pub service_ref: Option<String>,
    pub line_ref: Option<String>,
    pub journey_pattern_ref: Option<String>,
    pub departure_time: Option<String>,
    pub garage_ref: Option<String>,

    pub block_description: Option<String>,
    pub block_number: Option<String>,

    pub ticket_machine_service_code: Option<String>,
    pub ticket_machine_journey_code: Option<String>,

    pub monday: Option<bool>,
    pub tuesday: Option<bool>,
    pub wednesday: Option<bool>,
    pub thursday: Option<bool>,
    pub friday: Option<bool>,
    pub saturday: Option<bool>,
    pub sunday: Option<bool>,

    pub operating_today: Option<bool>,

    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone)]
pub struct TxVehicleJourneys {
    pool: Pool<SqliteConnectionManager>,
}

impl TxVehicleJourneys {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(
        &self,
        id: &str,
    ) -> rusqlite::Result<Option<TxVehicleJourney>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT *
            FROM tx_vehicle_journeys
            WHERE id = ?
            "#,
        )?;

        let mut rows =
            statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_vehicle_journey(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(
        &self,
        query: &TxVehicleJourneyQuery,
    ) -> rusqlite::Result<Vec<TxVehicleJourney>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);
        let offset = query.offset.unwrap_or(0).max(0);

        let mut sql = String::from(
            r#"
        SELECT v.*
        FROM tx_vehicle_journeys v
        WHERE 1 = 1
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if query.operating_today == Some(true) {
            let today = Local::now().date_naive();
            let today_string = today.format("%Y-%m-%d").to_string();

            let weekday_column = match today.weekday() {
                Weekday::Mon => "monday",
                Weekday::Tue => "tuesday",
                Weekday::Wed => "wednesday",
                Weekday::Thu => "thursday",
                Weekday::Fri => "friday",
                Weekday::Sat => "saturday",
                Weekday::Sun => "sunday",
            };

            //
            // AND v.{weekday_column} = 1
            //

            sql.push_str(&format!(
                r#"
            AND EXISTS (
                SELECT 1
                FROM tx_services s
                WHERE s.service_code = v.service_ref
                  AND s.{weekday_column} = 1
                  AND (
                      s.start_date IS NULL
                      OR date(s.start_date) <= date(?)
                  )
                  AND (
                      s.end_date IS NULL
                      OR date(s.end_date) >= date(?)
                  )
            )
            "#
            ));

            params.push(Box::new(today_string.clone()));
            params.push(Box::new(today_string));
        }

        macro_rules! like_filter {
        ($field:ident) => {
            if let Some(value) = &query.$field {
                sql.push_str(concat!(
                    " AND v.",
                    stringify!($field),
                    " LIKE '%' || ? || '%'"
                ));
                params.push(Box::new(value.clone()));
            }
        };
    }

        macro_rules! exact_filter {
        ($field:ident) => {
            if let Some(value) = &query.$field {
                sql.push_str(concat!(
                    " AND v.",
                    stringify!($field),
                    " = ?"
                ));
                params.push(Box::new(value.clone()));
            }
        };
    }

        like_filter!(id);
        exact_filter!(operator_ref);
        like_filter!(service_ref);
        like_filter!(line_ref);
        like_filter!(journey_pattern_ref);
        exact_filter!(departure_time);
        exact_filter!(garage_ref);
        like_filter!(block_description);
        exact_filter!(block_number);
        exact_filter!(ticket_machine_service_code);
        exact_filter!(ticket_machine_journey_code);

        macro_rules! bool_filter {
        ($field:ident) => {
            if let Some(value) = query.$field {
                sql.push_str(concat!(
                    " AND v.",
                    stringify!($field),
                    " = ?"
                ));
                params.push(Box::new(value));
            }
        };
    }

        bool_filter!(monday);
        bool_filter!(tuesday);
        bool_filter!(wednesday);
        bool_filter!(thursday);
        bool_filter!(friday);
        bool_filter!(saturday);
        bool_filter!(sunday);

        if let Some(value) = &query.search {
            for word in value.split_whitespace() {
                sql.push_str(
                    r#"
                AND (
                    v.id LIKE '%' || ? || '%' OR
                    v.operator_ref LIKE '%' || ? || '%' OR
                    v.service_ref LIKE '%' || ? || '%' OR
                    v.line_ref LIKE '%' || ? || '%' OR
                    v.journey_pattern_ref LIKE '%' || ? || '%' OR
                    v.departure_time LIKE '%' || ? || '%' OR
                    v.garage_ref LIKE '%' || ? || '%' OR
                    v.block_description LIKE '%' || ? || '%' OR
                    v.block_number LIKE '%' || ? || '%' OR
                    v.ticket_machine_service_code LIKE '%' || ? || '%' OR
                    v.ticket_machine_journey_code LIKE '%' || ? || '%'
                )
                "#,
                );

                for _ in 0..11 {
                    params.push(Box::new(word.to_string()));
                }
            }
        }

        sql.push_str(" ORDER BY v.departure_time, v.id");
        sql.push_str(" LIMIT ?");
        params.push(Box::new(page_length));
        sql.push_str(" OFFSET ?");
        params.push(Box::new(offset));

        println!("{}", sql);

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_tx_vehicle_journey,
            )?
            .collect()
    }

    pub fn get_by_line_ref(
        &self,
        line_ref: &str,
    ) -> rusqlite::Result<Vec<TxVehicleJourney>> {
        self.get_by_column("line_ref", line_ref)
    }

    pub fn get_by_service_ref(
        &self,
        service_ref: &str,
    ) -> rusqlite::Result<Vec<TxVehicleJourney>> {
        self.get_by_column("service_ref", service_ref)
    }

    pub fn get_by_journey_pattern_ref(
        &self,
        journey_pattern_ref: &str,
    ) -> rusqlite::Result<Vec<TxVehicleJourney>> {
        self.get_by_column("journey_pattern_ref", journey_pattern_ref)
    }

    fn get_by_column(
        &self,
        column: &str,
        value: &str,
    ) -> rusqlite::Result<Vec<TxVehicleJourney>> {
        // Column names are supplied only by the fixed methods above.
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let sql = format!(
            "SELECT * FROM tx_vehicle_journeys WHERE {column} = ? \
             ORDER BY departure_time, id"
        );

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params![value],
                Self::row_to_tx_vehicle_journey,
            )?
            .collect()
    }

    pub fn get_operating_today(&self) -> rusqlite::Result<Vec<TxVehicleJourney>> {
        let today = Local::now().date_naive();
        let today_string = today.format("%Y-%m-%d").to_string();

        let weekday_column = match today.weekday() {
            Weekday::Mon => "monday",
            Weekday::Tue => "tuesday",
            Weekday::Wed => "wednesday",
            Weekday::Thu => "thursday",
            Weekday::Fri => "friday",
            Weekday::Sat => "saturday",
            Weekday::Sun => "sunday",
        };

        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let sql = format!(
            r#"
        SELECT v.*
        FROM tx_vehicle_journeys v
        INNER JOIN tx_services s
            ON s.service_code = v.service_ref
        WHERE
            s.{weekday_column} = 1
            AND v.{weekday_column} = 1
            AND (
                s.start_date IS NULL
                OR date(s.start_date) <= date(?1)
            )
            AND (
                s.end_date IS NULL
                OR date(s.end_date) >= date(?1)
            )
        ORDER BY v.departure_time
        "#
        );

        let mut statement = connection.prepare(&sql)?;

        let rows = statement.query_map([today_string], |row| {
            Ok(TxVehicleJourney {
                id: row.get("id")?,
                operator_ref: row.get("operator_ref")?,
                service_ref: row.get("service_ref")?,
                line_ref: row.get("line_ref")?,
                journey_pattern_ref: row.get("journey_pattern_ref")?,
                departure_time: row.get("departure_time")?,
                garage_ref: row.get("garage_ref")?,
                block_description: row.get("block_description")?,
                block_number: row.get("block_number")?,
                ticket_machine_service_code: row
                    .get("ticket_machine_service_code")?,
                ticket_machine_journey_code: row
                    .get("ticket_machine_journey_code")?,
                monday: row.get("monday")?,
                tuesday: row.get("tuesday")?,
                wednesday: row.get("wednesday")?,
                thursday: row.get("thursday")?,
                friday: row.get("friday")?,
                saturday: row.get("saturday")?,
                sunday: row.get("sunday")?,
            })
        })?;

        rows.collect()
    }

    pub fn create(
        &self,
        journey: &TxVehicleJourney,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_vehicle_journeys (
                id,
                operator_ref,
                service_ref,
                line_ref,
                journey_pattern_ref,
                departure_time,
                garage_ref,
                block_description,
                block_number,
                ticket_machine_service_code,
                ticket_machine_journey_code,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday
            )
            VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?
            )
            "#,
            Self::params(journey),
        )?;

        Ok(())
    }

    pub fn upsert(
        &self,
        journey: &TxVehicleJourney,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_vehicle_journeys (
                id,
                operator_ref,
                service_ref,
                line_ref,
                journey_pattern_ref,
                departure_time,
                garage_ref,
                block_description,
                block_number,
                ticket_machine_service_code,
                ticket_machine_journey_code,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday
            )
            VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?
            )
            ON CONFLICT(id) DO UPDATE SET
                operator_ref = excluded.operator_ref,
                service_ref = excluded.service_ref,
                line_ref = excluded.line_ref,
                journey_pattern_ref = excluded.journey_pattern_ref,
                departure_time = excluded.departure_time,
                garage_ref = excluded.garage_ref,
                block_description = excluded.block_description,
                block_number = excluded.block_number,
                ticket_machine_service_code =
                    excluded.ticket_machine_service_code,
                ticket_machine_journey_code =
                    excluded.ticket_machine_journey_code,
                monday = excluded.monday,
                tuesday = excluded.tuesday,
                wednesday = excluded.wednesday,
                thursday = excluded.thursday,
                friday = excluded.friday,
                saturday = excluded.saturday,
                sunday = excluded.sunday
            "#,
            Self::params(journey),
        )?;

        Ok(())
    }

    pub fn upsert_many(
        &self,
        journeys: &[TxVehicleJourney],
    ) -> rusqlite::Result<()> {
        let mut connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let transaction = connection.transaction()?;

        {
            let mut statement = transaction.prepare(
                r#"
            INSERT INTO tx_vehicle_journeys (
                id,
                operator_ref,
                service_ref,
                line_ref,
                journey_pattern_ref,
                departure_time,
                garage_ref,
                block_description,
                block_number,
                ticket_machine_service_code,
                ticket_machine_journey_code,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
            )
            ON CONFLICT(id) DO UPDATE SET
                operator_ref = excluded.operator_ref,
                service_ref = excluded.service_ref,
                line_ref = excluded.line_ref,
                journey_pattern_ref = excluded.journey_pattern_ref,
                departure_time = excluded.departure_time,
                garage_ref = excluded.garage_ref,
                block_description = excluded.block_description,
                block_number = excluded.block_number,
                ticket_machine_service_code = excluded.ticket_machine_service_code,
                ticket_machine_journey_code = excluded.ticket_machine_journey_code,
                monday = excluded.monday,
                tuesday = excluded.tuesday,
                wednesday = excluded.wednesday,
                thursday = excluded.thursday,
                friday = excluded.friday,
                saturday = excluded.saturday,
                sunday = excluded.sunday
            "#,
            )?;

            for journey in journeys {
                statement.execute(rusqlite::params![
                journey.id,
                journey.operator_ref,
                journey.service_ref,
                journey.line_ref,
                journey.journey_pattern_ref,
                journey.departure_time,
                journey.garage_ref,
                journey.block_description,
                journey.block_number,
                journey.ticket_machine_service_code,
                journey.ticket_machine_journey_code,
                journey.monday,
                journey.tuesday,
                journey.wednesday,
                journey.thursday,
                journey.friday,
                journey.saturday,
                journey.sunday,
            ])?;
            }
        }

        transaction.commit()?;

        Ok(())
    }

    pub fn delete(
        &self,
        id: &str,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM tx_vehicle_journeys WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn params(
        journey: &TxVehicleJourney,
    ) -> [&dyn rusqlite::ToSql; 18] {
        [
            &journey.id,
            &journey.operator_ref,
            &journey.service_ref,
            &journey.line_ref,
            &journey.journey_pattern_ref,
            &journey.departure_time,
            &journey.garage_ref,
            &journey.block_description,
            &journey.block_number,
            &journey.ticket_machine_service_code,
            &journey.ticket_machine_journey_code,
            &journey.monday,
            &journey.tuesday,
            &journey.wednesday,
            &journey.thursday,
            &journey.friday,
            &journey.saturday,
            &journey.sunday,
        ]
    }

    fn row_to_tx_vehicle_journey(
        row: &rusqlite::Row<'_>,
    ) -> rusqlite::Result<TxVehicleJourney> {
        Ok(TxVehicleJourney {
            id: row.get("id")?,
            operator_ref: row.get("operator_ref")?,
            service_ref: row.get("service_ref")?,
            line_ref: row.get("line_ref")?,
            journey_pattern_ref: row.get("journey_pattern_ref")?,
            departure_time: row.get("departure_time")?,
            garage_ref: row.get("garage_ref")?,
            block_description: row.get("block_description")?,
            block_number: row.get("block_number")?,
            ticket_machine_service_code: row.get("ticket_machine_service_code")?,
            ticket_machine_journey_code: row.get("ticket_machine_journey_code")?,
            monday: row.get::<_, bool>("monday")?,
            tuesday: row.get::<_, bool>("tuesday")?,
            wednesday: row.get::<_, bool>("wednesday")?,
            thursday: row.get::<_, bool>("thursday")?,
            friday: row.get::<_, bool>("friday")?,
            saturday: row.get::<_, bool>("saturday")?,
            sunday: row.get::<_, bool>("sunday")?,
        })
    }
}
