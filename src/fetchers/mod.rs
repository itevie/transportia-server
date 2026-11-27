use crate::db::Database;
use crate::db::resource::last_fetch::last_fetch::LastFetch;
use crate::fetchers::transxchange_parser::parse_transxchange;
use chrono::TimeDelta;
use std::path::Path;
use std::sync::Arc;

pub mod bustimes;
pub mod mbc;
pub mod model_manufacturers;
mod transxchange;
mod transxchange_parser;
pub mod ttcdiecast;

pub fn run_all_in_background(db: Arc<Database>) {
    tokio::spawn(async move {
        run_all(db).await;
    });
}

pub async fn run_all(db: Arc<Database>) {
    let last_fetches = db.last_fetches.get().await.unwrap();

    if Path::new("./tx/should_update").exists() {
        let db2 = Arc::clone(&db);

        let mut parsed_results = Vec::new();

        match transxchange::for_each_xml(
            Path::new("./tx/bodds.zip"),
            |name, xml| {
                println!("Processing: {name}");

                match parse_transxchange(&xml) {
                    Ok(ok) => {
                        println!(
                            "Found {} ({}) running {} lines {} trips {} services",
                            ok.0.short_name,
                            ok.0.id,
                            ok.1.len(),
                            ok.2.len(),
                            ok.3.len()
                        );

                        parsed_results.push(ok);
                    }
                    Err(err) => {
                        eprintln!("{:#?}", err);
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

        for (operator, lines, journeys, services) in parsed_results {
            match db2.tx_operators.upsert(&operator).await {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{:#?}", err);
                    continue;
                }
            }

            match db2.tx_lines.upsert_many(&lines).await {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{:#?}", err);
                }
            }
        }

        let _ = std::fs::remove_file("./tx/should_update");
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
            match db.bus_listings.upsert(&v).await {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }

        db.last_fetches.set(LastFetch::ModelsMbc).await.unwrap();
    }

    if bustimes::has_passed(last_fetches.models_ttcdiecast, TimeDelta::days(1)).unwrap() == true {
        println!("Scraping TTCDiecast");
        let ttcdiecast = match ttcdiecast::scrape_products(
            "https://www.ttcdiecast.com/collections/northcord-model-buses",
        )
        .await
        {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

        for v in ttcdiecast {
            match db.bus_listings.upsert(&v).await {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }

        db.last_fetches
            .set(LastFetch::ModelsTtcdiecast)
            .await
            .unwrap();
    }
}
