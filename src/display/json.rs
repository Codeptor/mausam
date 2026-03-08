use crate::types::*;
use serde_json::json;

pub fn render(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let hourly_data: Vec<_> = weather
        .hourly
        .time
        .iter()
        .enumerate()
        .map(|(i, t)| {
            json!({
                "time": t,
                "temperature": weather.hourly.temperature_2m[i],
                "rain_chance": weather.hourly.precipitation_probability[i],
            })
        })
        .collect();

    let daily_data: Vec<_> = weather
        .daily
        .time
        .iter()
        .enumerate()
        .map(|(i, t)| {
            json!({
                "date": t,
                "high": weather.daily.temperature_2m_max[i],
                "low": weather.daily.temperature_2m_min[i],
                "rain_chance": weather.daily.precipitation_probability_max[i],
            })
        })
        .collect();

    let output = json!({
        "location": {
            "name": &loc.name,
            "country": &loc.country,
        },
        "current": {
            "temperature": weather.current.temperature_2m,
            "feels_like": weather.current.apparent_temperature,
            "humidity": weather.current.relative_humidity_2m,
            "wind_speed": weather.current.wind_speed_10m,
            "wind_direction": weather.current.wind_direction_10m,
            "pressure": weather.current.surface_pressure,
            "uv_index": weather.current.uv_index,
            "is_day": weather.current.is_day != 0,
        },
        "hourly": hourly_data,
        "daily": daily_data,
        "air_quality": air.as_ref().map(|a| json!({
            "aqi": a.current.us_aqi,
            "pm2_5": a.current.pm2_5,
            "pm10": a.current.pm10,
        })),
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
