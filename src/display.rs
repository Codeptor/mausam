use chrono::{NaiveDate, Timelike};
use colored::*;

use crate::types::*;

fn weather_icon(code: u32, is_day: bool) -> (&'static str, &'static str) {
    match code {
        0 => {
            if is_day {
                ("☀", "Clear sky")
            } else {
                ("🌙", "Clear sky")
            }
        }
        1 => ("⛅", "Mainly clear"),
        2 => ("⛅", "Partly cloudy"),
        3 => ("☁", "Overcast"),
        45 | 48 => ("🌫", "Foggy"),
        51 | 53 | 55 => ("🌦", "Drizzle"),
        56 | 57 => ("🌧", "Freezing drizzle"),
        61 | 63 | 65 => ("🌧", "Rain"),
        66 | 67 => ("🌧", "Freezing rain"),
        71 | 73 | 75 => ("❄", "Snowfall"),
        77 => ("❄", "Snow grains"),
        80 | 81 | 82 => ("🌧", "Rain showers"),
        85 | 86 => ("❄", "Snow showers"),
        95 => ("⛈", "Thunderstorm"),
        96 | 99 => ("⛈", "Thunderstorm w/ hail"),
        _ => ("?", "Unknown"),
    }
}

fn wind_arrow(deg: f64) -> &'static str {
    let arrows = ["↓", "↙", "←", "↖", "↑", "↗", "→", "↘"];
    let idx = ((deg + 22.5) / 45.0) as usize % 8;
    arrows[idx]
}

fn uv_label(uv: f64) -> ColoredString {
    let v = uv as u32;
    if v <= 2 {
        "Low".green()
    } else if v <= 5 {
        "Moderate".yellow()
    } else if v <= 7 {
        "High".truecolor(255, 165, 0)
    } else if v <= 10 {
        "Very High".red()
    } else {
        "Extreme".magenta()
    }
}

fn aqi_label(aqi: f64) -> ColoredString {
    let v = aqi as u32;
    if v <= 50 {
        "Good".green()
    } else if v <= 100 {
        "Moderate".yellow()
    } else if v <= 150 {
        "Unhealthy (Sensitive)".truecolor(255, 165, 0)
    } else if v <= 200 {
        "Unhealthy".red()
    } else if v <= 300 {
        "Very Unhealthy".truecolor(128, 0, 128)
    } else {
        "Hazardous".truecolor(128, 0, 0)
    }
}

fn temp_color(temp: f64) -> ColoredString {
    let s = format!("{:.0}°", temp);
    if temp < 0.0 {
        s.cyan()
    } else if temp <= 10.0 {
        s.blue()
    } else if temp <= 20.0 {
        s.green()
    } else if temp <= 30.0 {
        s.yellow()
    } else if temp <= 40.0 {
        s.truecolor(255, 165, 0)
    } else {
        s.red()
    }
}

fn sparkline(values: &[f64]) -> String {
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() {
        return String::new();
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    values
        .iter()
        .map(|v| {
            if range == 0.0 {
                bars[3]
            } else {
                let idx = (((v - min) / range) * 7.0) as usize;
                bars[idx.min(7)]
            }
        })
        .collect()
}

fn temp_bar(min: f64, max: f64, abs_min: f64, abs_max: f64, width: usize) -> ColoredString {
    let range = abs_max - abs_min;
    if range == 0.0 {
        return "━".repeat(width).dimmed();
    }
    let start = ((min - abs_min) / range * width as f64) as usize;
    let end = ((max - abs_min) / range * width as f64) as usize;
    let bar_len = (end - start).max(1);
    format!(
        "{}{}{}",
        " ".repeat(start),
        "━".repeat(bar_len),
        " ".repeat(width.saturating_sub(start + bar_len))
    )
    .dimmed()
}

fn format_time(iso: &str) -> String {
    if let Some(t) = iso.split('T').nth(1) {
        t.get(..5).unwrap_or(t).to_string()
    } else {
        iso.to_string()
    }
}

fn day_name(date_str: &str) -> String {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        date.format("%a").to_string()
    } else {
        date_str.to_string()
    }
}

fn section_header(title: &str) {
    println!("  ─── {} {}", title.bold(), "─".repeat(46 - title.len()));
}

fn current_hour() -> usize {
    chrono::Local::now().hour() as usize
}

// ─────────────────────────────────────────────────────────
// Compact view (default)
// ─────────────────────────────────────────────────────────

pub fn compact(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let (icon, desc) = weather_icon(cur.weather_code, cur.is_day != 0);

    println!();
    println!(
        "  {}  {}, {}",
        icon,
        loc.name.bold(),
        loc.country.dimmed()
    );
    println!(
        "     {}C (feels {}C) · {}",
        temp_color(cur.temperature_2m),
        temp_color(cur.apparent_temperature),
        desc.dimmed(),
    );
    println!(
        "     Wind: {:.0} km/h {} · Humidity: {:.0}%",
        cur.wind_speed_10m,
        wind_arrow(cur.wind_direction_10m),
        cur.relative_humidity_2m,
    );

    if let Some(air) = air {
        println!(
            "     UV: {:.0} {} · AQI: {:.0} {}",
            cur.uv_index,
            uv_label(cur.uv_index),
            air.current.us_aqi,
            aqi_label(air.current.us_aqi),
        );
    } else {
        println!("     UV: {:.0} {}", cur.uv_index, uv_label(cur.uv_index));
    }

    println!();
    section_header("3-Day Forecast");

    let days = 3.min(daily.time.len());
    let abs_min = daily.temperature_2m_min[..days]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let abs_max = daily.temperature_2m_max[..days]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    for i in 0..days {
        let (d_icon, _) = weather_icon(daily.weather_code[i], true);
        let bar = temp_bar(
            daily.temperature_2m_min[i],
            daily.temperature_2m_max[i],
            abs_min,
            abs_max,
            20,
        );
        println!(
            "  {}  {}  {} {} {} {:>3.0}%",
            day_name(&daily.time[i]).dimmed(),
            d_icon,
            temp_color(daily.temperature_2m_min[i]),
            bar,
            temp_color(daily.temperature_2m_max[i]),
            daily.precipitation_probability_max[i],
        );
    }

    if !daily.sunrise.is_empty() {
        println!();
        println!(
            "  🌅 {}  🌇 {}",
            format_time(&daily.sunrise[0]),
            format_time(&daily.sunset[0]),
        );
    }

    println!();
}

// ─────────────────────────────────────────────────────────
// Full dashboard (-f)
// ─────────────────────────────────────────────────────────

pub fn full(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let hourly = &weather.hourly;
    let (icon, desc) = weather_icon(cur.weather_code, cur.is_day != 0);

    println!();
    println!(
        "  {}  {}, {}",
        icon,
        loc.name.bold(),
        loc.country.dimmed()
    );
    println!(
        "     {}C (feels {}C) · {}",
        temp_color(cur.temperature_2m),
        temp_color(cur.apparent_temperature),
        desc,
    );
    println!(
        "     Wind: {:.0} km/h {} · Humidity: {:.0}% · {:.0} hPa",
        cur.wind_speed_10m,
        wind_arrow(cur.wind_direction_10m),
        cur.relative_humidity_2m,
        cur.surface_pressure,
    );

    if let Some(air) = air {
        println!(
            "     UV: {:.0} {} · AQI: {:.0} {}",
            cur.uv_index,
            uv_label(cur.uv_index),
            air.current.us_aqi,
            aqi_label(air.current.us_aqi),
        );
        println!(
            "     PM2.5: {:.1} µg/m³ · PM10: {:.1} µg/m³",
            air.current.pm2_5, air.current.pm10,
        );
    } else {
        println!("     UV: {:.0} {}", cur.uv_index, uv_label(cur.uv_index));
    }

    // Hourly sparkline
    println!();
    section_header("Hourly");

    let now = current_hour();
    let step = 3;
    let count = 8;

    let mut hours: Vec<String> = Vec::new();
    let mut temps: Vec<f64> = Vec::new();
    let mut rains: Vec<f64> = Vec::new();

    for j in 0..count {
        let idx = now + j * step;
        if idx < hourly.time.len() {
            hours.push(format_time(&hourly.time[idx]).get(..2).unwrap_or("??").to_string());
            temps.push(hourly.temperature_2m[idx]);
            rains.push(hourly.precipitation_probability[idx]);
        }
    }

    let hour_str: String = hours.iter().map(|h| format!("{:>5}", h)).collect();
    let spark = sparkline(&temps);
    let spark_str: String = spark.chars().map(|c| format!("{:^5}", c)).collect();
    let temp_str: String = temps.iter().map(|t| format!("{:>4.0}°", t)).collect();
    let rain_str: String = rains
        .iter()
        .map(|r| {
            if *r > 50.0 {
                format!("{:>4.0}%", r)
            } else if *r > 0.0 {
                format!("{:>4.0}%", r).dimmed().to_string()
            } else {
                "   · ".to_string()
            }
        })
        .collect();

    println!("  {}", hour_str.dimmed());
    println!("  {}", spark_str);
    println!("  {}", temp_str);
    println!("  {}", rain_str);

    // 7-day forecast
    println!();
    section_header("7-Day Forecast");

    let days = 7.min(daily.time.len());
    let abs_min = daily.temperature_2m_min[..days]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let abs_max = daily.temperature_2m_max[..days]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    for i in 0..days {
        let (d_icon, _) = weather_icon(daily.weather_code[i], true);
        let bar = temp_bar(
            daily.temperature_2m_min[i],
            daily.temperature_2m_max[i],
            abs_min,
            abs_max,
            18,
        );
        println!(
            "  {}  {}  {} {} {} {:>3.0}%",
            day_name(&daily.time[i]).dimmed(),
            d_icon,
            temp_color(daily.temperature_2m_min[i]),
            bar,
            temp_color(daily.temperature_2m_max[i]),
            daily.precipitation_probability_max[i],
        );
    }

    // Sunrise/sunset
    if !daily.sunrise.is_empty() {
        println!();
        println!(
            "  🌅 {}  🌇 {}",
            format_time(&daily.sunrise[0]),
            format_time(&daily.sunset[0]),
        );
    }

    println!();
}

// ─────────────────────────────────────────────────────────
// Hourly view (-H)
// ─────────────────────────────────────────────────────────

pub fn hourly(loc: &Location, weather: &WeatherResponse) {
    let hourly = &weather.hourly;
    let now = current_hour();

    println!();
    println!(
        "  {} · {}, {}",
        "Hourly Forecast".bold(),
        loc.name,
        loc.country.dimmed()
    );
    println!();

    let end = (now + 24).min(hourly.time.len());
    for i in now..end {
        let hour = format_time(&hourly.time[i]);
        let is_day = (i % 24) >= 6 && (i % 24) < 19;
        let (icon, _) = weather_icon(hourly.weather_code[i], is_day);
        let temp = hourly.temperature_2m[i];
        let rain = hourly.precipitation_probability[i];

        let rain_str = if rain > 0.0 {
            format!("  {:>3.0}%", rain).dimmed().to_string()
        } else {
            String::new()
        };

        println!("  {}  {}  {}C{}", hour.dimmed(), icon, temp_color(temp), rain_str);
    }

    println!();
}

// ─────────────────────────────────────────────────────────
// AQI detail view (-a)
// ─────────────────────────────────────────────────────────

pub fn aqi_detail(loc: &Location, air: &AirQualityResponse) {
    let cur = &air.current;

    println!();
    println!(
        "  {} · {}, {}",
        "Air Quality".bold(),
        loc.name,
        loc.country.dimmed()
    );
    println!();
    println!("  AQI:   {:.0} {}", cur.us_aqi, aqi_label(cur.us_aqi));
    println!("  PM2.5: {:.1} µg/m³", cur.pm2_5);
    println!("  PM10:  {:.1} µg/m³", cur.pm10);
    println!();
}
