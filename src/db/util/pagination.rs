use serde::Serialize;

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub current_page: i64,
    pub total_count: i64,
    pub items: Vec<T>,
}