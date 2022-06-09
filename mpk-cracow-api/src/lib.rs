use serde::Deserialize;
use std::fmt;

pub const TTSS_TRAM_API_URL: &str = "http://www.ttss.krakow.pl/internetservice";
pub const TTSS_BUS_API_URL: &str = "http://ttss.mpk.krakow.pl/internetservice";

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutocompleteType {
    Divider,
    Stop,
}

#[derive(Deserialize)]
pub struct AutocompleteResult {
    pub name: String,
    pub count: Option<i32>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: AutocompleteType,
}

async fn autocomplete_stop_names(
    api_url: &str,
    query: &str,
) -> reqwest::Result<Vec<AutocompleteResult>> {
    reqwest::get(format!(
        "{}/services/lookup/autocomplete/json?query={}",
        api_url, query
    ))
    .await?
    .json::<Vec<AutocompleteResult>>()
    .await
}

pub async fn autocomplete_tram_stop_names(query: &str) -> reqwest::Result<Vec<AutocompleteResult>> {
    autocomplete_stop_names(TTSS_TRAM_API_URL, query).await
}

pub async fn autocomplete_bus_stop_names(query: &str) -> reqwest::Result<Vec<AutocompleteResult>> {
    autocomplete_stop_names(TTSS_BUS_API_URL, query).await
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StopCategories {
    Tram,
    Bus,
    Other,
}

#[derive(Deserialize)]
pub struct Stop {
    pub category: StopCategories,
    pub id: String,     // u64 sent as a JSON string
    pub latitude: u32, // you need to divide the coordinates by 3 600 000 to get a proper coordinate
    pub longitude: u32, // you need to divide the coordinates by 3 600 000 to get a proper coordinate
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: u32, // short unsigned int (u16-ish?) sent as a JSON string
}

#[derive(Deserialize)]
struct StopArray {
    pub stops: Vec<Stop>,
}

async fn fetch_all_stops(api_url: &str) -> reqwest::Result<Vec<Stop>> {
    let res = reqwest::get(api_url.to_owned() + "/geoserviceDispatcher/services/stopinfo/stops?left=-648000000&bottom=-324000000&right=648000000&top=324000000")
        .await?
        .json::<StopArray>()
        .await?;

    Ok(res.stops)
}

pub async fn fetch_all_tram_stops() -> reqwest::Result<Vec<Stop>> {
    fetch_all_stops(TTSS_TRAM_API_URL).await
}

pub async fn fetch_all_bus_stops() -> reqwest::Result<Vec<Stop>> {
    fetch_all_stops(TTSS_BUS_API_URL).await
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Arrival,
    Departure,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Arrival => write!(f, "arrival"),
            Mode::Departure => write!(f, "departure"),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StopInfoStatus {
    Predicted,
    Departed,
    Stopping,
}

impl fmt::Display for StopInfoStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StopInfoStatus::Predicted => write!(f, "PREDICTED"),
            StopInfoStatus::Departed => write!(f, "DEPARTED"),
            StopInfoStatus::Stopping => write!(f, "STOPPING"),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopInfoTrips {
    pub actual_relative_time: i32, // time till (from for negative values) arrival/departure in seconds.
    pub actual_time: Option<String>, // HH:MM if the system can detect the location of the tram.
    // Not present if it's not possible, or if they're old trips.
    pub direction: String,
    pub mixed_time: String, // Time as displayed on the signs.
    // MM %UNIT_MIN% if the system can detect the location of the tram.
    // HH:MM otherwise.
    // 0 %UNIT_MIN% for old trips.
    pub passageid: String,    // Negative number (~i64?)
    pub pattern_text: String, // Line "number".
    pub planned_time: String, // Planned time in HH:MM.
    pub route_id: String, // Internal route ID. Unique for each route. Direction does not matter.
    pub status: StopInfoStatus,
    pub trip_id: String,    // Seems to be an unique ID for each trip.
    pub vehicle_id: String, // Negative number (~i64?)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopInfoRoutes {
    //pub alerts: Vec<???>  // empty in testing
    pub authority: String, // Who manages the line. Two primary authorities would probably be MPK and Mobilis, but I'm not certain about it, so I left it as is.
    pub directions: [String; 2],
    pub id: String,
    pub name: String,       // Line "number".
    pub route_type: String, // tram or bus
    pub short_name: String, // Also line "number".
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopInfo {
    pub actual: Vec<StopInfoTrips>,
    //pub directions: Vec<???> // empty in testing
    pub first_passage_time: i64, // Unix timestamp in milliseconds
    //pub generalAlerts: Vec<???> // empty in testing
    pub last_passage_time: i64, // Unix timestamp in milliseconds
    pub old: Vec<StopInfoTrips>,
    pub routes: Vec<StopInfoRoutes>,
    pub stop_name: String,
    pub stop_short_name: u32, // short unsigned int (u16-ish?) sent as a JSON string
}

async fn stop_info(api_url: &str, stop: u32, mode: Mode) -> reqwest::Result<StopInfo> {
    reqwest::get(format!(
        "{}/services/passageInfo/stopPassages/stopPoint?stopPoint={}&mode={}",
        api_url, stop, mode
    ))
    .await?
    .json::<StopInfo>()
    .await
}

pub async fn tram_stop_info(stop: u32, mode: Mode) -> reqwest::Result<StopInfo> {
    stop_info(TTSS_TRAM_API_URL, stop, mode).await
}

pub async fn bus_stop_info(stop: u32, mode: Mode) -> reqwest::Result<StopInfo> {
    stop_info(TTSS_BUS_API_URL, stop, mode).await
}
