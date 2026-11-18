use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, TimeDelta, Utc};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use crate::db::Database;
use crate::db::resource::bustimes::livery_def::BtLivery;
use crate::db::resource::bustimes::vehicle_def::BtVehicle;
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use crate::db::resource::last_fetch::last_fetch::LastFetch;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct BtVehicleResponseWithId {
    pub id: Option<i64>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct BtVehicleResponseWithIdString {
    pub id: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtVehicleResponse {
    pub id: i64,
    pub slug: String,
    pub fleet_code: String,
    pub reg: String,
    pub previous_reg: String,
    pub vehicle_type: Option<BtVehicleResponseWithId>,
    pub livery: Option<BtVehicleResponseWithId>,
    pub branding: String,
    pub operator: Option<BtVehicleResponseWithIdString>,
    // pub garage_id: Option<String>,
    pub name: String,
    pub notes: String,
    pub withdrawn: bool,
}

pub async fn fetch_bustimes_resources(db: Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let last_fetches = db.last_fetches.get().await?;

    if has_passed(last_fetches.bt_liveries, TimeDelta::days(1))? == true {
        let liveries = fetch_all_pages::<BtLivery>(
            &client,
            "https://bustimes.org/api/liveries?limit=10000",
        ).await?;

        for livery in liveries {
            db.liveries.upsert(&livery).await?;
        }

        db.last_fetches.set(LastFetch::BtLiveries).await?;
    }

    if has_passed(last_fetches.bt_vehicle_types, TimeDelta::days(1))? == true {
        let vehicle_types = fetch_all_pages::<BtVehicleType>(
            &client,
            "https://bustimes.org/api/vehicletypes?limit=10000",
        ).await?;

        for vt in vehicle_types {
            db.vehicle_types.upsert(&vt).await?;
        }

        db.last_fetches.set(LastFetch::BtVehicleTypes).await?;
    }

    if has_passed(last_fetches.bt_vehicles, TimeDelta::days(3))? == true {
        let vehicles = fetch_all_pages::<BtVehicleResponse>(
            &client,
            "https://bustimes.org/api/vehicles?limit=1000",
        ).await?;

        for v in vehicles {
            db.vehicles.upsert_bustimes(&BtVehicle {
                id: v.id,
                slug: v.slug,
                fleet: v.fleet_code,
                reg: v.reg,
                previous_reg: v.previous_reg,
                vehicle_type_id: v.vehicle_type.and_then(|x| x.id),
                livery_id: v.livery.and_then(|x| x.id),
                branding: v.branding,
                operator_id: v.operator
                    .and_then(|operator| operator.id)
                    .unwrap_or_default(),
                garage_id: None,
                name: v.name,
                notes: v.notes,
                withdrawn: v.withdrawn,
                training_vehicle: false,
                demonstrator: false,
                open_top: false,
                comment_thread: None
            }).await?;
        }

        db.last_fetches.set(LastFetch::BtVehicles).await?;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ApiPage<T> {
    count: i64,
    next: Option<String>,
    // _previous: Option<String>,
    results: Vec<T>,
}

pub async fn fetch_all_pages<T>(
    client: &Client,
    url: &str,
) -> Result<Vec<T>, reqwest::Error>
where
    T: DeserializeOwned,
{
    let mut url = Some(url.to_string());
    let mut results = Vec::new();
    let mut c = 0;

    while let Some(current_url) = url {
        println!("Fetching {}", current_url);
        let page: ApiPage<T> = client
            .get(&current_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;


        results.extend(page.results);
        url = page.next;

        if url.is_some() {
            sleep(Duration::from_millis(750)).await;
        }

        c += 1;
        println!("^ At: page {} of {}", c, page.count / 1000);

        //
        // if c > 3 {
        //     break;
        // }
    }

    Ok(results)
}

pub fn has_passed(
    timestamp: Option<String>,
    duration: TimeDelta,
) -> Result<bool, chrono::ParseError> {
    let Some(timestamp) = timestamp else {
        return Ok(true);
    };

    let timestamp = timestamp.parse::<DateTime<Utc>>()?;

    Ok(Utc::now() - timestamp >= duration)
}