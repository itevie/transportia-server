use std::time::Duration;
use reqwest::{Client, Url};
use serde::Deserialize;
use tokio::time::sleep;
use crate::db::resources::bus_listing::BusListingResource;

pub async fn fetch() -> Result<Vec<BusListingResource>, Box<dyn std::error::Error>> {
    let client = Client::new();

    let per_page = 30;
    let category = "YLQYN3GWSKLXG7KHJLBLOIQI".to_string();

    let mut listings = Vec::new();

    for page in 1.. {
        if page > 1 {
            sleep(Duration::from_secs(1)).await;
        }

        let url = make_mbc_url(page, per_page, category.clone());

        let http_response = client
            .get(&url)
            .send()
            .await?;

        let status = http_response.status();
        let text = http_response.text().await?;

        if !status.is_success() {
            println!("Failure! {}", status);
            break;
        }

        match serde_json::from_str::<ProductsResponse>(&text) {
            Ok(response) => {
                println!("Parsed successfully");
            }
            Err(e) => {
                println!("JSON error: {e}");
                println!("Around error:");

                let pos = e.column();
                println!("{}", &text[pos.saturating_sub(200)..pos + 200]);
            }
        }

        let response: ProductsResponse = serde_json::from_str(&text)?;

        println!("{} {} {}", response.meta.pagination.current_page, response.meta.pagination.total_pages, response.meta.pagination.total);

        for part in response.data {
            listings.push(convert_product(part, &url));
        }

        if page >= response.meta.pagination.total_pages {
            break;
        }
    }

    Ok(listings)
}

fn convert_product(
    part: Product,
    url: &str,
) -> BusListingResource {
    let title_parts = part.name.split(" / ").collect::<Vec<&str>>();

    let manufacturer_id = title_parts[0];

    let manufacturer = manufacturer_id
        .rsplit_once(' ')
        .map(|(before, _)| before)
        .unwrap_or(manufacturer_id);

    let scale = title_parts
        .iter()
        .find(|part| part.ends_with("Scale"))
        .map(|part| part.to_string());

    BusListingResource {
        id: 0,
        external_id: Some(part.id),
        origin: "https://www.modelbus.uk/".to_string(),
        origin_code: "mbc".to_string(),
        fetched_from: url.to_string(),
        title: part.name.clone(),
        description: part.short_description,
        link: Some(part.absolute_site_link),
        scale,
        manufacturer: Some(manufacturer.to_string()),
        manufacturer_code: Some(manufacturer_id.to_string()),
        price_low: part.price.low,
        price_high: part.price.high,
        currency: Some("£".to_string()),
        image_list:  part
            .images
            .data
            .first()
            .map(|image| image.absolute_url.clone())
            .unwrap_or_default(),
        seller_name: None,
        seller_link: None,
        condition: if manufacturer_id.contains("PRE-OWNED") {
            Some("PRE-OWNED".to_string())
        } else {
            None
        },
        listed_at: Some(part.created_date),
        ends_at: None,
        fetched_at: chrono::Utc::now().to_rfc3339(),
        category: Some(part.category_ids.join(";;;")),
        total: Some(part.inventory.total),
        lowest: part.inventory.lowest,
        sold_out: Some(part.inventory.all_variations_sold_out),
        some_sold_out: Some(part.inventory.some_variations_sold_out),
    }
}

fn make_mbc_url(page: u32, per_page: u32, category: String) -> String {
    let mut url = Url::parse(
        "https://cdn5.editmysite.com/app/store/api/v28/editor/users/142086096/sites/674346216273917466/products"
    ).unwrap();

    url.query_pairs_mut()
        .append_pair("page", &page.to_string())
        .append_pair("per_page", &per_page.to_string())
        .append_pair("sort_by", "category_order")
        .append_pair("sort_order", "asc")
        //.append_pair("categories[]", &category.to_string())
        .append_pair("include", "images,media_files,discounts")
        .append_pair("excluded_fulfillment", "dine_in")
        .append_pair("cache-version", "2026-03-25");

    url.to_string()
}




#[derive(Debug, Deserialize)]
struct ProductsResponse {
    data: Vec<Product>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Meta {
    pagination: Pagination,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    total: u32,
    count: u32,
    per_page: u32,
    current_page: u32,
    total_pages: u32,
    links: PaginationLinks,
}

#[derive(Debug, Deserialize)]
struct PaginationLinks {
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Product {
    id: String,
    name: String,
    site_product_id: String,

    short_description: Option<String>,
    product_type: String,
    site_link: String,
    absolute_site_link: String,

    price: Price,
    inventory: Inventory,

    thumbnail: ImageContainer,
    images: ImageList,

    created_date: String,

    #[serde(rename = "categoryIds")]
    category_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Price {
    low: f64,
    high: f64,

    low_formatted: String,
    high_formatted: String,

    low_subunits: i64,
    high_subunits: i64,
}

#[derive(Debug, Deserialize)]
struct Inventory {
    total: i64,
    lowest: Option<i64>,
    all_variations_sold_out: bool,
    some_variations_sold_out: bool,
}

#[derive(Debug, Deserialize)]
struct ImageContainer {
    data: Option<Image>,
}

#[derive(Debug, Deserialize)]
struct ImageList {
    data: Vec<Image>,
}

#[derive(Debug, Deserialize)]
struct Image {
    id: String,
    format: String,

    absolute_url: String,
    url: String,

    width: u32,
    height: u32,

    urls: ImageUrls,
    absolute_urls: ImageUrls,
}

#[derive(Debug, Deserialize)]
struct ImageUrls {
    #[serde(rename = "80")]
    size_80: String,

    #[serde(rename = "160")]
    size_160: String,

    #[serde(rename = "320")]
    size_320: String,

    #[serde(rename = "640")]
    size_640: String,

    #[serde(rename = "1280")]
    size_1280: String,

    #[serde(rename = "2560")]
    size_2560: String,
}