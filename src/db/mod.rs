pub mod bus_listings;
pub mod resources;
pub mod users;
pub mod liveries;
pub mod last_fetches;
pub mod vehicle_types;
pub mod vehicles;
pub mod tx_operators;
pub mod tx_lines;
pub mod photo;
pub mod bus_photo_details;
pub mod route_checklists;
pub mod route_checklist_items;
mod favourites;
mod photo_ratings;
pub mod tx_vehicle_journeys;
pub mod tx_services;
pub mod comment_threads;
pub mod comments;
pub mod big_search;
pub mod favourites_groups;
mod tram_operators;
mod tram_types;
mod tram_liveries;
pub mod trams;
pub mod model_buses;
mod bustimes;
pub mod model_bus_ownerships;
pub mod tram_photo_details;
pub mod tram_stops;
mod tram_routes;
pub mod tram_announcement_audios;
mod model_bus_photo_details;
mod util;

use std::fs;
use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::types::Value;
use crate::db::bus_listings::{BusListings, SQL_BUS_LISTING};
use crate::db::bus_photo_details::{BusPhotoDetailsManager, SQL_BUS_PHOTO_DETAILS};
use crate::db::comment_threads::{CommentThreads, SQL_COMMENT_THREAD};
use crate::db::comments::{Comments, SQL_COMMENT};
use crate::db::favourites::{Favourites, SQL_FAVOURITE};
use crate::db::favourites_groups::{FavouriteGroups, SQL_FAVOURITES_GROUPS};
use crate::db::last_fetches::{LastFetchesDb, SQL_LAST_FETCHES};
use crate::db::liveries::{BtLiveries, SQL_BT_LIVERY};
use crate::db::model_bus_ownerships::{ModelBusOwnerships, SQL_MODEL_BUS_OWNERSHIPS};
use crate::db::model_bus_photo_details::{ModelBusPhotoDetailsManager, SQL_MODEL_BUS_PHOTO_DETAILS};
use crate::db::model_buses::{ModelBuses, SQL_MODEL_BUSES};
use crate::db::photo::{Photos, SQL_PHOTOS};
use crate::db::photo_ratings::{PhotoRatings, SQL_PHOTO_RATING};
use crate::db::resources::tram_photo_details::TramPhotoDetails;
use crate::db::resources::tram_type::TramType;
use crate::db::route_checklist_items::{RouteChecklistItems, SQL_ROUTE_CHECKLIST_ITEMS};
use crate::db::route_checklists::{RouteChecklists, SQL_ROUTE_CHECKLISTS};
use crate::db::tram_announcement_audios::{TramAnnouncementAudios, SQL_TRAM_ANNOUNCEMENT_AUDIO};
use crate::db::tram_liveries::{TramLiveries, SQL_TRAM_LIVERY};
use crate::db::tram_operators::{TramOperators, SQL_TRAM_OPERATOR};
use crate::db::tram_photo_details::{TramPhotoDetailsManager, SQL_TRAM_PHOTO_DETAILS};
use crate::db::tram_types::{TramTypes, SQL_TRAM_TYPE};
use crate::db::trams::{Trams, SQL_TRAM};
use crate::db::tx_lines::{TxLines, SQL_TX_LINE};
use crate::db::tx_operators::{TxOperators, SQL_TX_OPERATOR};
use crate::db::tx_services::{TxServices, SQL_TX_SERVICE};
use crate::db::tx_vehicle_journeys::{TxVehicleJourneys, SQL_TX_VEHICLE_JOURNEY};
use crate::db::users::{Users, SQL_USER};
use crate::db::vehicle_types::{BtVehicleTypes, SQL_BT_VEHICLE_TYPE};
use crate::db::vehicles::{BtVehicles, SQL_BT_VEHICLE};

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<SqliteConnectionManager>,
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
    pub route_checklists: RouteChecklists,
    pub route_checklist_items: RouteChecklistItems,
    pub favourites: Favourites,
    pub photo_ratings: PhotoRatings,
    pub tx_vehicle_journeys: TxVehicleJourneys,
    pub tx_services: TxServices,
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
    pub model_bus_photo_details: ModelBusPhotoDetailsManager
}

macro_rules! execute_sql {
    ($connection:expr, $sql:ident) => {
        $connection.execute($sql, [])
            .map_err(|e| format!("Error with {}: {}", stringify!($sql), e))?;
    };
}

pub fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    fs::create_dir_all("./data")
        .expect("Failed to create data directory");

    let manager = SqliteConnectionManager::file("./data/transportia.db");

    let pool = Pool::new(manager)?;

    {
        let connection = pool.get()?;

        execute_sql!(connection, SQL_BUS_LISTING);
        execute_sql!(connection, SQL_USER);
        execute_sql!(connection, SQL_LAST_FETCHES);
        execute_sql!(connection, SQL_BT_LIVERY);
        execute_sql!(connection, SQL_BT_VEHICLE_TYPE);
        execute_sql!(connection, SQL_BT_VEHICLE);
        execute_sql!(connection, SQL_TX_LINE);
        execute_sql!(connection, SQL_TX_OPERATOR);
        execute_sql!(connection, SQL_PHOTOS);
        execute_sql!(connection, SQL_BUS_PHOTO_DETAILS);
        execute_sql!(connection, SQL_ROUTE_CHECKLISTS);
        execute_sql!(connection, SQL_ROUTE_CHECKLIST_ITEMS);
        execute_sql!(connection, SQL_FAVOURITE);
        execute_sql!(connection, SQL_PHOTO_RATING);
        execute_sql!(connection, SQL_TX_VEHICLE_JOURNEY);
        execute_sql!(connection, SQL_TX_SERVICE);
        execute_sql!(connection, SQL_COMMENT_THREAD);
        execute_sql!(connection, SQL_COMMENT);
        execute_sql!(connection, SQL_FAVOURITES_GROUPS);
        execute_sql!(connection, SQL_TRAM);
        execute_sql!(connection, SQL_TRAM_LIVERY);
        execute_sql!(connection, SQL_TRAM_TYPE);
        execute_sql!(connection, SQL_TRAM_OPERATOR);
        execute_sql!(connection, SQL_MODEL_BUSES);
        execute_sql!(connection, SQL_MODEL_BUS_OWNERSHIPS);
        execute_sql!(connection, SQL_TRAM_PHOTO_DETAILS);
        execute_sql!(connection, SQL_TRAM_ANNOUNCEMENT_AUDIO);
        execute_sql!(connection, SQL_MODEL_BUS_PHOTO_DETAILS);
    }

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
        route_checklists: RouteChecklists::new(pool.clone()),
        route_checklist_items: RouteChecklistItems::new(pool.clone()),
        favourites: Favourites::new(pool.clone()),
        photo_ratings: PhotoRatings::new(pool.clone()),
        tx_vehicle_journeys: TxVehicleJourneys::new(pool.clone()),
        tx_services: TxServices::new(pool.clone()),
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

pub fn create_missing_comment_threads(
    db: Arc<Database>,
    table_name: &str,
) -> rusqlite::Result<usize> {
    const ALLOWED_TABLES: &[&str] = &[
        "tx_operators",
        "photos",
        "tx_routes",
        "bt_vehicles",
    ];

    if !ALLOWED_TABLES.contains(&table_name) {
        return Err(rusqlite::Error::InvalidParameterName(
            table_name.to_string(),
        ));
    }

    let mut connection = db.pool.get().map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(err))
    })?;

    let tx = connection.transaction()?;

    let sql = format!(
        r#"SELECT id FROM "{table_name}" WHERE comment_thread IS NULL"#
    );

    let ids: Vec<Value> = {
        let mut statement = tx.prepare(&sql)?;
        let rows = statement.query_map([], |row| row.get(0))?;
        rows.collect::<rusqlite::Result<Vec<Value>>>()?
    };

    let mut created = 0;

    for id in ids {
        let now = chrono::Utc::now().to_rfc3339();

        tx.execute(
            "INSERT INTO comment_threads (created_at) VALUES (?1)",
            [&now],
        )?;

        let thread_id = tx.last_insert_rowid();

        let sql = format!(
            r#"UPDATE "{table_name}"
               SET comment_thread = ?1
               WHERE id = ?2 AND comment_thread IS NULL"#
        );

        created += tx.execute(
            &sql,
            rusqlite::params![thread_id, id],
        )?;
    }

    tx.commit()?;

    Ok(created)
}