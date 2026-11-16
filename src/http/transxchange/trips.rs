use axum::extract::{Query, State};
use axum::Json;
use crate::db;
use crate::db::tx_lines::TxLineQuery;
use crate::db::tx_vehicle_journeys::TxVehicleJourneyQuery;
use crate::http::HttpState;

#[utoipa::path(
    get,
    path = "/trips",
    tag = "Transxchange",
    summary = "Search Trips",
    params(TxLineQuery),
    responses(
        (
            status = 200,
            description = "Lines matching the search query",
            body = [TxVehicleJourneyQuery]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_trips(
    State(state): State<HttpState>,
    Query(query): Query<TxVehicleJourneyQuery>,
) -> Result<Json<Vec<db::resources::tx_vehicle_journey::TxVehicleJourney>>, String> {

    let listings = state
        .db
        .tx_vehicle_journeys
        .search(&query)
        .map_err(|err| err.to_string())?;

    Ok(Json(listings))
}

#[utoipa::path(
    get,
    path = "/trips/today",
    tag = "Transxchange",
    summary = "Get Today's Trips",
    responses(
        (
            status = 200,
            description = "Vehicle journeys operating today",
            body = [db::resources::tx_vehicle_journey::TxVehicleJourney]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_trips_today(
    State(state): State<HttpState>,
) -> Result<
    Json<Vec<db::resources::tx_vehicle_journey::TxVehicleJourney>>,
    String,
> {
    let journeys = state
        .db
        .tx_vehicle_journeys
        .get_operating_today()
        .map_err(|err| err.to_string())?;

    Ok(Json(journeys))
}