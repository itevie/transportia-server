pub mod bus_listing;
pub mod resources;

use std::fs;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::db::bus_listing::{BusListings, SQL_BUS_LISTING};

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<SqliteConnectionManager>,
    pub bus_listings: BusListings,
}

pub fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    fs::create_dir_all("./data")
        .expect("Failed to create data directory");

    let manager = SqliteConnectionManager::file("./data/transportia.db");

    let pool = Pool::new(manager)?;

    {
        let connection = pool.get()?;

        connection.execute(SQL_BUS_LISTING, [])?;
    }

    let db = Database {
        pool: pool.clone(),
        bus_listings: BusListings::new(pool.clone()),
    };

    Ok(db)
}