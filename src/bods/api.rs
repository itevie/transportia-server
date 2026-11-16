
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

const BASE_URL: &str = "https://data.bus-data.dft.gov.uk/api/v1";

#[derive(Debug, Error)]
pub enum BodsError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid API key header")]
    InvalidApiKey,
}

#[derive(Clone)]
pub struct BodsClient {
    client: Client,
    base_url: String,
}

impl BodsClient {
    pub fn new(api_key: Option<&str>) -> Result<Self, BodsError> {
        let mut headers = HeaderMap::new();

        if let Some(key) = api_key {
            let value = HeaderValue::from_str(key)
                .map_err(|_| BodsError::InvalidApiKey)?;

            headers.insert("x-api-key", value);
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: BASE_URL.to_string(),
        })
    }

    /// Fetch an endpoint and deserialize the JSON into a typed value.
    pub async fn get<T>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T, BodsError>
    where
        T: DeserializeOwned,
    {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        Ok(self
            .client
            .get(url)
            .query(params)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?)
    }

    /// Fetch an endpoint when its JSON structure is not yet known.
    pub async fn get_json(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<Value, BodsError> {
        self.get(endpoint, params).await
    }
}