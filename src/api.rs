use crate::types::*;
use anyhow::{Context, Result};

const FORECAST_URL: &str = "http://api.weatherapi.com/v1/forecast.json";

pub async fn fetch_all(
    key: &str,
    query: &str,
    units: &str,
) -> Result<(
    Location,
    WeatherResponse,
    Option<AirQualityResponse>,
    String,
)> {
    let url = format!(
        "{}?key={}&q={}&days=7&aqi=yes&alerts=yes",
        FORECAST_URL, key, query
    );

    let mut last_status = 0u16;
    let mut text = String::new();

    for attempt in 0..3 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        let response = reqwest::get(&url).await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                anyhow::anyhow!("Could not connect. Check your internet connection.")
            } else {
                anyhow::anyhow!("Network error: {}", e)
            }
        })?;

        last_status = response.status().as_u16();
        text = response.text().await.context("Failed to read response")?;

        if last_status < 500 {
            break;
        }
    }

    if last_status >= 400 {
        match last_status {
            401 | 403 => anyhow::bail!("Invalid API key. Check your key at https://weatherapi.com"),
            429 => anyhow::bail!("API rate limit reached. Try again in a few minutes."),
            _ => {
                if text.contains("No matching location") {
                    anyhow::bail!("City not found. Try a different spelling.");
                }
                anyhow::bail!("API error ({}). Try again later.", last_status);
            }
        }
    }

    let resp: WapiResponse = serde_json::from_str(&text).context("Failed to parse weather data")?;

    let (location, weather, air_quality) = convert_response(resp, units);

    Ok((location, weather, air_quality, text))
}

pub fn parse_cached(
    text: &str,
    units: &str,
) -> Result<(Location, WeatherResponse, Option<AirQualityResponse>)> {
    let resp: WapiResponse =
        serde_json::from_str(text).context("Failed to parse cached weather data")?;
    Ok(convert_response(resp, units))
}

fn convert_response(
    resp: WapiResponse,
    units: &str,
) -> (Location, WeatherResponse, Option<AirQualityResponse>) {
    let imperial = units == "imperial";
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
    let mut d_moon_phase = Vec::new();
    let mut d_moon_illum = Vec::new();
    let mut d_moonrise = Vec::new();
    let mut d_moonset = Vec::new();

    for day in &resp.forecast.forecastday {
        d_time.push(day.date.clone());
        d_code.push(wapi_to_wmo(day.day.condition.code, true));
        d_max.push(day.day.maxtemp_c);
        d_min.push(day.day.mintemp_c);
        d_sunrise.push(time_12h_to_24h(&day.astro.sunrise));
        d_sunset.push(time_12h_to_24h(&day.astro.sunset));
        d_rain.push(day.day.daily_chance_of_rain);
        d_uv.push(day.day.uv);
        d_moon_phase.push(day.astro.moon_phase.clone());
        d_moon_illum.push(day.astro.moon_illumination);
        d_moonrise.push(time_12h_to_24h_optional(&day.astro.moonrise));
        d_moonset.push(time_12h_to_24h_optional(&day.astro.moonset));
    }

    // Convert alerts
    let alerts: Vec<Alert> = resp
        .alerts
        .map(|a| {
            a.alert
                .into_iter()
                .map(|wa| Alert {
                    headline: wa.headline.unwrap_or_default(),
                    severity: wa.severity.unwrap_or_default(),
                    event: wa.event.unwrap_or_default(),
                    expires: wa.expires.unwrap_or_default(),
                })
                .collect()
        })
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
            visibility_km: resp.current.vis_km,
            dewpoint_c: resp.current.dewpoint_c,
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
            moon_phase: d_moon_phase,
            moon_illumination: d_moon_illum,
            moonrise: d_moonrise,
            moonset: d_moonset,
        },
        alerts,
    };

    let air_quality = resp.current.air_quality.map(|aq| AirQualityResponse {
        current: CurrentAirQuality {
            us_aqi: pm25_to_aqi(aq.pm2_5),
            pm2_5: aq.pm2_5,
            pm10: aq.pm10,
        },
    });

    if imperial {
        let weather = apply_imperial(weather);
        (location, weather, air_quality)
    } else {
        (location, weather, air_quality)
    }
}

fn c_to_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn kph_to_mph(kph: f64) -> f64 {
    kph * 0.621371
}

fn mb_to_inhg(mb: f64) -> f64 {
    mb * 0.02953
}

fn apply_imperial(mut w: WeatherResponse) -> WeatherResponse {
    w.current.temperature_2m = c_to_f(w.current.temperature_2m);
    w.current.apparent_temperature = c_to_f(w.current.apparent_temperature);
    w.current.wind_speed_10m = kph_to_mph(w.current.wind_speed_10m);
    w.current.surface_pressure = mb_to_inhg(w.current.surface_pressure);
    w.current.visibility_km *= 0.621371; // km → miles
    w.current.dewpoint_c = c_to_f(w.current.dewpoint_c);

    w.hourly.temperature_2m = w.hourly.temperature_2m.into_iter().map(c_to_f).collect();

    w.daily.temperature_2m_max = w.daily.temperature_2m_max.into_iter().map(c_to_f).collect();
    w.daily.temperature_2m_min = w.daily.temperature_2m_min.into_iter().map(c_to_f).collect();

    w
}

// Convert WeatherAPI condition code → WMO weather code
fn wapi_to_wmo(code: u32, _is_day: bool) -> u32 {
    match code {
        1000 => 0,                       // Clear
        1003 => 2,                       // Partly cloudy
        1006 => 3,                       // Cloudy
        1009 => 3,                       // Overcast
        1030 | 1135 => 45,               // Mist / Fog
        1147 => 48,                      // Freezing fog
        1150 | 1153 => 51,               // Drizzle
        1168 | 1171 => 56,               // Freezing drizzle
        1180 | 1183 => 61,               // Light rain
        1186 | 1189 => 63,               // Moderate rain
        1192 | 1195 => 65,               // Heavy rain
        1198 | 1204 => 66,               // Freezing rain / sleet
        1201 | 1207 => 67,               // Heavy freezing rain
        1063 | 1240 => 80,               // Rain showers
        1243 => 81,                      // Heavy rain showers
        1246 => 82,                      // Torrential showers
        1066 | 1210 | 1213 => 71,        // Light snow
        1114 | 1216 | 1219 => 73,        // Moderate snow
        1117 | 1222 | 1225 => 75,        // Heavy snow
        1237 | 1261 | 1264 => 77,        // Ice pellets
        1069 | 1249 | 1252 | 1255 => 85, // Snow/sleet showers
        1258 => 86,                      // Heavy snow showers
        1072 | 1087 | 1273 => 95,        // Thunder
        1276 | 1279 | 1282 => 96,        // Heavy thunder
        _ => 3,                          // Fallback: cloudy
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

// Convert "06:30 AM" → "06:30", but return "—" for "No moonrise" etc.
fn time_12h_to_24h_optional(t: &str) -> String {
    if t.starts_with("No ") || t.is_empty() {
        "—".to_string()
    } else {
        time_12h_to_24h(t)
    }
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
