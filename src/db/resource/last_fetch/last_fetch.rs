
use sqlx::PgPool;
use crate::db::resource::last_fetch::last_fetch_def::LastFetches;

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
    ModelsTtcdiecast,
}

#[derive(Clone)]
pub struct LastFetchesDb {
    pool: PgPool,
}

impl LastFetchesDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> sqlx::Result<LastFetches> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO last_fetches (id)
            VALUES (1)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
            .execute(&mut *transaction)
            .await?;

        let last_fetches = sqlx::query_as::<_, LastFetches>(
            r#"
            SELECT *
            FROM last_fetches
            WHERE id = 1
            "#,
        )
            .fetch_one(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(last_fetches)
    }

    pub async fn set(&self, fetch: LastFetch) -> sqlx::Result<()> {
        let now = chrono::Utc::now().to_rfc3339_opts(
            chrono::SecondsFormat::Secs,
            true,
        );

        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO last_fetches (id)
            VALUES (1)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
            .execute(&mut *transaction)
            .await?;

        let query = match fetch {
            LastFetch::BtLiveries => {
                "UPDATE last_fetches SET bt_liveries = $1 WHERE id = 1"
            }
            LastFetch::BtOperators => {
                "UPDATE last_fetches SET bt_operators = $1 WHERE id = 1"
            }
            LastFetch::BtVehicleTypes => {
                "UPDATE last_fetches SET bt_vehicle_types = $1 WHERE id = 1"
            }
            LastFetch::BtVehicles => {
                "UPDATE last_fetches SET bt_vehicles = $1 WHERE id = 1"
            }
            LastFetch::ModelsMbc => {
                "UPDATE last_fetches SET models_mbc = $1 WHERE id = 1"
            }
            LastFetch::ModelsTtcdiecast => {
                "UPDATE last_fetches SET models_ttcdiecast = $1 WHERE id = 1"
            }
        };

        sqlx::query(query)
            .bind(now)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }
}
