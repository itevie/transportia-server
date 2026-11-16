
use reqwest::Client;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use utoipa::{IntoParams, ToSchema};
use crate::bods::siri_vm::{ServiceDelivery, SiriResponse};

const DATAFEED_URL: &str =
    "https://data.bus-data.dft.gov.uk/api/v1/datafeed";

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct DatafeedQuery {
    /// [minLongitude, minLatitude, maxLongitude, maxLatitude]
    pub bounding_box: Option<[f64; 4]>,
    pub operator_ref: Option<String>,
    pub vehicle_ref: Option<String>,
    pub line_ref: Option<String>,
    pub producer_ref: Option<String>,
    pub origin_ref: Option<String>,
    pub destination_ref: Option<String>,
}

#[derive(Debug, Error)]
pub enum DatafeedError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("XML parsing failed: {0}")]
    Xml(String),
}

#[derive(Debug, Clone, Serialize, ToSchema, IntoParams)]
pub struct VehicleJourney {
    pub recorded_at_time: Option<String>,
    pub valid_untile_time: Option<String>,
    pub item_identifier: Option<String>,

    pub line: Option<String>,
    pub direction_ref: Option<String>,
    pub published_line_name: Option<String>,
    pub operator: Option<String>,

    pub origin_ref: Option<String>,
    pub origin_name: Option<String>,
    pub destination_ref: Option<String>,
    pub destination: Option<String>,

    pub origin_aimed_departure_time: Option<String>,
    pub destination_aimed_arrival_time: Option<String>,

    pub vehicle_location_longitude: Option<f64>,
    pub vehicle_location_latitude: Option<f64>,
    pub bearing: Option<f64>,

    pub vehicle: Option<String>,
    pub block_ref: Option<String>,

    pub data_frame_ref: Option<String>,
    pub dated_vehicle_journey_ref: Option<String>,

    pub vehicle_unique_id: Option<String>,
    pub ticket_machine_service_code: Option<String>,
    pub journey_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BodsSiriVmClient {
    client: Client,
    api_key: String,
}

impl BodsSiriVmClient {
    pub fn new(api_key: impl Into<String>) -> Result<Self, DatafeedError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.into(),
        })
    }

    pub fn convert(data: ServiceDelivery) -> Vec<VehicleJourney> {
        let mut journeys = Vec::new();

        for monitor in data.vehicle_monitoring_delivery {
            for activity in monitor.vehicle_activity {
                let monitored = activity.monitored_vehicle_journey;

                let extensions = activity.extensions;
                let extension_journey = extensions
                    .as_ref()
                    .and_then(|e| e.vehicle_journey.as_ref());

                let ticket_machine = extension_journey
                    .and_then(|j| j.operational.as_ref())
                    .and_then(|o| o.ticket_machine.as_ref());

                let framed_journey = monitored
                    .as_ref()
                    .and_then(|j| j.framed_vehicle_journey_ref.as_ref());

                let location = monitored
                    .as_ref()
                    .and_then(|j| j.vehicle_location.as_ref());

                journeys.push(VehicleJourney {
                    recorded_at_time: activity.recorded_at_time,
                    valid_untile_time: activity.valid_until_time,

                    line: monitored.as_ref().and_then(|j| j.line_ref.clone()),
                    direction_ref: monitored
                        .as_ref()
                        .and_then(|j| j.direction_ref.clone()),
                    published_line_name: monitored
                        .as_ref()
                        .and_then(|j| j.published_line_name.clone()),

                    operator: monitored
                        .as_ref()
                        .and_then(|j| j.operator_ref.clone()),

                    origin_ref: monitored
                        .as_ref()
                        .and_then(|j| j.origin_ref.clone()),
                    origin_name: monitored
                        .as_ref()
                        .and_then(|j| j.origin_name.clone()),

                    destination_ref: monitored
                        .as_ref()
                        .and_then(|j| j.destination_ref.clone()),
                    destination: monitored
                        .as_ref()
                        .and_then(|j| j.destination_name.clone()),

                    origin_aimed_departure_time: monitored
                        .as_ref()
                        .and_then(|j| j.origin_aimed_departure_time.clone()),
                    destination_aimed_arrival_time: monitored
                        .as_ref()
                        .and_then(|j| j.destination_aimed_arrival_time.clone()),

                    vehicle_location_longitude: location.map(|l| l.longitude),
                    vehicle_location_latitude: location.map(|l| l.latitude),
                    bearing: monitored.as_ref().and_then(|j| j.bearing),

                    vehicle: monitored
                        .as_ref()
                        .and_then(|j| j.vehicle_ref.clone()),
                    block_ref: monitored
                        .as_ref()
                        .and_then(|j| j.block_ref.clone()),

                    item_identifier: activity.item_identifier,

                    data_frame_ref: framed_journey
                        .and_then(|j| j.data_frame_ref.clone()),
                    dated_vehicle_journey_ref: framed_journey
                        .and_then(|j| j.dated_vehicle_journey_ref.clone()),

                    vehicle_unique_id: extension_journey
                        .and_then(|j| j.vehicle_unique_id.clone()),

                    ticket_machine_service_code: ticket_machine
                        .and_then(|t| t.ticket_machine_service_code.clone()),
                    journey_code: ticket_machine
                        .and_then(|t| t.journey_code.clone()),
                });
            }
        }

        journeys
    }

    pub async fn get_datafeed(
        &self,
        query: &DatafeedQuery,
    ) -> Result<ServiceDelivery, DatafeedError> {
        let mut request = self
            .client
            .get(DATAFEED_URL)
            .query(&[("api_key", self.api_key.as_str())]);

        if let Some([min_lon, min_lat, max_lon, max_lat]) =
            query.bounding_box
        {
            request = request.query(&[
                ("boundingBox", min_lon.to_string()),
                ("boundingBox", min_lat.to_string()),
                ("boundingBox", max_lon.to_string()),
                ("boundingBox", max_lat.to_string()),
            ]);
        }

        for operator in &query.operator_ref {
            request = request.query(&[("operatorRef", operator)]);
        }

        if let Some(value) = &query.vehicle_ref {
            request = request.query(&[("vehicleRef", value)]);
        }

        if let Some(value) = &query.line_ref {
            request = request.query(&[("lineRef", value)]);
        }

        if let Some(value) = &query.producer_ref {
            request = request.query(&[("producerRef", value)]);
        }

        if let Some(value) = &query.origin_ref {
            request = request.query(&[("originRef", value)]);
        }

        if let Some(value) = &query.destination_ref {
            request = request.query(&[("destinationRef", value)]);
        }

        let response = request.send().await?.error_for_status()?;
        let body = response.text().await?;

        let siri: SiriResponse = quick_xml::de::from_str(&body)
            .map_err(|err| {
                eprintln!("BODS XML parsing failed: {err}");
                DatafeedError::Xml(err.to_string())
            })?;

        Ok(siri.service_delivery)
    }
}