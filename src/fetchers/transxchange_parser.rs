use anyhow::{Context, Result};
use chrono::Local;
use quick_xml::de::from_reader;
use serde::Deserialize;
use crate::db::resource::transxchange::line_def::TxLine;
use crate::db::resource::transxchange::operator_def::TxOperator;
use crate::db::resource::transxchange::service_def::TxService;
use crate::db::resource::transxchange::vehicle_journey_def::TxVehicleJourney;

#[derive(Debug, Deserialize)]
struct TransXChange {
    #[serde(rename = "Operators")]
    operators: Operators,

    #[serde(rename = "Services")]
    services: Services,

    #[serde(rename = "VehicleJourneys")]
    vehicle_journeys: Option<VehicleJourneys>,
}

#[derive(Debug, Deserialize)]
struct Operators {
    #[serde(rename = "Operator", default)]
    operators: Vec<XmlOperator>,
}

#[derive(Debug, Deserialize)]
struct XmlOperator {
    #[serde(rename = "@id")]
    _id: String,

    #[serde(rename = "NationalOperatorCode")]
    national_code: Option<String>,

    #[serde(rename = "OperatorCode")]
    code: Option<String>,

    #[serde(rename = "OperatorShortName")]
    short_name: String,

    #[serde(rename = "LicenceNumber")]
    licence_number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Services {
    #[serde(rename = "Service", default)]
    services: Vec<Service>,
}

#[derive(Debug, Deserialize)]
struct Service {
    #[serde(rename = "ServiceCode")]
    service_code: String,

    #[serde(rename = "Lines")]
    lines: Lines,

    #[serde(rename = "StandardService")]
    standard_service: Option<StandardService>,

    #[serde(rename = "OperatingPeriod")]
    operating_period: Option<OperatingPeriod>,

    #[serde(rename = "OperatingProfile")]
    operating_profile: Option<OperatingProfile>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct OperatingPeriod {
    #[serde(rename = "StartDate")]
    start_date: Option<String>,

    #[serde(rename = "EndDate")]
    end_date: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct OperatingProfile {
    #[serde(rename = "RegularDayType")]
    regular_day_type: Option<RegularDayType>,

    #[serde(rename = "SpecialDaysOperation")]
    special_days_operation: Option<SpecialDaysOperation>,

    #[serde(rename = "BankHolidayOperation")]
    bank_holiday_operation: Option<BankHolidayOperation>,

    #[serde(rename = "ServicedOrganisationDayType")]
    serviced_organisation_day_type: Option<ServicedOrganisationDayType>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SpecialDaysOperation {
    #[serde(rename = "DaysOfOperation")]
    days_of_operation: Option<SpecialDates>,

    #[serde(rename = "DaysOfNonOperation")]
    days_of_non_operation: Option<SpecialDates>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SpecialDates {
    #[serde(rename = "Date", default)]
    dates: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct BankHolidayOperation {
    #[serde(rename = "DaysOfNonOperation")]
    days_of_non_operation: Option<BankHolidayDays>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct BankHolidayDays {
    #[serde(rename = "ChristmasDay")]
    christmas_day: Option<EmptyElement>,
    #[serde(rename = "BoxingDay")]
    boxing_day: Option<EmptyElement>,
    #[serde(rename = "GoodFriday")]
    good_friday: Option<EmptyElement>,
    #[serde(rename = "NewYearsDay")]
    new_years_day: Option<EmptyElement>,
    #[serde(rename = "LateSummerBankHolidayNotScotland")]
    late_summer_bank_holiday_not_scotland: Option<EmptyElement>,
    #[serde(rename = "MayDay")]
    may_day: Option<EmptyElement>,
    #[serde(rename = "EasterMonday")]
    easter_monday: Option<EmptyElement>,
    #[serde(rename = "SpringBank")]
    spring_bank: Option<EmptyElement>,
    #[serde(rename = "ChristmasDayHoliday")]
    christmas_day_holiday: Option<EmptyElement>,
    #[serde(rename = "BoxingDayHoliday")]
    boxing_day_holiday: Option<EmptyElement>,
    #[serde(rename = "NewYearsDayHoliday")]
    new_years_day_holiday: Option<EmptyElement>,
    #[serde(rename = "ChristmasEve")]
    christmas_eve: Option<EmptyElement>,
    #[serde(rename = "NewYearsEve")]
    new_years_eve: Option<EmptyElement>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ServicedOrganisationDayType {
    #[serde(rename = "DaysOfNonOperation")]
    days_of_non_operation: Option<ServicedOrganisationDays>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ServicedOrganisationDays {
    #[serde(rename = "WorkingDays")]
    working_days: Option<WorkingDays>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct WorkingDays {
    #[serde(rename = "ServicedOrganisationRef", default)]
    refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Lines {
    #[serde(rename = "Line", default)]
    lines: Vec<XmlLine>,
}

#[derive(Debug, Deserialize)]
struct XmlLine {
    #[serde(rename = "@id")]
    id: String,

    #[serde(rename = "LineName")]
    name: String,

    #[serde(rename = "OutboundDescription")]
    outbound_description: Option<Description>,

    #[serde(rename = "InboundDescription")]
    inbound_description: Option<Description>,
}

#[derive(Debug, Deserialize)]
struct Description {
    #[serde(rename = "Description")]
    description: String,
}

#[derive(Debug, Deserialize)]
struct StandardService {
    #[serde(rename = "JourneyPattern", default)]
    journey_patterns: Vec<JourneyPattern>,
}

#[derive(Debug, Deserialize)]
struct JourneyPattern {
    #[serde(rename = "@id")]
    id: String,

    #[serde(rename = "DestinationDisplay")]
    destination_display: Option<String>,

    #[serde(rename = "Direction")]
    direction: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VehicleJourneys {
    #[serde(rename = "VehicleJourney", default)]
    journeys: Vec<XmlVehicleJourney>,
}

#[derive(Debug, Deserialize)]
struct XmlVehicleJourney {
    #[serde(rename = "OperatorRef")]
    operator_ref: Option<String>,

    #[serde(rename = "Operational")]
    operational: Option<Operational>,

    #[serde(rename = "OperatingProfile")]
    operating_profile: Option<OperatingProfile>,

    #[serde(rename = "GarageRef")]
    garage_ref: Option<String>,

    #[serde(rename = "VehicleJourneyCode")]
    vehicle_journey_code: Option<String>,

    #[serde(rename = "ServiceRef")]
    service_ref: Option<String>,

    #[serde(rename = "LineRef")]
    line_ref: Option<String>,

    #[serde(rename = "JourneyPatternRef")]
    journey_pattern_ref: Option<String>,

    #[serde(rename = "DepartureTime")]
    departure_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Operational {
    #[serde(rename = "Block")]
    block: Option<Block>,

    #[serde(rename = "TicketMachine")]
    ticket_machine: Option<TicketMachine>,
}

#[derive(Debug, Deserialize)]
struct Block {
    #[serde(rename = "Description")]
    description: Option<String>,

    #[serde(rename = "BlockNumber")]
    number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TicketMachine {
    #[serde(rename = "TicketMachineServiceCode")]
    service_code: Option<String>,

    #[serde(rename = "JourneyCode")]
    journey_code: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RegularDayType {
    #[serde(rename = "DaysOfWeek")]
    days_of_week: Option<DaysOfWeek>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct DaysOfWeek {
    #[serde(rename = "Monday")]
    monday: Option<EmptyElement>,
    #[serde(rename = "Tuesday")]
    tuesday: Option<EmptyElement>,
    #[serde(rename = "Wednesday")]
    wednesday: Option<EmptyElement>,
    #[serde(rename = "Thursday")]
    thursday: Option<EmptyElement>,
    #[serde(rename = "Friday")]
    friday: Option<EmptyElement>,
    #[serde(rename = "Saturday")]
    saturday: Option<EmptyElement>,
    #[serde(rename = "Sunday")]
    sunday: Option<EmptyElement>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct EmptyElement {}

pub fn parse_transxchange(
    xml: &[u8],
) -> Result<(TxOperator, Vec<TxLine>, Vec<TxVehicleJourney>, Vec<TxService>)> {
    let document: TransXChange =
        from_reader(xml).context("Failed to parse TransXChange XML")?;

    let operator = document
        .operators
        .operators
        .into_iter()
        .next()
        .context("TransXChange document contains no operators")
        .map(|operator| TxOperator {
            id: format!(
                "{}:{}",
                operator.national_code.as_deref().unwrap_or(""),
                operator.code.as_deref().unwrap_or("")
            ),
            national_code: operator.national_code,
            code: operator.code,
            short_name: operator.short_name,
            licence_number: operator.licence_number,
            comment_thread: None
        })?;

    let operator_code = operator.code.clone();
    let operator_national_code = operator.national_code.clone();

    let mut lines = Vec::new();
    let mut journeys = Vec::new();
    let mut tx_services = Vec::new();

    for journey in document
        .vehicle_journeys
        .map(|v| v.journeys)
        .unwrap_or_default()
    {
        let operational = journey.operational;
        let block = operational.as_ref().and_then(|o| o.block.as_ref());
        let ticket_machine = operational
            .as_ref()
            .and_then(|o| o.ticket_machine.as_ref());

        let profile = journey.operating_profile.as_ref();

        let days = profile
            .and_then(|p| p.regular_day_type.as_ref())
            .and_then(|d| d.days_of_week.as_ref());

        journeys.push(TxVehicleJourney {
            id: format!(
                "{}_{}_{}",
                journey.vehicle_journey_code.as_deref().unwrap_or(""),
                journey.line_ref.as_deref().unwrap_or(""),
                ticket_machine
                    .and_then(|tm| tm.journey_code.as_deref())
                    .unwrap_or("")
            ),
            operator_ref: operator_national_code.clone(),
            service_ref: journey.service_ref,
            line_ref: journey.line_ref,
            journey_pattern_ref: journey.journey_pattern_ref,
            departure_time: journey.departure_time,
            garage_ref: journey.garage_ref,

            block_description: block.and_then(|b| b.description.clone()),
            block_number: block.and_then(|b| b.number.clone()),

            ticket_machine_service_code: ticket_machine
                .and_then(|t| t.service_code.clone()),
            ticket_machine_journey_code: ticket_machine
                .and_then(|t| t.journey_code.clone()),

            monday: days.and_then(|d| d.monday.as_ref()).is_some(),
            tuesday: days.and_then(|d| d.tuesday.as_ref()).is_some(),
            wednesday: days.and_then(|d| d.wednesday.as_ref()).is_some(),
            thursday: days.and_then(|d| d.thursday.as_ref()).is_some(),
            friday: days.and_then(|d| d.friday.as_ref()).is_some(),
            saturday: days.and_then(|d| d.saturday.as_ref()).is_some(),
            sunday: days.and_then(|d| d.sunday.as_ref()).is_some(),
        });
    }

    for service in document.services.services {
        let profile = service.operating_profile.as_ref();

        let days = profile
            .and_then(|p| p.regular_day_type.as_ref())
            .and_then(|d| d.days_of_week.as_ref());

        let period = service.operating_period.as_ref();

        let today = Local::now().date_naive();

        if period
            .and_then(|p| p.end_date.as_deref())
            .and_then(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
            .is_some_and(|end_date| end_date < today)
        {
            continue;
        }

        let special_days = profile.and_then(|p| p.special_days_operation.as_ref());

        let bank_holidays = profile
            .and_then(|p| p.bank_holiday_operation.as_ref())
            .and_then(|b| b.days_of_non_operation.as_ref());

        let serviced_organisation_refs = profile
            .and_then(|p| p.serviced_organisation_day_type.as_ref())
            .and_then(|d| d.days_of_non_operation.as_ref())
            .and_then(|d| d.working_days.as_ref())
            .map(|d| d.refs.clone())
            .unwrap_or_default();

        let mut bank_holidays_non_operation = Vec::new();

        if let Some(b) = bank_holidays {
            macro_rules! add_holiday {
            ($field:ident) => {
                if b.$field.is_some() {
                    bank_holidays_non_operation.push(stringify!($field).to_string());
                }
            };
        }

            add_holiday!(christmas_day);
            add_holiday!(boxing_day);
            add_holiday!(good_friday);
            add_holiday!(new_years_day);
            add_holiday!(late_summer_bank_holiday_not_scotland);
            add_holiday!(may_day);
            add_holiday!(easter_monday);
            add_holiday!(spring_bank);
            add_holiday!(christmas_day_holiday);
            add_holiday!(boxing_day_holiday);
            add_holiday!(new_years_day_holiday);
            add_holiday!(christmas_eve);
            add_holiday!(new_years_eve);
        }

        let special_days_of_operation = special_days
            .and_then(|s| s.days_of_operation.as_ref())
            .map(|s| s.dates.clone())
            .unwrap_or_default();

        let special_days_of_non_operation = special_days
            .and_then(|s| s.days_of_non_operation.as_ref())
            .map(|s| s.dates.clone())
            .unwrap_or_default();

        tx_services.push(TxService {
            id: format!("{}_{}_{}", service.service_code.clone(), operator_national_code.clone().unwrap_or("".to_string()), uuid::Uuid::new_v4()),
            service_code: service.service_code.clone(),

            start_date: period.and_then(|p| p.start_date.clone()),
            end_date: period.and_then(|p| p.end_date.clone()),

            monday: days.and_then(|d| d.monday.as_ref()).is_some(),
            tuesday: days.and_then(|d| d.tuesday.as_ref()).is_some(),
            wednesday: days.and_then(|d| d.wednesday.as_ref()).is_some(),
            thursday: days.and_then(|d| d.thursday.as_ref()).is_some(),
            friday: days.and_then(|d| d.friday.as_ref()).is_some(),
            saturday: days.and_then(|d| d.saturday.as_ref()).is_some(),
            sunday: days.and_then(|d| d.sunday.as_ref()).is_some(),

            bank_holidays_non_operation: serde_json::to_string(
                &bank_holidays_non_operation,
            )?,

            special_days_of_operation: serde_json::to_string(
                &special_days_of_operation,
            )?,

            special_days_of_non_operation: serde_json::to_string(
                &special_days_of_non_operation,
            )?,

            serviced_organisation_refs: serde_json::to_string(
                &serviced_organisation_refs,
            )?,

            operator: operator_national_code.clone(),
        });

        for line in service.lines.lines {
            lines.push(TxLine {
                tx_id: line.id.clone(),
                name: line.name,
                operator_code: operator_code.clone(),
                operator_national_code: operator_national_code.clone(),
                line_id: line.id,
                service_code: service.service_code.clone(),
                outbound_description: line
                    .outbound_description
                    .map(|x| x.description),
                inbound_description: line
                    .inbound_description
                    .map(|x| x.description),
            });
        }
    }

    Ok((operator, lines, journeys, tx_services))
}