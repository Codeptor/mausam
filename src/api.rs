use anyhow::{Context, Result};
use crate::types::*;

const WEATHER_URL: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_QUALITY_URL: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";
const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const IP_LOCATE_URL: &str = "http://ip-api.com/json/";

pub async fn locate_ip() -> Result<Location> {
    let resp: IpLocation = reqwest::get(IP_LOCATE_URL)
        .await?
        .json()
        .await
        .context("Failed to detect location from IP")?;

    Ok(Location {
        name: resp.city,
        country: resp.country,
        latitude: resp.lat,
        longitude: resp.lon,
    })
}

pub async fn geocode(city: &str) -> Result<Location> {
    let url = format!("{}?name={}&count=1&language=en", GEOCODE_URL, city);
    let resp: GeocodeResponse = reqwest::get(&url)
        .await?
        .json()
        .await
        .context("Failed to geocode city")?;

    let result = resp
        .results
        .and_then(|r| r.into_iter().next())
        .with_context(|| format!("City '{}' not found", city))?;

    Ok(Location {
        name: result.name,
        country: result.country,
        latitude: result.latitude,
        longitude: result.longitude,
    })
}

pub async fn fetch_weather(loc: &Location) -> Result<WeatherResponse> {
    let url = format!(
        "{}?latitude={}&longitude={}\
         &current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,\
         wind_speed_10m,wind_direction_10m,surface_pressure,uv_index,is_day\
         &hourly=temperature_2m,precipitation_probability,weather_code\
         &daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,\
         precipitation_probability_max,uv_index_max\
         &timezone=auto&forecast_days=7",
        WEATHER_URL, loc.latitude, loc.longitude
    );

    reqwest::get(&url)
        .await?
        .json()
        .await
        .context("Failed to fetch weather data")
}

pub async fn fetch_air_quality(loc: &Location) -> Result<AirQualityResponse> {
    let url = format!(
        "{}?latitude={}&longitude={}&current=us_aqi,pm2_5,pm10",
        AIR_QUALITY_URL, loc.latitude, loc.longitude
    );

    reqwest::get(&url)
        .await?
        .json()
        .await
        .context("Failed to fetch air quality data")
}
