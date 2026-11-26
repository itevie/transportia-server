use std::sync::Arc;
use sqlx::{PgPool};
use crate::db::Database;

#[derive(Clone)]
pub struct ActivityFeed {
    pool: PgPool,
    db: Arc<Database>
}

pub struct ActivityFeedQuery {
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl ActivityFeed {
    pub async fn search_feed(&self, query: &ActivityFeedQuery) {

    }
}