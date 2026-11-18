use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema, FromRow)]
pub struct BusListingResource {
    // Identity
    pub id: i64,
    pub external_id: Option<String>,
    pub origin: String,
    pub origin_code: String,
    pub fetched_from: String,

    // Listing
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,

    // Item Details
    pub manufacturer: Option<String>,
    pub scale: Option<String>,
    pub manufacturer_code: Option<String>,

    // Pricing
    pub price_low: f64,
    pub price_high: f64,
    pub currency: Option<String>,

    // Images
    pub image_list: String, // Separated by ;;;

    // Seller
    pub seller_name: Option<String>,
    pub seller_link: Option<String>,

    // Product
    pub condition: Option<String>,

    // Dates
    pub listed_at: Option<String>,
    pub ends_at: Option<String>,
    pub fetched_at: String,

    // Categorisation
    pub category: Option<String>, // Seperated by ;;;

    // Inventory
    pub total: Option<i64>,
    pub lowest: Option<i64>,
    pub sold_out: Option<bool>,
    pub some_sold_out: Option<bool>
}