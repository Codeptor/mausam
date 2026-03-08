use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Location {
    pub name: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

// --- IP Geolocation (ip-api.com) ---

#[derive(Deserialize)]
pub struct IpLocation {
    pub city: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
}

// --- Open-Meteo Geocoding ---

#[derive(Deserialize)]
pub struct GeocodeResponse {
    pub results: Option<Vec<GeocodeResult>>,
}

#[derive(Deserialize)]
pub struct GeocodeResult {
    pub name: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

// --- Open-Meteo Weather ---

#[derive(Deserialize)]
pub struct WeatherResponse {
    pub current: CurrentWeather,
    pub hourly: HourlyWeather,
    pub daily: DailyWeather,
}

#[derive(Deserialize)]
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
}

#[derive(Deserialize)]
pub struct HourlyWeather {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
    pub precipitation_probability: Vec<f64>,
    pub weather_code: Vec<u32>,
}

#[derive(Deserialize)]
pub struct DailyWeather {
    pub time: Vec<String>,
    pub weather_code: Vec<u32>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub sunrise: Vec<String>,
    pub sunset: Vec<String>,
    pub precipitation_probability_max: Vec<f64>,
    pub uv_index_max: Vec<f64>,
}

// --- Open-Meteo Air Quality ---

#[derive(Deserialize)]
pub struct AirQualityResponse {
    pub current: CurrentAirQuality,
}

#[derive(Deserialize)]
pub struct CurrentAirQuality {
    pub us_aqi: f64,
    pub pm2_5: f64,
    pub pm10: f64,
}
