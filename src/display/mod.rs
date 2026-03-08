use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{NaiveDate, Timelike};
use colored::*;

use crate::types::*;

static IMPERIAL: AtomicBool = AtomicBool::new(false);

pub fn set_imperial(imperial: bool) {
    IMPERIAL.store(imperial, Ordering::Relaxed);
}

pub(crate) fn is_imperial() -> bool {
    IMPERIAL.load(Ordering::Relaxed)
}

pub(crate) fn wind_label() -> &'static str {
    if is_imperial() {
        "mph"
    } else {
        "km/h"
    }
}

pub(crate) fn pressure_label() -> &'static str {
    if is_imperial() {
        "inHg"
    } else {
        "hPa"
    }
}

mod aqi;
mod compact;
mod full;
mod hourly;
mod json;

pub use aqi::render as aqi_detail;
pub use compact::render as compact;
pub use full::render as full;
pub use hourly::render as hourly;
pub use json::render as json;

// ─── Nerd Font Icons ──────────────────────────────────

pub(crate) const ICON_SUNNY: &str = "\u{f0599}";
pub(crate) const ICON_CLOUDY: &str = "\u{f0590}";
pub(crate) const ICON_PARTLY: &str = "\u{f0595}";
pub(crate) const ICON_RAINY: &str = "\u{f0597}";
pub(crate) const ICON_SNOWY: &str = "\u{f0598}";
pub(crate) const ICON_THUNDER: &str = "\u{f0593}";
pub(crate) const ICON_FOG: &str = "\u{f0591}";
pub(crate) const ICON_NIGHT: &str = "\u{f0594}";
pub(crate) const ICON_DRIZZLE: &str = "\u{f0596}";
pub(crate) const ICON_WIND: &str = "\u{f059d}";
pub(crate) const ICON_HUMIDITY: &str = "\u{f058e}";
pub(crate) const ICON_SUNRISE: &str = "\u{f059c}";
pub(crate) const ICON_SUNSET: &str = "\u{f059b}";
pub(crate) const ICON_DROP: &str = "\u{f043}";
pub(crate) const ICON_GAUGE: &str = "\u{f0241}";
pub(crate) const ICON_LEAF: &str = "\u{f0312}";

// ─── Color Helpers ────────────────────────────────────

pub(crate) fn temp_to_rgb(temp: f64) -> (u8, u8, u8) {
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

pub(crate) fn temp_colored(temp: f64) -> ColoredString {
    let (r, g, b) = temp_to_rgb(temp);
    format!("{:.0}°", temp).truecolor(r, g, b).bold()
}

pub(crate) fn temp_colored_dim(temp: f64) -> String {
    let (r, g, b) = temp_to_rgb(temp);
    let visible = format!("{:.0}°", temp);
    let colored = visible.truecolor(r, g, b).to_string();
    let width = 4; // enough for "-XX°"
    let vis_len = visible.chars().count();
    if vis_len < width {
        format!("{}{}", " ".repeat(width - vis_len), colored)
    } else {
        colored
    }
}

pub(crate) fn gradient_bar(min: f64, max: f64, abs_min: f64, abs_max: f64, width: usize) -> String {
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

pub(crate) fn colored_sparkline(values: &[f64], spacing: usize) -> String {
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
            let pad = spacing - 1;
            let left = pad / 2;
            let right = pad - left;
            format!("{}{}{}", " ".repeat(left), bar_str, " ".repeat(right))
        })
        .collect()
}

pub(crate) fn center_ansi(colored: &str, visible_len: usize, width: usize) -> String {
    let pad = width.saturating_sub(visible_len);
    let left = pad / 2;
    let right = pad - left;
    format!("{}{}{}", " ".repeat(left), colored, " ".repeat(right))
}

pub(crate) fn weather_icon(code: u32, is_day: bool) -> (&'static str, &'static str) {
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
        80..=82 => (ICON_RAINY, "Showers"),
        85 | 86 => (ICON_SNOWY, "Snow showers"),
        95 => (ICON_THUNDER, "Thunderstorm"),
        96 | 99 => (ICON_THUNDER, "Thunderstorm"),
        _ => (ICON_CLOUDY, "Unknown"),
    }
}

pub(crate) fn wind_arrow(deg: f64) -> &'static str {
    let arrows = ["↓", "↙", "←", "↖", "↑", "↗", "→", "↘"];
    let idx = ((deg + 22.5) / 45.0) as usize % 8;
    arrows[idx]
}

pub(crate) fn uv_label(uv: f64) -> ColoredString {
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

pub(crate) fn aqi_color(aqi: f64) -> (u8, u8, u8) {
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

pub(crate) fn aqi_label(aqi: f64) -> ColoredString {
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

pub(crate) fn format_time(iso: &str) -> String {
    if let Some(t) = iso.split('T').nth(1) {
        t.get(..5).unwrap_or(t).to_string()
    } else {
        iso.to_string()
    }
}

pub(crate) fn day_name(date_str: &str) -> String {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        date.format("%a").to_string()
    } else {
        date_str.to_string()
    }
}

pub(crate) fn current_hour() -> usize {
    chrono::Local::now().hour() as usize
}

pub(crate) fn daylight_str(rise: &str, set: &str) -> String {
    let parse = |t: &str| -> Option<(u32, u32)> {
        let mut parts = t.split(':');
        let h = parts.next()?.parse().ok()?;
        let m = parts.next()?.parse().ok()?;
        Some((h, m))
    };
    if let (Some((rh, rm)), Some((sh, sm))) = (parse(rise), parse(set)) {
        let rise_mins = rh * 60 + rm;
        let set_mins = sh * 60 + sm;
        if set_mins > rise_mins {
            let diff = set_mins - rise_mins;
            let hours = diff / 60;
            let mins = diff % 60;
            return format!("{}h {}m daylight", hours, mins);
        }
    }
    String::new()
}

pub(crate) fn format_hour_human(h: usize) -> String {
    match h {
        0 => "12a".to_string(),
        1..=11 => format!("{}am", h),
        12 => "12p".to_string(),
        _ => format!("{}pm", h - 12),
    }
}

pub(crate) fn tomorrow_comparison(daily: &DailyWeather) -> Option<String> {
    if daily.temperature_2m_max.len() < 2 {
        return None;
    }
    let today_high = daily.temperature_2m_max[0];
    let tomorrow_high = daily.temperature_2m_max[1];
    let diff = (today_high - tomorrow_high).round() as i32;
    if diff.abs() < 2 {
        return None;
    }
    if diff > 0 {
        Some(format!("{}° warmer than tomorrow", diff))
    } else {
        Some(format!("{}° cooler than tomorrow", diff.abs()))
    }
}

pub(crate) fn clothing_hint(feels_like: f64, rain_chance: f64, uv: f64) -> String {
    // Convert to Celsius for threshold checks if imperial
    let c = if is_imperial() {
        (feels_like - 32.0) * 5.0 / 9.0
    } else {
        feels_like
    };
    let base = if c < 0.0 {
        "Bundle up, it's freezing"
    } else if c < 10.0 {
        "Grab a warm jacket"
    } else if c < 18.0 {
        "Light jacket weather"
    } else if c < 25.0 {
        "Comfortable, no layers needed"
    } else if c < 33.0 {
        "Light clothes, stay cool"
    } else if c < 40.0 {
        "Stay hydrated, it's scorching"
    } else {
        "Dangerously hot, limit outdoor exposure"
    };
    let mut hint = base.to_string();
    if rain_chance > 60.0 {
        hint.push_str(" · Carry an umbrella");
    }
    if uv > 7.0 {
        hint.push_str(" · Wear sunscreen");
    }
    hint
}

pub(crate) fn render_alerts(alerts: &[Alert]) {
    for alert in alerts {
        let color = match alert.severity.to_lowercase().as_str() {
            "extreme" => (255, 50, 50),
            "severe" => (255, 140, 0),
            _ => (230, 200, 0),
        };
        println!(
            "   {} {}",
            "!".truecolor(color.0, color.1, color.2).bold(),
            alert.headline.truecolor(color.0, color.1, color.2),
        );
    }
    if !alerts.is_empty() {
        println!();
    }
}

pub(crate) fn divider() {
    println!(
        "  {}",
        "╶─────────────────────────────────────────────────────╴".dimmed()
    );
}

pub(crate) fn rain_indicator(pct: f64) -> String {
    if pct <= 0.0 {
        "    ".to_string()
    } else {
        let icon = ICON_DROP.truecolor(100, 160, 255);
        format!("{} {:>2.0}%", icon, pct)
    }
}
