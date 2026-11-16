use std::sync::Arc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::db::Database;
use crate::db::resources::bt_livery::BtLivery;
use crate::db::resources::bt_vehicle_type::BtVehicleType;
use crate::db::resources::tx_line::TxLine;
use crate::db::resources::tx_operator::TxOperator;
use crate::db::resources::user::User;
use crate::db::tx_lines::TxLineQuery;
use crate::db::users::{UserQuery, Users};

#[derive(Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct BigSearchResult {
    pub users: Vec<User>,
    pub operators: Vec<TxOperator>,
    pub routes: Vec<TxLine>,
    pub vehicle_types: Vec<BtVehicleType>,
    pub liveries: Vec<BtLivery>,
}

pub fn big_search(db: Arc<Database>, query: String) -> rusqlite::Result<BigSearchResult> {
    let users = db.users.search(&UserQuery {
        search: Some(query.clone()),
        page_length: None,
        offset: None,
    })?;

    let operators = db.tx_operators.search(query.as_str())?;

    let routes = db.tx_lines.search(&TxLineQuery {
        search: Some(query.clone()),
        tx_id: None,
        name: None,
        operator_code: None,
        operator_national_code: None,
        line_id: None,
        service_code: None,
        outbound_description: None,
        inbound_description: None,
        page_length: None,
        offset: None,
    })?;

    let vehicle_types = db.vehicle_types.search(query.as_str())?;

    let liveries = db.liveries.search(query.as_str())?;

    Ok(BigSearchResult {
        users,
        operators,
        routes,
        vehicle_types,
        liveries,
    })
}