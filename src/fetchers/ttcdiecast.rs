use anyhow::{Context, Result};
use reqwest::Url;
use scraper::{Html, Selector};
use crate::db::resource::model_bus_listings::model_bus_listing_def::BusListingResource;
use crate::fetchers::mbc::get_details;

#[derive(Debug, Clone)]
pub struct TtcdiecastProduct {
    pub title: String,
    pub url: String,
    pub fetch_url: String,
    pub image_url: String,
    pub regular_price: Option<String>,
    pub original_price: Option<String>,
    pub sale_price: Option<String>,
}

fn text_of(element: scraper::ElementRef<'_>) -> String {
    element.text().collect::<String>().trim().to_string()
}

pub async fn scrape_products(page_url: &str) -> Result<Vec<BusListingResource>> {
    // Fetch the page.
    let response = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; ProductScraper/1.0)")
        .build()?
        .get(page_url)
        .send()
        .await?
        .error_for_status()?;

    let html = response.text().await?;
    let document = Html::parse_document(&html);

    // CSS selectors.
    let product_selector = Selector::parse("li.grid__item").unwrap();
    let title_selector = Selector::parse("h3.card__heading a").unwrap();
    let image_selector = Selector::parse("img").unwrap();

    let regular_selector =
        Selector::parse(".price__regular .price-item--regular").unwrap();

    let original_selector =
        Selector::parse(".price__sale s.price-item--regular").unwrap();

    let sale_selector =
        Selector::parse(".price__sale .price-item--sale").unwrap();

    let base_url = Url::parse(page_url)?;
    let mut products = Vec::new();

    for item in document.select(&product_selector) {
        // There are two matching title links in your HTML.
        // Use the last one to get the main product card's link.
        let title_link = item.select(&title_selector).last();

        let Some(title_link) = title_link else {
            continue;
        };

        let title = text_of(title_link);

        let href = title_link
            .value()
            .attr("href")
            .unwrap_or_default();

        let url = base_url.join(href)?.to_string();

        // Find the product image.
        let image_url = item
            .select(&image_selector)
            .find_map(|img| img.value().attr("src"))
            .map(|src| base_url.join(src).map(|url| url.to_string()))
            .transpose()?
            .unwrap_or_default();

        // Find the prices within their respective containers.
        let regular_price = item
            .select(&regular_selector)
            .next()
            .map(text_of);

        let original_price = item
            .select(&original_selector)
            .next()
            .map(text_of);

        let sale_price = item
            .select(&sale_selector)
            .next()
            .map(text_of);

        products.push(TtcdiecastProduct {
            title,
            url,
            fetch_url: page_url.to_string(),
            image_url,
            regular_price,
            original_price,
            sale_price,
        });
    }

    Ok(products.iter().map(|x| convert(x.clone())).collect())
}

pub fn convert(product: TtcdiecastProduct) -> BusListingResource {
    let sale_price = parse_price(&product.sale_price
        .unwrap_or("0".to_string())).unwrap_or_default();

    let regular_price = parse_price(&product.regular_price
        .unwrap_or("0".to_string())).unwrap_or_default();

    let details = get_details(product.title.clone());

    BusListingResource {
        id: 0,
        external_id: Some(format!("Unknown_{}", product.title.clone())),
        origin: "https://www.ttcdiecast.com/".to_string(),
        origin_code: "ttcdiecast".to_string(),
        fetched_from: product.fetch_url.to_string(),
        title: product.title,
        description: None,
        link: product.url.into(),
        manufacturer: Some(details.manufacturer),
        scale: details.scale,
        manufacturer_code: Some(details.manufacturer_id),
        price_low: sale_price,
        price_high: regular_price,
        currency: Some("£".to_string()),
        image_list: product.image_url,
        seller_name: None,
        seller_link: None,
        condition: None,
        listed_at: None,
        ends_at: None,
        fetched_at: chrono::Utc::now().to_rfc3339(),
        category: None,
        total: None,
        lowest: None,
        sold_out: None,
        some_sold_out: None,
    }
}

fn parse_price(price: &str) -> Option<f64> {
    price
        .trim()
        .trim_start_matches('£')
        .trim_end_matches(" GBP")
        .parse()
        .ok()
}