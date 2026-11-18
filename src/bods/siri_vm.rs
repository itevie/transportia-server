use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SiriResponse {
    #[serde(rename = "ServiceDelivery")]
    pub service_delivery: ServiceDelivery,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceDelivery {
    pub response_timestamp: Option<String>,
    pub producer_ref: Option<String>,

    #[serde(default)]
    pub vehicle_monitoring_delivery: Vec<VehicleMonitoringDelivery>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleMonitoringDelivery {
    pub response_timestamp: Option<String>,
    pub request_message_ref: Option<String>,
    pub valid_until: Option<String>,
    pub shortest_possible_cycle: Option<String>,

    #[serde(default)]
    pub vehicle_activity: Vec<VehicleActivity>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleActivity {
    pub recorded_at_time: Option<String>,
    pub item_identifier: Option<String>,
    pub valid_until_time: Option<String>,
    pub monitored_vehicle_journey: Option<MonitoredVehicleJourney>,
    pub extensions: Option<VehicleActivityExtensions>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MonitoredVehicleJourney {
    pub line_ref: Option<String>,
    pub direction_ref: Option<String>,

    pub framed_vehicle_journey_ref: Option<FramedVehicleJourneyRef>,

    pub published_line_name: Option<String>,
    pub operator_ref: Option<String>,

    pub origin_ref: Option<String>,
    pub origin_name: Option<String>,

    pub destination_ref: Option<String>,
    pub destination_name: Option<String>,

    pub origin_aimed_departure_time: Option<String>,
    pub destination_aimed_arrival_time: Option<String>,

    pub vehicle_location: Option<VehicleLocation>,
    pub bearing: Option<f64>,

    pub block_ref: Option<String>,
    pub vehicle_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FramedVehicleJourneyRef {
    pub data_frame_ref: Option<String>,
    pub dated_vehicle_journey_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleLocation {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleActivityExtensions {
    pub vehicle_journey: Option<ExtensionVehicleJourney>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExtensionVehicleJourney {
    pub operational: Option<Operational>,
    pub vehicle_unique_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Operational {
    pub ticket_machine: Option<TicketMachine>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TicketMachine {
    pub ticket_machine_service_code: Option<String>,
    pub journey_code: Option<String>,
}
