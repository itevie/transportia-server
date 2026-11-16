use std::path::Path;
use std::sync::Arc;
use chrono::TimeDelta;
use crate::db::Database;
use crate::db::last_fetches::LastFetch;
use crate::fetchers::transxchange_parser::parse_transxchange;

pub mod mbc;
pub mod model_manufacturers;
pub mod bustimes;
mod transxchange;
mod transxchange_parser;
pub mod ttcdiecast;

pub fn run_all_in_background(db: Arc<Database>) {
    tokio::spawn(async move {
        run_all(db).await;
    });
}

pub async fn run_all(db: Arc<Database>) {
    let last_fetches = db.last_fetches.get().unwrap();
    
    
    if 1 == 2 {
        let db2 = Arc::clone(&db);

        match transxchange::for_each_xml(
            Path::new("/home/isabella/Documents/projects/rust/transportia/tx/bodds.zip"),
            |name, xml| {
                println!("Processing: {name}");
                let result = parse_transxchange(&xml);


                let ok = match result {
                    Ok(ok) => {
                        println!("Found {} ({}) running {} lines {} trips {} services", ok.0.short_name, ok.0.id, ok.1.len(), ok.2.len(), ok.3.len());
                        ok
                    },
                    Err(err) => {
                        eprintln!("{:#?}", err);
                        return Ok(());
                    }
                };
                //
                // match db2.tx_vehicle_journeys.upsert_many(&*ok.2) {
                //     Ok(_) => (),
                //     Err(err) => {
                //         eprintln!("{:#?}", err);
                //     }
                // }
                //
                // match db2.tx_services.upsert_many(&*ok.3) {
                //     Ok(_) => (),
                //     Err(err) => {
                //         eprintln!("{:#?}", err);
                //     }
                // }

                // for trip in ok.2 {
                //     match db2.tx_vehicle_journeys.upsert(&trip) {
                //         Ok(_) => (),
                //         Err(err) => {
                //             eprintln!("{:#?}", err);
                //             continue;
                //         }
                //     }
                // }



                match db2.tx_lines.upsert_many(&*ok.1) {
                    Ok(_) => (),
                    Err(err) => {
                        eprintln!("{:#?}", err);
                    }
                }

                match db2.tx_operators.upsert(&ok.0) {
                    Ok(_) => (),
                    Err(err) => {
                        eprintln!("{:#?}", err);
                        return Ok(());
                    }
                }

                Ok(())
            },
        ) {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("{:#?}", err);
                return;
            }
        }
    }

    match bustimes::fetch_bustimes_resources(db.clone()).await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{:#?}", err);
        }
    }

    if bustimes::has_passed(last_fetches.models_mbc, TimeDelta::days(1)).unwrap() == true {
        let mbc = match mbc::fetch().await {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

        for v in mbc {
            match db.bus_listings.upsert(&v) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }

        db.last_fetches.set(LastFetch::ModelsMbc).unwrap();
    }



    if bustimes::has_passed(last_fetches.models_ttcdiecast, TimeDelta::days(1)).unwrap() == true {
        println!("Scraping TTCDiecast");
        let ttcdiecast = match ttcdiecast::scrape_products("https://www.ttcdiecast.com/collections/northcord-model-buses").await {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

        for v in ttcdiecast {
            match db.bus_listings.upsert(&v) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }


        db.last_fetches.set(LastFetch::ModelsTtcdiecast).unwrap();
    }
}