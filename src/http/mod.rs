use crate::bods::datafeed::BodsSiriVmClient;
use crate::db::Database;
use crate::http::users::registration;
use axum::Router;
use meta::meta;
use std::sync::Arc;
use utoipa::OpenApi;
use crate::extractors::ratelimit::RateLimitState;

mod big_search;
mod bods;
pub mod bus_listings;
pub mod bustimes;
mod comments;
pub mod doc;
pub mod errors;
pub mod favourites;
mod favourites_groups;
pub mod meta;
mod model_buses;
mod photo_ratings;
pub mod photos;
pub mod route_checklists;
mod trams;
pub mod transxchange;
pub mod users;

#[derive(Clone)]
pub struct HttpState {
    pub db: Arc<Database>,
    pub s3: Arc<aws_sdk_s3::Client>,
    pub bods_client: BodsSiriVmClient,
    pub ratelimit: RateLimitState
}

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route("/meta", axum::routing::get(meta))
        .route(
            "/everything-search",
            axum::routing::get(big_search::big_search_route),
        )
        .merge(bus_listings::routes())
        .merge(users::routes())
        .merge(bustimes::routes())
        .merge(transxchange::routes())
        .merge(photos::routes())
        .merge(route_checklists::routes())
        .merge(favourites::routes())
        .merge(photo_ratings::routes())
        .merge(bods::routes())
        .merge(comments::routes())
        .merge(favourites_groups::routes())
        .merge(trams::routes())
        .merge(model_buses::routes())
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Transportia API"
    ),
    tags(
        (name = "Photos", description = "Photo management and retrieval"),
        (name = "Favourites", description = "User favourites"),
        (name = "Bustimes", description = "Resources fetched from bustimes.org"),
        (name = "Transxchange", description = "Resources loaded from transxchange"),
        (name = "Route Checklists", description = "Route checklist management"),
        (name = "Authentication", description = "Login / registration routes")
    ),
    paths(
        meta::meta,

        photos::query::query_photos,
        photos::query::get_photo,
        photos::manage::update_photo,
        photos::manage::delete_photo,
        photos::bus_details::update_details,
        photos::image::get_photo_image,
        photos::upload::upload_image,

        favourites::manage::create_favourite,
        favourites::manage::delete_favourite,
        favourites::manage::update_favourite_group,
        favourites::query::get_favourites,
        favourites::query::get_favourites_by_group,
        favourites::query::get_favourite,

        bustimes::liveries::search_liveries,
        bustimes::liveries::get_livery,
        bustimes::vehicle_types::search_vehicle_types,
        bustimes::vehicle_types::get_vehicle_type,
        bustimes::vehicles::search_vehicles,
        bustimes::vehicles::get_vehicle,
        bustimes::vehicles::update_vehicle,

        transxchange::lines::search_lines,
        transxchange::operators::search_operators,

        route_checklists::items::create_route_checklist_item,
        route_checklists::items::create_route_checklist_item_from_operators,
        route_checklists::items::complete_route_checklist_item,
        route_checklists::manage::delete_route_checklist,
        route_checklists::manage::create_route_checklist,
        route_checklists::query::get_route_checklists,
        route_checklists::query::get_route_checklist,

        favourites_groups::manage::create_favourite_group,
        favourites_groups::manage::update_favourite_group,
        favourites_groups::manage::delete_favourite_group,
        favourites_groups::query::fetch_favourite_groups,

        users::login::login,
        users::query::get_user,
        users::query::me,
        users::query::search_users,
        users::registration::register,
        users::manage::update_user,

        photo_ratings::manage::rate_photo,
        photo_ratings::manage::delete_photo_rating,
        photo_ratings::query::get_photo_rating,

        comments::query::get_comments,
        comments::manage::create_comment,
        comments::manage::update_comment,
        comments::manage::delete_comment,

        trams::vehicle::search_trams,
        trams::vehicle::get_tram,
        trams::vehicle::create_tram,
        trams::vehicle::update_tram,
        trams::vehicle::delete_tram,
        trams::tram_types::search_tram_types,
        trams::tram_types::get_tram_type,
        trams::tram_types::create_tram_type,
        trams::tram_types::update_tram_type,
        trams::tram_types::delete_tram_type,
        trams::tram_operators::search_tram_operators,
        trams::tram_operators::get_tram_operator,
        trams::tram_operators::create_tram_operator,
        trams::tram_operators::update_tram_operator,
        trams::tram_operators::delete_tram_operator,
        trams::tram_liveries::search_tram_liveries,
        trams::tram_liveries::get_tram_livery,
        trams::tram_liveries::create_tram_livery,
        trams::tram_liveries::update_tram_livery,
        trams::tram_liveries::delete_tram_livery,

        model_buses::resource::search_model_buses,
        model_buses::resource::get_model_bus,
        model_buses::resource::create_model_bus,
        model_buses::resource::update_model_bus,
        model_buses::resource::delete_model_bus,

        model_buses::model_bus_ownerships::search_model_bus_ownerships,
        model_buses::model_bus_ownerships::get_model_bus_ownership,
        model_buses::model_bus_ownerships::create_model_bus_ownership,
        model_buses::model_bus_ownerships::update_model_bus_ownership,
        model_buses::model_bus_ownerships::delete_model_bus_ownership,
    )
)]
pub struct ApiDoc;

pub fn a() {
    // FilledOutPhotoBus
}
