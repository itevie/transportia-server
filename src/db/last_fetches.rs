use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::resources::last_fetches::LastFetches;

pub static SQL_LAST_FETCHES: &str = r#"
CREATE TABLE IF NOT EXISTS last_fetches (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    bt_liveries TEXT,
    bt_operators TEXT,
    bt_vehicle_types TEXT,
    bt_vehicles TEXT,
    models_mbc TEXT,
    models_ttcdiecast TEXT
)
"#;

#[derive(Debug, Clone, Copy)]
pub enum LastFetch {
    BtLiveries,
    BtOperators,
    BtVehicleTypes,
    BtVehicles,
    ModelsMbc,
    ModelsTtcdiecast
}

#[derive(Clone)]
pub struct LastFetchesDb {
    pool: Pool<SqliteConnectionManager>,
}

impl LastFetchesDb {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self) -> rusqlite::Result<LastFetches> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT OR IGNORE INTO last_fetches (id)
            VALUES (1)
            "#,
            [],
        )?;

        connection.query_row(
            r#"
            SELECT
                bt_liveries,
                bt_operators,
                bt_vehicle_types,
                bt_vehicles,
                models_mbc,
                models_ttcdiecast
            FROM last_fetches
            WHERE id = 1
            "#,
            [],
            |row| {
                Ok(LastFetches {
                    bt_liveries: row.get("bt_liveries")?,
                    bt_operators: row.get("bt_operators")?,
                    bt_vehicles: row.get("bt_vehicles")?,
                    bt_vehicle_types: row.get("bt_vehicle_types")?,
                    models_mbc: row.get("models_mbc")?,
                    models_ttcdiecast: row.get("models_ttcdiecast")?,
                })
            },
        )
    }

    pub fn set(&self, fetch: LastFetch) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let now = chrono::Utc::now().to_rfc3339_opts(
            chrono::SecondsFormat::Secs,
            true,
        );

        connection.execute(
            r#"
            INSERT OR IGNORE INTO last_fetches (id)
            VALUES (1)
            "#,
            [],
        )?;

        let column = match fetch {
            LastFetch::BtLiveries => "bt_liveries",
            LastFetch::BtOperators => "bt_operators",
            LastFetch::BtVehicleTypes => "bt_vehicle_types",
            LastFetch::BtVehicles => "bt_vehicles",
            LastFetch::ModelsMbc => "models_mbc",
            LastFetch::ModelsTtcdiecast => "models_ttcdiecast",
        };

        connection.execute(
            &format!(
                "UPDATE last_fetches SET {} = ? WHERE id = 1",
                column
            ),
            rusqlite::params![now],
        )?;

        Ok(())
    }
}