use crate::http::HttpState;
use axum::Router;

pub mod items;
pub mod manage;
pub mod query;

pub fn routes() -> Router<HttpState> {
    Router::new()
        .route(
            "/route-checklists",
            axum::routing::get(query::get_route_checklists),
        )
        .route(
            "/route-checklists/{id}",
            axum::routing::get(query::get_route_checklist),
        )
        .route(
            "/route-checklists/{id}/items",
            axum::routing::get(query::get_route_checklist_items),
        )
        .route(
            "/route-checklists/{id}",
            axum::routing::delete(manage::delete_route_checklist),
        )
        .route(
            "/route-checklists",
            axum::routing::post(manage::create_route_checklist),
        )
        .route(
            "/route-checklists/{id}/items/operators/{operator_id}",
            axum::routing::post(items::create_route_checklist_item_from_operators),
        )
        .route(
            "/route-checklists/{id}/items/operators-bulk",
            axum::routing::post(items::create_route_checklist_items_from_operators_bulk),
        )
        .route(
            "/route-checklists/items/operators-bulk/count",
            axum::routing::post(items::count_lines_from_operators_bulk),
        )
        .route(
            "/route-checklists/{id}/items/{line_id}",
            axum::routing::post(items::create_route_checklist_item),
        )
        .route(
            "/route-checklists/{id}/items/{item_id}/toggle-completed",
            axum::routing::post(items::complete_route_checklist_item),
        )
}
