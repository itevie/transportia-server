use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::db::resources::tx_service::TxService;

pub static SQL_TX_SERVICE: &str = r#"
CREATE TABLE IF NOT EXISTS tx_services (
    id TEXT PRIMARY KEY,
    service_code TEXT,
    start_date TEXT,
    end_date TEXT,
    monday INTEGER NOT NULL DEFAULT 0,
    tuesday INTEGER NOT NULL DEFAULT 0,
    wednesday INTEGER NOT NULL DEFAULT 0,
    thursday INTEGER NOT NULL DEFAULT 0,
    friday INTEGER NOT NULL DEFAULT 0,
    saturday INTEGER NOT NULL DEFAULT 0,
    sunday INTEGER NOT NULL DEFAULT 0,
    bank_holidays_non_operation TEXT NOT NULL DEFAULT '[]',
    special_days_of_operation TEXT NOT NULL DEFAULT '[]',
    special_days_of_non_operation TEXT NOT NULL DEFAULT '[]',
    serviced_organisation_refs TEXT NOT NULL DEFAULT '[]',
    operator TEXT
)
"#;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TxServiceQuery {
    pub id: Option<String>,

    pub search: Option<String>,

    pub service_code: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,

    pub monday: Option<bool>,
    pub tuesday: Option<bool>,
    pub wednesday: Option<bool>,
    pub thursday: Option<bool>,
    pub friday: Option<bool>,
    pub saturday: Option<bool>,
    pub sunday: Option<bool>,

    pub bank_holidays_non_operation: Option<String>,
    pub special_days_of_operation: Option<String>,
    pub special_days_of_non_operation: Option<String>,
    pub serviced_organisation_refs: Option<String>,

    pub operator: Option<String>,

    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone)]
pub struct TxServices {
    pool: Pool<SqliteConnectionManager>,
}

impl TxServices {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(
        &self,
        service_code: &str,
    ) -> rusqlite::Result<Option<TxService>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            "SELECT * FROM tx_services WHERE service_code = ?",
        )?;

        let mut rows = statement.query(rusqlite::params![service_code])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tx_service(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(
        &self,
        query: &TxServiceQuery,
    ) -> rusqlite::Result<Vec<TxService>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query.page_length.unwrap_or(50).clamp(1, 1000);
        let offset = query.offset.unwrap_or(0).max(0);

        let mut sql = String::from(
            "SELECT * FROM tx_services WHERE 1 = 1",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        macro_rules! like_filter {
        ($field:ident) => {
            if let Some(value) = &query.$field {
                sql.push_str(concat!(
                    " AND ",
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
                    " AND ",
                    stringify!($field),
                    " = ?"
                ));
                params.push(Box::new(value.clone()));
            }
        };
    }

        like_filter!(id);
        like_filter!(service_code);
        like_filter!(operator);

        exact_filter!(start_date);
        exact_filter!(end_date);

        like_filter!(bank_holidays_non_operation);
        like_filter!(special_days_of_operation);
        like_filter!(special_days_of_non_operation);
        like_filter!(serviced_organisation_refs);

        macro_rules! bool_filter {
        ($field:ident) => {
            if let Some(value) = query.$field {
                sql.push_str(concat!(
                    " AND ",
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
                    id LIKE '%' || ? || '%' OR
                    service_code LIKE '%' || ? || '%' OR
                    operator LIKE '%' || ? || '%' OR
                    start_date LIKE '%' || ? || '%' OR
                    end_date LIKE '%' || ? || '%' OR
                    bank_holidays_non_operation LIKE '%' || ? || '%' OR
                    special_days_of_operation LIKE '%' || ? || '%' OR
                    special_days_of_non_operation LIKE '%' || ? || '%' OR
                    serviced_organisation_refs LIKE '%' || ? || '%'
                )
                "#,
                );

                for _ in 0..9 {
                    params.push(Box::new(word.to_string()));
                }
            }
        }

        sql.push_str(" ORDER BY service_code, start_date, end_date, id");
        sql.push_str(" LIMIT ?");
        params.push(Box::new(page_length));
        sql.push_str(" OFFSET ?");
        params.push(Box::new(offset));


        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_tx_service,
            )?
            .collect()
    }

    pub fn create(
        &self,
        service: &TxService,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_services (
                service_code,
                start_date,
                end_date,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,
                bank_holidays_non_operation,
                special_days_of_operation,
                special_days_of_non_operation,
                serviced_organisation_refs,
                id,
                operator
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
            )
            "#,
            Self::params(service),
        )?;

        Ok(())
    }

    pub fn upsert(
        &self,
        service: &TxService,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tx_services (
                service_code,
                start_date,
                end_date,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,
                bank_holidays_non_operation,
                special_days_of_operation,
                special_days_of_non_operation,
                serviced_organisation_refs,
                id,
                operator
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
            )
            ON CONFLICT(id) DO UPDATE SET
                operator = excluded.operator,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                monday = excluded.monday,
                tuesday = excluded.tuesday,
                wednesday = excluded.wednesday,
                thursday = excluded.thursday,
                friday = excluded.friday,
                saturday = excluded.saturday,
                sunday = excluded.sunday,
                bank_holidays_non_operation =
                    excluded.bank_holidays_non_operation,
                special_days_of_operation =
                    excluded.special_days_of_operation,
                special_days_of_non_operation =
                    excluded.special_days_of_non_operation,
                serviced_organisation_refs =
                    excluded.serviced_organisation_refs
            "#,
            Self::params(service),
        )?;

        Ok(())
    }

    pub fn upsert_many(
        &self,
        services: &[TxService],
    ) -> rusqlite::Result<()> {
        let mut connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let transaction = connection.transaction()?;

        {
            let mut statement = transaction.prepare(
                r#"
                INSERT INTO tx_services (
                    service_code,
                    start_date,
                    end_date,
                    monday,
                    tuesday,
                    wednesday,
                    thursday,
                    friday,
                    saturday,
                    sunday,
                    bank_holidays_non_operation,
                    special_days_of_operation,
                    special_days_of_non_operation,
                    serviced_organisation_refs,
                    id,
                    operator
                )
                VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                    ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
                )
                ON CONFLICT(id) DO UPDATE SET
                    operator = excluded.operator,
                    start_date = excluded.start_date,
                    end_date = excluded.end_date,
                    monday = excluded.monday,
                    tuesday = excluded.tuesday,
                    wednesday = excluded.wednesday,
                    thursday = excluded.thursday,
                    friday = excluded.friday,
                    saturday = excluded.saturday,
                    sunday = excluded.sunday,
                    bank_holidays_non_operation =
                        excluded.bank_holidays_non_operation,
                    special_days_of_operation =
                        excluded.special_days_of_operation,
                    special_days_of_non_operation =
                        excluded.special_days_of_non_operation,
                    serviced_organisation_refs =
                        excluded.serviced_organisation_refs
                "#,
            )?;

            for service in services {
                statement.execute(Self::params(service))?;
            }
        }

        transaction.commit()?;

        Ok(())
    }

    pub fn delete(
        &self,
        service_code: &str,
    ) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM tx_services WHERE service_code = ?",
            rusqlite::params![service_code],
        )?;

        Ok(())
    }

    fn params(
        service: &TxService,
    ) -> [&dyn rusqlite::ToSql; 16] {
        [
            &service.service_code,
            &service.start_date,
            &service.end_date,
            &service.monday,
            &service.tuesday,
            &service.wednesday,
            &service.thursday,
            &service.friday,
            &service.saturday,
            &service.sunday,
            &service.bank_holidays_non_operation,
            &service.special_days_of_operation,
            &service.special_days_of_non_operation,
            &service.serviced_organisation_refs,
            &service.id,
            &service.operator,
        ]
    }

    fn row_to_tx_service(
        row: &rusqlite::Row<'_>,
    ) -> rusqlite::Result<TxService> {
        Ok(TxService {
            id: row.get("id")?,
            service_code: row.get("service_code")?,
            start_date: row.get("start_date")?,
            end_date: row.get("end_date")?,
            monday: row.get::<_, bool>("monday")?,
            tuesday: row.get::<_, bool>("tuesday")?,
            wednesday: row.get::<_, bool>("wednesday")?,
            thursday: row.get::<_, bool>("thursday")?,
            friday: row.get::<_, bool>("friday")?,
            saturday: row.get::<_, bool>("saturday")?,
            sunday: row.get::<_, bool>("sunday")?,
            bank_holidays_non_operation: row.get(
                "bank_holidays_non_operation",
            )?,
            special_days_of_operation: row.get(
                "special_days_of_operation",
            )?,
            special_days_of_non_operation: row.get(
                "special_days_of_non_operation",
            )?,
            serviced_organisation_refs: row.get(
                "serviced_organisation_refs",
            )?,
            operator: row.get("operator")?,
        })
    }
}