pub mod big_search;
mod bustimes;
pub mod macros;
pub mod resource;
pub mod util;
mod macro_pro_max;

use std::sync::Arc;

use crate::db::resource::bustimes::livery::{BtLiveries, SQL_BT_LIVERY};
use crate::db::resource::bustimes::vehicle::{BtVehicles, SQL_BT_VEHICLE};
use crate::db::resource::bustimes::vehicle_type::{BtVehicleTypes, SQL_BT_VEHICLE_TYPE};
use crate::db::resource::checklist::checklist::{Checklists, SQL_CHECKLISTS};
use crate::db::resource::checklist::checklist_item::{ChecklistItems, SQL_CHECKLIST_ITEMS};
use crate::db::resource::comment::comment::{Comments, SQL_COMMENT};
use crate::db::resource::comment::comment_thread::{CommentThreads, SQL_COMMENT_THREAD};
use crate::db::resource::favourite::favourite::{Favourites, SQL_FAVOURITE};
use crate::db::resource::favourite::favourite_group::{FavouriteGroups, SQL_FAVOURITES_GROUPS};
use crate::db::resource::last_fetch::last_fetch::{LastFetchesDb, SQL_LAST_FETCHES};
use crate::db::resource::model_bus::model_bus::{ModelBuses, SQL_MODEL_BUSES};
use crate::db::resource::model_bus::model_bus_ownership::{
    ModelBusOwnerships, SQL_MODEL_BUS_OWNERSHIPS,
};
use crate::db::resource::model_bus_listings::model_bus_listing::{BusListings, SQL_BUS_LISTING};
use crate::db::resource::photo::bus_photo_details::{
    BusPhotoDetailsManager, SQL_BUS_PHOTO_DETAILS,
};
use crate::db::resource::photo::model_bus_photo_details::{
    ModelBusPhotoDetailsManager, SQL_MODEL_BUS_PHOTO_DETAILS,
};
use crate::db::resource::photo::photo::{Photos, SQL_PHOTOS};
use crate::db::resource::photo::tram_photo_details::{
    TramPhotoDetailsManager, SQL_TRAM_PHOTO_DETAILS,
};
use crate::db::resource::star_rating::star_rating::{PhotoRatings, SQL_PHOTO_RATING};
use crate::db::resource::tram::tram::{Trams, SQL_TRAM};
use crate::db::resource::tram::tram_announcement_audio::{
    TramAnnouncementAudios, SQL_TRAM_ANNOUNCEMENT_AUDIO,
};
use crate::db::resource::tram::tram_livery::{TramLiveries, SQL_TRAM_LIVERY};
use crate::db::resource::tram::tram_operator::{TramOperators, SQL_TRAM_OPERATOR};
use crate::db::resource::tram::tram_route::SQL_TRAM_ROUTE;
use crate::db::resource::tram::tram_stop::SQL_TRAM_STOP;
use crate::db::resource::tram::tram_type::{TramTypes, SQL_TRAM_TYPE};
use crate::db::resource::transxchange::line::{TxLines, SQL_TX_LINE};
use crate::db::resource::transxchange::operator::{TxOperators, SQL_TX_OPERATOR};
use crate::db::resource::user::{Users, SQL_USER};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
    pub bus_listings: BusListings,
    pub users: Users,
    pub last_fetches: LastFetchesDb,
    pub liveries: BtLiveries,
    pub vehicle_types: BtVehicleTypes,
    pub vehicles: BtVehicles,
    pub tx_operators: TxOperators,
    pub tx_lines: TxLines,
    pub photos: Photos,
    pub bus_photo_details: BusPhotoDetailsManager,
    pub checklists: Checklists,
    pub checklist_items: ChecklistItems,
    pub favourites: Favourites,
    pub photo_ratings: PhotoRatings,
    pub comment_threads: CommentThreads,
    pub comments: Comments,
    pub favourites_groups: FavouriteGroups,
    pub trams: Trams,
    pub tram_liveries: TramLiveries,
    pub tram_types: TramTypes,
    pub tram_operators: TramOperators,
    pub model_buses: ModelBuses,
    pub model_bus_ownerships: ModelBusOwnerships,
    pub tram_photo_details: TramPhotoDetailsManager,
    pub tram_announcement_audios: TramAnnouncementAudios,
    pub model_bus_photo_details: ModelBusPhotoDetailsManager,
}

pub async fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    macro_rules! execute_sql {
        ($sql:ident) => {
            sqlx::query($sql)
                .execute(&pool)
                .await
                .map_err(|e| format!("Error with {}: {}", stringify!($sql), e))?;
        };
    }

    // Create tables in dependency order.
    execute_sql!(SQL_COMMENT_THREAD);

    execute_sql!(SQL_USER);
    execute_sql!(SQL_BUS_LISTING);
    execute_sql!(SQL_LAST_FETCHES);

    execute_sql!(SQL_BT_LIVERY);
    execute_sql!(SQL_BT_VEHICLE_TYPE);
    execute_sql!(SQL_BT_VEHICLE);

    execute_sql!(SQL_TX_OPERATOR);
    execute_sql!(SQL_TX_LINE);

    execute_sql!(SQL_PHOTOS);
    execute_sql!(SQL_BUS_PHOTO_DETAILS);

    execute_sql!(SQL_CHECKLISTS);
    execute_sql!(SQL_CHECKLIST_ITEMS);

    execute_sql!(SQL_FAVOURITES_GROUPS);
    execute_sql!(SQL_FAVOURITE);
    execute_sql!(SQL_PHOTO_RATING);

    execute_sql!(SQL_COMMENT);

    execute_sql!(SQL_TRAM_LIVERY);
    execute_sql!(SQL_TRAM_TYPE);
    execute_sql!(SQL_TRAM_OPERATOR);
    execute_sql!(SQL_TRAM);
    execute_sql!(SQL_TRAM_PHOTO_DETAILS);
    execute_sql!(SQL_TRAM_STOP);
    execute_sql!(SQL_TRAM_ROUTE);
    execute_sql!(SQL_TRAM_ANNOUNCEMENT_AUDIO);

    execute_sql!(SQL_MODEL_BUSES);
    execute_sql!(SQL_MODEL_BUS_OWNERSHIPS);
    execute_sql!(SQL_MODEL_BUS_PHOTO_DETAILS);

    let db = Database {
        pool: pool.clone(),
        bus_listings: BusListings::new(pool.clone()),
        users: Users::new(pool.clone()),
        last_fetches: LastFetchesDb::new(pool.clone()),
        liveries: BtLiveries::new(pool.clone()),
        vehicle_types: BtVehicleTypes::new(pool.clone()),
        vehicles: BtVehicles::new(pool.clone()),
        tx_operators: TxOperators::new(pool.clone()),
        tx_lines: TxLines::new(pool.clone()),
        photos: Photos::new(pool.clone()),
        bus_photo_details: BusPhotoDetailsManager::new(pool.clone()),
        checklists: Checklists::new(pool.clone()),
        checklist_items: ChecklistItems::new(pool.clone()),
        favourites: Favourites::new(pool.clone()),
        photo_ratings: PhotoRatings::new(pool.clone()),
        comment_threads: CommentThreads::new(pool.clone()),
        comments: Comments::new(pool.clone()),
        favourites_groups: FavouriteGroups::new(pool.clone()),
        trams: Trams::new(pool.clone()),
        tram_liveries: TramLiveries::new(pool.clone()),
        tram_types: TramTypes::new(pool.clone()),
        tram_operators: TramOperators::new(pool.clone()),
        model_buses: ModelBuses::new(pool.clone()),
        model_bus_ownerships: ModelBusOwnerships::new(pool.clone()),
        tram_photo_details: TramPhotoDetailsManager::new(pool.clone()),
        tram_announcement_audios: TramAnnouncementAudios::new(pool.clone()),
        model_bus_photo_details: ModelBusPhotoDetailsManager::new(pool.clone()),
    };

    Ok(db)
}

pub async fn create_missing_comment_threads(
    db: Arc<Database>,
    table_name: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = db.pool.begin().await?;

    println!("Ensuring comment threads for {}", table_name);

    let query = match table_name {
        "tx_operators" => {
            r#"
            WITH missing AS (
                SELECT
                    id,
                    nextval(pg_get_serial_sequence('comment_threads', 'id')) AS thread_id
                FROM tx_operators
                WHERE comment_thread IS NULL
            ),
            inserted AS (
                INSERT INTO comment_threads (id, created_at)
                SELECT thread_id, NOW()::TEXT
                FROM missing
                RETURNING id
            )
            UPDATE tx_operators AS t
            SET comment_thread = m.thread_id
            FROM missing AS m
            JOIN inserted AS i ON i.id = m.thread_id
            WHERE t.id = m.id
              AND t.comment_thread IS NULL
            "#
        }
        "photos" => {
            r#"
            WITH missing AS (
                SELECT
                    id,
                    nextval(pg_get_serial_sequence('comment_threads', 'id')) AS thread_id
                FROM photos
                WHERE comment_thread IS NULL
            ),
            inserted AS (
                INSERT INTO comment_threads (id, created_at)
                SELECT thread_id, NOW()::TEXT
                FROM missing
                RETURNING id
            )
            UPDATE photos AS t
            SET comment_thread = m.thread_id
            FROM missing AS m
            JOIN inserted AS i ON i.id = m.thread_id
            WHERE t.id = m.id
              AND t.comment_thread IS NULL
            "#
        }
        "tx_routes" => {
            r#"
            WITH missing AS (
                SELECT
                    id,
                    nextval(pg_get_serial_sequence('comment_threads', 'id')) AS thread_id
                FROM tx_routes
                WHERE comment_thread IS NULL
            ),
            inserted AS (
                INSERT INTO comment_threads (id, created_at)
                SELECT thread_id, NOW()::TEXT
                FROM missing
                RETURNING id
            )
            UPDATE tx_routes AS t
            SET comment_thread = m.thread_id
            FROM missing AS m
            JOIN inserted AS i ON i.id = m.thread_id
            WHERE t.id = m.id
              AND t.comment_thread IS NULL
            "#
        }
        "bt_vehicles" => {
            r#"
            WITH missing AS (
                SELECT
                    id,
                    nextval(pg_get_serial_sequence('comment_threads', 'id')) AS thread_id
                FROM bt_vehicles
                WHERE comment_thread IS NULL
            ),
            inserted AS (
                INSERT INTO comment_threads (id, created_at)
                SELECT thread_id, NOW()::TEXT
                FROM missing
                RETURNING id
            )
            UPDATE bt_vehicles AS t
            SET comment_thread = m.thread_id
            FROM missing AS m
            JOIN inserted AS i ON i.id = m.thread_id
            WHERE t.id = m.id
              AND t.comment_thread IS NULL
            "#
        }
        _ => {
            return Err(sqlx::Error::Protocol(format!(
                "Invalid table name: {table_name}"
            )));
        }
    };

    let result = sqlx::query(query).execute(&mut *tx).await?;

    println!(
        "Created {} comment threads for {}",
        result.rows_affected(),
        table_name
    );

    tx.commit().await?;

    Ok(())
}
