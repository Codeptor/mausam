use serde::Deserialize;

// ─── Internal Types (used by display) ────────────────

#[derive(Debug, Clone)]
pub struct Location {
    pub name: String,
    pub country: String,
    #[allow(dead_code)]
    pub latitude: f64,
    #[allow(dead_code)]
    pub longitude: f64,
    pub localtime: String,
}

pub struct WeatherResponse {
    pub current: CurrentWeather,
    pub hourly: HourlyWeather,
    pub daily: DailyWeather,
    pub alerts: Vec<Alert>,
}

pub struct CurrentWeather {
    pub temperature_2m: f64,
    pub relative_humidity_2m: f64,
    pub apparent_temperature: f64,
    pub weather_code: u32,
    pub wind_speed_10m: f64,
    pub wind_direction_10m: f64,
    pub surface_pressure: f64,
    pub uv_index: f64,
    pub is_day: u8,
    pub visibility_km: f64,
    pub dewpoint_c: f64,
}

pub struct HourlyWeather {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
    pub precipitation_probability: Vec<f64>,
    pub weather_code: Vec<u32>,
}

pub struct DailyWeather {
    pub time: Vec<String>,
    pub weather_code: Vec<u32>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub sunrise: Vec<String>,
    pub sunset: Vec<String>,
    pub precipitation_probability_max: Vec<f64>,
    #[allow(dead_code)]
    pub uv_index_max: Vec<f64>,
    pub moon_phase: Vec<String>,
    pub moon_illumination: Vec<u32>,
    pub moonrise: Vec<String>,
    pub moonset: Vec<String>,
}

pub struct AirQualityResponse {
    pub current: CurrentAirQuality,
}

pub struct CurrentAirQuality {
    pub us_aqi: f64,
    pub pm2_5: f64,
    pub pm10: f64,
}

#[allow(dead_code)]
pub struct Alert {
    pub headline: String,
    pub severity: String,
    pub event: String,
    pub expires: String,
}

// ─── WeatherAPI.com Raw Response ─────────────────────

#[derive(Deserialize)]
pub struct WapiResponse {
    pub location: WapiLocation,
    pub current: WapiCurrent,
    pub forecast: WapiForecast,
    pub alerts: Option<WapiAlerts>,
}

#[derive(Deserialize)]
pub struct WapiLocation {
    pub name: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    pub localtime: String,
}

#[derive(Deserialize)]
pub struct WapiCurrent {
    pub temp_c: f64,
    pub feelslike_c: f64,
    pub humidity: f64,
    pub wind_kph: f64,
    pub wind_degree: f64,
    pub pressure_mb: f64,
    pub uv: f64,
    pub is_day: u8,
    pub vis_km: f64,
    pub dewpoint_c: f64,
    pub condition: WapiCondition,
    pub air_quality: Option<WapiAirQuality>,
}

#[derive(Deserialize)]
pub struct WapiCondition {
    pub code: u32,
}

#[derive(Deserialize)]
pub struct WapiAirQuality {
    pub pm2_5: f64,
    pub pm10: f64,
}

#[derive(Deserialize)]
pub struct WapiForecast {
    pub forecastday: Vec<WapiForecastDay>,
}

#[derive(Deserialize)]
pub struct WapiForecastDay {
    pub date: String,
    pub day: WapiDay,
    pub astro: WapiAstro,
    pub hour: Vec<WapiHour>,
}

#[derive(Deserialize)]
pub struct WapiDay {
    pub maxtemp_c: f64,
    pub mintemp_c: f64,
    pub daily_chance_of_rain: f64,
    pub uv: f64,
    pub condition: WapiCondition,
}

#[derive(Deserialize)]
pub struct WapiAstro {
    pub sunrise: String,
    pub sunset: String,
    pub moonrise: String,
    pub moonset: String,
    pub moon_phase: String,
    pub moon_illumination: u32,
}

#[derive(Deserialize)]
pub struct WapiHour {
    pub time: String,
    pub temp_c: f64,
    pub chance_of_rain: f64,
    pub is_day: u8,
    pub condition: WapiCondition,
}

#[derive(Deserialize)]
pub struct WapiAlerts {
    pub alert: Vec<WapiAlert>,
}

#[derive(Deserialize)]
pub struct WapiAlert {
    pub headline: Option<String>,
    pub severity: Option<String>,
    pub event: Option<String>,
    pub expires: Option<String>,
}
