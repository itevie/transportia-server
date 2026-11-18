use std::sync::Arc;

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::db::Database;
use crate::db::resource::bustimes::livery_def::BtLivery;
use crate::db::resource::bustimes::vehicle_type_def::BtVehicleType;
use crate::db::resource::transxchange::line::TxLineQuery;
use crate::db::resource::transxchange::line_def::TxLine;
use crate::db::resource::transxchange::operator_def::TxOperator;
use crate::db::resource::user::UserQuery;
use crate::db::resource::user::def::User;

#[derive(Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct BigSearchResult {
    pub users: Vec<User>,
    pub operators: Vec<TxOperator>,
    pub routes: Vec<TxLine>,
    pub vehicle_types: Vec<BtVehicleType>,
    pub liveries: Vec<BtLivery>,
}

pub async fn big_search(db: Arc<Database>, query: String) -> Result<BigSearchResult, sqlx::Error> {
    let users = db
        .users
        .search(&UserQuery {
            search: Some(query.clone()),
            page_length: None,
            offset: None,
        })
        .await?;

    let operators = db.tx_operators.search(query.as_str()).await?;

    let routes = db
        .tx_lines
        .search(&TxLineQuery {
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
        })
        .await?;

    let vehicle_types = db.vehicle_types.search(query.as_str()).await?;

    let liveries = db.liveries.search(query.as_str()).await?;

    Ok(BigSearchResult {
        users,
        operators,
        routes,
        vehicle_types,
        liveries,
    })
}
