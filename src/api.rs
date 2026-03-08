use anyhow::{Context, Result};
use crate::types::*;

const FORECAST_URL: &str = "http://api.weatherapi.com/v1/forecast.json";

pub async fn fetch_all(key: &str, query: &str) -> Result<(Location, WeatherResponse, Option<AirQualityResponse>, String)> {
    let url = format!(
        "{}?key={}&q={}&days=7&aqi=yes&alerts=yes",
        FORECAST_URL, key, query
    );

    let response = reqwest::get(&url).await?;
    let text = response.text().await.context("Failed to fetch weather data")?;
    let resp: WapiResponse = serde_json::from_str(&text).context("Failed to parse weather data")?;

    let (location, weather, air_quality) = convert_response(resp);

    Ok((location, weather, air_quality, text))
}

pub fn parse_cached(text: &str) -> Result<(Location, WeatherResponse, Option<AirQualityResponse>)> {
    let resp: WapiResponse = serde_json::from_str(text).context("Failed to parse cached weather data")?;
    Ok(convert_response(resp))
}

fn convert_response(resp: WapiResponse) -> (Location, WeatherResponse, Option<AirQualityResponse>) {
    let location = Location {
        name: resp.location.name,
        country: resp.location.country,
        latitude: resp.location.lat,
        longitude: resp.location.lon,
    };

    // Flatten hourly data from all forecast days
    let mut h_time = Vec::new();
    let mut h_temp = Vec::new();
    let mut h_rain = Vec::new();
    let mut h_code = Vec::new();

    for day in &resp.forecast.forecastday {
        for hour in &day.hour {
            // Convert "2024-01-01 00:00" → "2024-01-01T00:00"
            h_time.push(hour.time.replace(' ', "T"));
            h_temp.push(hour.temp_c);
            h_rain.push(hour.chance_of_rain);
            h_code.push(wapi_to_wmo(hour.condition.code, hour.is_day != 0));
        }
    }

    // Build daily arrays
    let mut d_time = Vec::new();
    let mut d_code = Vec::new();
    let mut d_max = Vec::new();
    let mut d_min = Vec::new();
    let mut d_sunrise = Vec::new();
    let mut d_sunset = Vec::new();
    let mut d_rain = Vec::new();
    let mut d_uv = Vec::new();

    for day in &resp.forecast.forecastday {
        d_time.push(day.date.clone());
        d_code.push(wapi_to_wmo(day.day.condition.code, true));
        d_max.push(day.day.maxtemp_c);
        d_min.push(day.day.mintemp_c);
        d_sunrise.push(time_12h_to_24h(&day.astro.sunrise));
        d_sunset.push(time_12h_to_24h(&day.astro.sunset));
        d_rain.push(day.day.daily_chance_of_rain);
        d_uv.push(day.day.uv);
    }

    // Convert alerts
    let alerts: Vec<Alert> = resp.alerts
        .map(|a| a.alert.into_iter().filter_map(|wa| {
            Some(Alert {
                headline: wa.headline.unwrap_or_default(),
                severity: wa.severity.unwrap_or_default(),
                event: wa.event.unwrap_or_default(),
                expires: wa.expires.unwrap_or_default(),
            })
        }).collect())
        .unwrap_or_default();

    let weather = WeatherResponse {
        current: CurrentWeather {
            temperature_2m: resp.current.temp_c,
            relative_humidity_2m: resp.current.humidity,
            apparent_temperature: resp.current.feelslike_c,
            weather_code: wapi_to_wmo(resp.current.condition.code, resp.current.is_day != 0),
            wind_speed_10m: resp.current.wind_kph,
            wind_direction_10m: resp.current.wind_degree,
            surface_pressure: resp.current.pressure_mb,
            uv_index: resp.current.uv,
            is_day: resp.current.is_day,
        },
        hourly: HourlyWeather {
            time: h_time,
            temperature_2m: h_temp,
            precipitation_probability: h_rain,
            weather_code: h_code,
        },
        daily: DailyWeather {
            time: d_time,
            weather_code: d_code,
            temperature_2m_max: d_max,
            temperature_2m_min: d_min,
            sunrise: d_sunrise,
            sunset: d_sunset,
            precipitation_probability_max: d_rain,
            uv_index_max: d_uv,
        },
        alerts,
    };

    let air_quality = resp.current.air_quality.map(|aq| {
        AirQualityResponse {
            current: CurrentAirQuality {
                us_aqi: pm25_to_aqi(aq.pm2_5),
                pm2_5: aq.pm2_5,
                pm10: aq.pm10,
            },
        }
    });

    (location, weather, air_quality)
}

// Convert WeatherAPI condition code → WMO weather code
fn wapi_to_wmo(code: u32, is_day: bool) -> u32 {
    match code {
        1000 => if is_day { 0 } else { 0 }, // Clear
        1003 => 2,                           // Partly cloudy
        1006 => 3,                           // Cloudy
        1009 => 3,                           // Overcast
        1030 | 1135 => 45,                   // Mist / Fog
        1147 => 48,                          // Freezing fog
        1150 | 1153 => 51,                   // Drizzle
        1168 | 1171 => 56,                   // Freezing drizzle
        1180 | 1183 => 61,                   // Light rain
        1186 | 1189 => 63,                   // Moderate rain
        1192 | 1195 => 65,                   // Heavy rain
        1198 | 1204 => 66,                   // Freezing rain / sleet
        1201 | 1207 => 67,                   // Heavy freezing rain
        1063 | 1240 => 80,                   // Rain showers
        1243 => 81,                          // Heavy rain showers
        1246 => 82,                          // Torrential showers
        1066 | 1210 | 1213 => 71,            // Light snow
        1114 | 1216 | 1219 => 73,            // Moderate snow
        1117 | 1222 | 1225 => 75,            // Heavy snow
        1237 | 1261 | 1264 => 77,            // Ice pellets
        1069 | 1249 | 1252 | 1255 => 85,     // Snow/sleet showers
        1258 => 86,                          // Heavy snow showers
        1072 | 1087 | 1273 => 95,            // Thunder
        1276 | 1279 | 1282 => 96,            // Heavy thunder
        _ => 3,                              // Fallback: cloudy
    }
}

// Convert "06:30 AM" / "06:30 PM" → "06:30" / "18:30"
fn time_12h_to_24h(t: &str) -> String {
    let t = t.trim();
    let is_pm = t.ends_with("PM");
    let time_part = t.trim_end_matches("AM").trim_end_matches("PM").trim();

    let mut parts = time_part.split(':');
    let hour: u32 = parts.next().and_then(|h| h.parse().ok()).unwrap_or(0);
    let min: u32 = parts.next().and_then(|m| m.parse().ok()).unwrap_or(0);

    let hour_24 = match (hour, is_pm) {
        (12, false) => 0,    // 12:xx AM → 00:xx
        (12, true) => 12,    // 12:xx PM → 12:xx
        (h, true) => h + 12, // 1-11 PM → 13-23
        (h, false) => h,     // 1-11 AM → 1-11
    };

    format!("{:02}:{:02}", hour_24, min)
}

// Calculate US EPA AQI from PM2.5 concentration
fn pm25_to_aqi(pm25: f64) -> f64 {
    let breakpoints: &[(f64, f64, f64, f64)] = &[
        (0.0, 12.0, 0.0, 50.0),
        (12.1, 35.4, 51.0, 100.0),
        (35.5, 55.4, 101.0, 150.0),
        (55.5, 150.4, 151.0, 200.0),
        (150.5, 250.4, 201.0, 300.0),
        (250.5, 350.4, 301.0, 400.0),
        (350.5, 500.4, 401.0, 500.0),
    ];

    let c = pm25.clamp(0.0, 500.4);
    for &(c_lo, c_hi, i_lo, i_hi) in breakpoints {
        if c <= c_hi {
            return (i_hi - i_lo) / (c_hi - c_lo) * (c - c_lo) + i_lo;
        }
    }
    500.0
}
