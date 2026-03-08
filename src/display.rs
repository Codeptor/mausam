use chrono::{NaiveDate, Timelike};
use colored::*;

use crate::types::*;

// ─── Nerd Font Icons ──────────────────────────────────

const ICON_SUNNY: &str = "\u{f0599}";
const ICON_CLOUDY: &str = "\u{f0590}";
const ICON_PARTLY: &str = "\u{f0595}";
const ICON_RAINY: &str = "\u{f0597}";
const ICON_SNOWY: &str = "\u{f0598}";
const ICON_THUNDER: &str = "\u{f0593}";
const ICON_FOG: &str = "\u{f0591}";
const ICON_NIGHT: &str = "\u{f0594}";
const ICON_DRIZZLE: &str = "\u{f0596}";
const ICON_WIND: &str = "\u{f059d}";
const ICON_HUMIDITY: &str = "\u{f058e}";
const ICON_SUNRISE: &str = "\u{f059c}";
const ICON_SUNSET: &str = "\u{f059b}";
const ICON_DROP: &str = "\u{f0210}";
const ICON_GAUGE: &str = "\u{f0241}";
const ICON_LEAF: &str = "\u{f0312}";

// ─── Color Helpers ────────────────────────────────────

fn temp_to_rgb(temp: f64) -> (u8, u8, u8) {
    let stops: [(f64, (f64, f64, f64)); 7] = [
        (-15.0, (70.0, 130.0, 255.0)),
        (0.0, (0.0, 210.0, 255.0)),
        (10.0, (0.0, 220.0, 120.0)),
        (20.0, (180.0, 230.0, 30.0)),
        (28.0, (255.0, 200.0, 0.0)),
        (36.0, (255.0, 130.0, 0.0)),
        (48.0, (255.0, 50.0, 50.0)),
    ];

    let t = temp.clamp(-15.0, 48.0);

    for i in 0..stops.len() - 1 {
        let (t0, c0) = stops[i];
        let (t1, c1) = stops[i + 1];
        if t <= t1 {
            let f = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
            return (
                (c0.0 + (c1.0 - c0.0) * f) as u8,
                (c0.1 + (c1.1 - c0.1) * f) as u8,
                (c0.2 + (c1.2 - c0.2) * f) as u8,
            );
        }
    }
    let c = stops.last().unwrap().1;
    (c.0 as u8, c.1 as u8, c.2 as u8)
}

fn temp_colored(temp: f64) -> ColoredString {
    let (r, g, b) = temp_to_rgb(temp);
    format!("{:.0}°", temp).truecolor(r, g, b).bold()
}

fn temp_colored_dim(temp: f64) -> ColoredString {
    let (r, g, b) = temp_to_rgb(temp);
    format!("{:.0}°", temp).truecolor(r, g, b)
}

fn gradient_bar(min: f64, max: f64, abs_min: f64, abs_max: f64, width: usize) -> String {
    let range = abs_max - abs_min;
    if range == 0.0 {
        return " ".repeat(width);
    }
    let start = ((min - abs_min) / range * width as f64) as usize;
    let end = ((max - abs_min) / range * width as f64) as usize;
    let bar_len = (end - start).max(1);

    let mut result = String::new();
    result.push_str(&"─".dimmed().to_string().repeat(start));

    for j in 0..bar_len {
        let t = min + (max - min) * (j as f64 / bar_len.max(1) as f64);
        let (r, g, b) = temp_to_rgb(t);
        result.push_str(&"━".truecolor(r, g, b).bold().to_string());
    }

    let remaining = width.saturating_sub(start + bar_len);
    result.push_str(&"─".dimmed().to_string().repeat(remaining));
    result
}

fn colored_sparkline(values: &[f64], spacing: usize) -> String {
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
            let idx = if range == 0.0 {
                3
            } else {
                (((v - min) / range) * 7.0) as usize
            };
            let bar = bars[idx.min(7)];
            let (r, g, b) = temp_to_rgb(*v);
            let bar_str = format!("{}", bar).truecolor(r, g, b).bold().to_string();
            format!("{:^width$}", bar_str, width = spacing + bar_str.len() - 1)
        })
        .collect()
}

fn weather_icon(code: u32, is_day: bool) -> (&'static str, &'static str) {
    match code {
        0 => {
            if is_day {
                (ICON_SUNNY, "Clear sky")
            } else {
                (ICON_NIGHT, "Clear night")
            }
        }
        1 => (ICON_PARTLY, "Mainly clear"),
        2 => (ICON_PARTLY, "Partly cloudy"),
        3 => (ICON_CLOUDY, "Overcast"),
        45 | 48 => (ICON_FOG, "Foggy"),
        51 | 53 | 55 => (ICON_DRIZZLE, "Drizzle"),
        56 | 57 => (ICON_DRIZZLE, "Freezing drizzle"),
        61 | 63 | 65 => (ICON_RAINY, "Rain"),
        66 | 67 => (ICON_RAINY, "Freezing rain"),
        71 | 73 | 75 => (ICON_SNOWY, "Snowfall"),
        77 => (ICON_SNOWY, "Snow grains"),
        80 | 81 | 82 => (ICON_RAINY, "Showers"),
        85 | 86 => (ICON_SNOWY, "Snow showers"),
        95 => (ICON_THUNDER, "Thunderstorm"),
        96 | 99 => (ICON_THUNDER, "Thunderstorm"),
        _ => (ICON_CLOUDY, "Unknown"),
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

fn aqi_color(aqi: f64) -> (u8, u8, u8) {
    let v = aqi as u32;
    if v <= 50 {
        (0, 200, 80)
    } else if v <= 100 {
        (230, 200, 0)
    } else if v <= 150 {
        (255, 140, 0)
    } else if v <= 200 {
        (220, 50, 50)
    } else if v <= 300 {
        (160, 0, 160)
    } else {
        (128, 0, 0)
    }
}

fn aqi_label(aqi: f64) -> ColoredString {
    let (r, g, b) = aqi_color(aqi);
    let v = aqi as u32;
    let label = if v <= 50 {
        "Good"
    } else if v <= 100 {
        "Moderate"
    } else if v <= 150 {
        "Sensitive"
    } else if v <= 200 {
        "Unhealthy"
    } else if v <= 300 {
        "Very Unhealthy"
    } else {
        "Hazardous"
    };
    label.truecolor(r, g, b)
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

fn current_hour() -> usize {
    chrono::Local::now().hour() as usize
}

fn divider() {
    println!(
        "  {}",
        "╶─────────────────────────────────────────────────────╴".dimmed()
    );
}

fn rain_indicator(pct: f64) -> String {
    if pct <= 0.0 {
        "    ".to_string()
    } else {
        let icon = ICON_DROP.truecolor(100, 160, 255);
        format!("{}{:>2.0}%", icon, pct)
    }
}

// ─── Compact View ─────────────────────────────────────

pub fn compact(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let (icon, desc) = weather_icon(cur.weather_code, cur.is_day != 0);

    println!();

    // Main temperature + location
    println!(
        "   {}  {}                              {}",
        icon.truecolor(255, 200, 80),
        temp_colored(cur.temperature_2m),
        format!("{}, {}", loc.name, loc.country).bold()
    );
    println!(
        "      {} {} {}",
        desc.white(),
        "·".dimmed(),
        format!("Feels like {:.0}°", cur.apparent_temperature).dimmed()
    );

    println!();
    divider();
    println!();

    // Metrics
    println!(
        "   {} {:<11} {} {:<10} {} {} {}",
        ICON_WIND.truecolor(150, 180, 210),
        format!("{:.0} km/h {}", cur.wind_speed_10m, wind_arrow(cur.wind_direction_10m)),
        ICON_HUMIDITY.truecolor(80, 170, 255),
        format!("{:.0}%", cur.relative_humidity_2m),
        "UV".dimmed(),
        format!("{:.0}", cur.uv_index).bold(),
        uv_label(cur.uv_index),
    );

    if let Some(air) = air {
        let (r, g, b) = aqi_color(air.current.us_aqi);
        println!(
            "   {} {} {} {}",
            ICON_LEAF.truecolor(r, g, b),
            format!("AQI {:.0}", air.current.us_aqi).truecolor(r, g, b).bold(),
            aqi_label(air.current.us_aqi),
            format!("· PM2.5 {:.0} · PM10 {:.0}", air.current.pm2_5, air.current.pm10).dimmed(),
        );
    }

    println!();
    divider();
    println!();

    // 3-day forecast
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
        let bar = gradient_bar(
            daily.temperature_2m_min[i],
            daily.temperature_2m_max[i],
            abs_min,
            abs_max,
            22,
        );
        let rain = rain_indicator(daily.precipitation_probability_max[i]);
        println!(
            "   {}  {}  {}  {}  {}  {}",
            day_name(&daily.time[i]).dimmed(),
            d_icon,
            temp_colored_dim(daily.temperature_2m_min[i]),
            bar,
            temp_colored_dim(daily.temperature_2m_max[i]),
            rain,
        );
    }

    // Sunrise/sunset
    if !daily.sunrise.is_empty() {
        println!();
        println!(
            "   {} {}     {} {}",
            ICON_SUNRISE.truecolor(255, 180, 50),
            format_time(&daily.sunrise[0]).dimmed(),
            ICON_SUNSET.truecolor(255, 100, 50),
            format_time(&daily.sunset[0]).dimmed(),
        );
    }

    println!();
}

// ─── Full Dashboard ───────────────────────────────────

pub fn full(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let hourly = &weather.hourly;
    let (icon, desc) = weather_icon(cur.weather_code, cur.is_day != 0);

    println!();

    // Main temperature + location
    println!(
        "   {}  {}                              {}",
        icon.truecolor(255, 200, 80),
        temp_colored(cur.temperature_2m),
        format!("{}, {}", loc.name, loc.country).bold()
    );
    println!(
        "      {} {} {}",
        desc.white(),
        "·".dimmed(),
        format!("Feels like {:.0}°", cur.apparent_temperature).dimmed()
    );

    println!();
    divider();
    println!();

    // Metrics - two rows
    println!(
        "   {} {:<11} {} {:<10} {} {} {}",
        ICON_WIND.truecolor(150, 180, 210),
        format!("{:.0} km/h {}", cur.wind_speed_10m, wind_arrow(cur.wind_direction_10m)),
        ICON_HUMIDITY.truecolor(80, 170, 255),
        format!("{:.0}%", cur.relative_humidity_2m),
        "UV".dimmed(),
        format!("{:.0}", cur.uv_index).bold(),
        uv_label(cur.uv_index),
    );

    println!(
        "   {} {:<11}",
        ICON_GAUGE.truecolor(150, 150, 170),
        format!("{:.0} hPa", cur.surface_pressure),
    );

    if let Some(air) = air {
        let (r, g, b) = aqi_color(air.current.us_aqi);
        println!(
            "   {} {} {} {}",
            ICON_LEAF.truecolor(r, g, b),
            format!("AQI {:.0}", air.current.us_aqi).truecolor(r, g, b).bold(),
            aqi_label(air.current.us_aqi),
            format!("· PM2.5 {:.0} µg/m³ · PM10 {:.0} µg/m³", air.current.pm2_5, air.current.pm10)
                .dimmed(),
        );
    }

    // Hourly sparkline
    println!();
    divider();
    println!();
    println!("   {}", "Hourly".bold());
    println!();

    let now = current_hour();
    let step = 3;
    let count = 8;

    let mut hours: Vec<String> = Vec::new();
    let mut temps: Vec<f64> = Vec::new();
    let mut rains: Vec<f64> = Vec::new();

    for j in 0..count {
        let idx = now + j * step;
        if idx < hourly.time.len() {
            hours.push(
                format_time(&hourly.time[idx])
                    .get(..2)
                    .unwrap_or("??")
                    .to_string(),
            );
            temps.push(hourly.temperature_2m[idx]);
            rains.push(hourly.precipitation_probability[idx]);
        }
    }

    let hour_str: String = hours.iter().map(|h| format!("{:^5}", h)).collect();
    let spark_str = colored_sparkline(&temps, 5);
    let temp_str: String = temps
        .iter()
        .map(|t| {
            let (r, g, b) = temp_to_rgb(*t);
            let ts = format!("{:.0}°", t).truecolor(r, g, b).to_string();
            // pad accounting for ANSI codes
            let visible_len = format!("{:.0}°", t).len();
            let total_pad = 5usize.saturating_sub(visible_len);
            let left = total_pad / 2;
            let right = total_pad - left;
            format!("{}{}{}", " ".repeat(left), ts, " ".repeat(right))
        })
        .collect::<String>();
    let rain_str: String = rains
        .iter()
        .map(|r| {
            if *r > 30.0 {
                let s = format!("{:.0}%", r)
                    .truecolor(100, 160, 255)
                    .to_string();
                let vis = format!("{:.0}%", r).len();
                let pad = 5usize.saturating_sub(vis);
                let l = pad / 2;
                let ri = pad - l;
                format!("{}{}{}", " ".repeat(l), s, " ".repeat(ri))
            } else if *r > 0.0 {
                let s = format!("{:.0}%", r).dimmed().to_string();
                let vis = format!("{:.0}%", r).len();
                let pad = 5usize.saturating_sub(vis);
                let l = pad / 2;
                let ri = pad - l;
                format!("{}{}{}", " ".repeat(l), s, " ".repeat(ri))
            } else {
                "  ·  ".dimmed().to_string()
            }
        })
        .collect::<String>();

    println!("   {}", hour_str.dimmed());
    println!("   {}", spark_str);
    println!("   {}", temp_str);
    println!("   {}", rain_str);

    // 7-day forecast
    println!();
    divider();
    println!();
    println!("   {}", "7-Day".bold());
    println!();

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
        let bar = gradient_bar(
            daily.temperature_2m_min[i],
            daily.temperature_2m_max[i],
            abs_min,
            abs_max,
            20,
        );
        let rain = rain_indicator(daily.precipitation_probability_max[i]);
        println!(
            "   {}  {}  {}  {}  {}  {}",
            day_name(&daily.time[i]).dimmed(),
            d_icon,
            temp_colored_dim(daily.temperature_2m_min[i]),
            bar,
            temp_colored_dim(daily.temperature_2m_max[i]),
            rain,
        );
    }

    // Sunrise/sunset
    if !daily.sunrise.is_empty() {
        println!();
        println!(
            "   {} {}     {} {}",
            ICON_SUNRISE.truecolor(255, 180, 50),
            format_time(&daily.sunrise[0]).dimmed(),
            ICON_SUNSET.truecolor(255, 100, 50),
            format_time(&daily.sunset[0]).dimmed(),
        );
    }

    println!();
}

// ─── Hourly View ──────────────────────────────────────

pub fn hourly(loc: &Location, weather: &WeatherResponse) {
    let hourly = &weather.hourly;
    let now = current_hour();

    println!();
    println!(
        "   {} {} {}",
        "Hourly".bold(),
        "·".dimmed(),
        format!("{}, {}", loc.name, loc.country).dimmed()
    );
    println!();

    let end = (now + 24).min(hourly.time.len());
    for i in now..end {
        let hour = format_time(&hourly.time[i]);
        let is_day = (i % 24) >= 6 && (i % 24) < 19;
        let (icon, _) = weather_icon(hourly.weather_code[i], is_day);
        let temp = hourly.temperature_2m[i];
        let rain = hourly.precipitation_probability[i];

        let rain_str = if rain > 30.0 {
            format!("  {} {:>2.0}%", ICON_DROP.truecolor(100, 160, 255), rain)
                .truecolor(100, 160, 255)
                .to_string()
        } else if rain > 0.0 {
            format!("  {:>3.0}%", rain).dimmed().to_string()
        } else {
            String::new()
        };

        let is_now = i == now;
        let time_str = if is_now {
            format!(" {} ", hour).on_truecolor(60, 60, 80).white().bold().to_string()
        } else {
            format!("  {}  ", hour).dimmed().to_string()
        };

        println!(
            "  {}{}  {}{}",
            time_str,
            icon,
            temp_colored(temp),
            rain_str,
        );
    }

    println!();
}

// ─── AQI Detail View ─────────────────────────────────

pub fn aqi_detail(loc: &Location, air: &AirQualityResponse) {
    let cur = &air.current;
    let (r, g, b) = aqi_color(cur.us_aqi);

    println!();
    println!(
        "   {} {} {} {}",
        ICON_LEAF.truecolor(r, g, b),
        "Air Quality".bold(),
        "·".dimmed(),
        format!("{}, {}", loc.name, loc.country).dimmed()
    );

    println!();

    // AQI bar
    let aqi_val = cur.us_aqi as u32;
    let bar_width = 40;
    let filled = ((aqi_val as f64 / 300.0) * bar_width as f64).min(bar_width as f64) as usize;
    let bar: String = (0..bar_width)
        .map(|i| {
            let aqi_at = (i as f64 / bar_width as f64) * 300.0;
            let (cr, cg, cb) = aqi_color(aqi_at);
            if i < filled {
                "█".truecolor(cr, cg, cb).to_string()
            } else {
                "░".truecolor(60, 60, 60).to_string()
            }
        })
        .collect();

    println!("   {} {}", bar, format!("{:.0}", cur.us_aqi).truecolor(r, g, b).bold());
    println!("   {}", aqi_label(cur.us_aqi));

    println!();

    println!(
        "   {}  {:.1} µg/m³",
        "PM2.5".truecolor(180, 180, 190).bold(),
        cur.pm2_5
    );
    println!(
        "   {}   {:.1} µg/m³",
        "PM10".truecolor(180, 180, 190).bold(),
        cur.pm10
    );

    println!();
}
