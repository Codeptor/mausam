use super::*;

pub fn render(loc: &Location, weather: &WeatherResponse) {
    let hourly = &weather.hourly;
    let daily = &weather.daily;
    let now = current_hour();
    let term_w = term_width();

    let rise_mins = daily
        .sunrise
        .first()
        .map(|s| parse_time_mins(&format_time(s)));
    let set_mins = daily
        .sunset
        .first()
        .map(|s| parse_time_mins(&format_time(s)));

    let title = format!("Hourly · {}, {}", loc.name, loc.country);

    // Pre-compute rows to measure max width
    let end = (now + 24).min(hourly.time.len());
    let mut rows: Vec<(String, usize)> = Vec::new();

    for i in now..end {
        let hour = format_time(&hourly.time[i]);
        let is_day = match (rise_mins, set_mins, parse_time_mins(&hour)) {
            (Some(Some(r)), Some(Some(s)), Some(h)) => h >= r && h < s,
            _ => (i % 24) >= 6 && (i % 24) < 19,
        };
        let (ic, _, ic_color) = weather_icon(hourly.weather_code[i], is_day);
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
            format!("  {}  ", hour)
                .on_truecolor(60, 60, 80)
                .white()
                .bold()
                .to_string()
        } else {
            format!("  {}  ", hour).dimmed().to_string()
        };

        let colored_icon = ic.truecolor(ic_color.0, ic_color.1, ic_color.2);
        let row = format!(
            "{}{}  {}{}",
            time_str,
            colored_icon,
            temp_colored(temp),
            rain_str
        );
        let vis_len = strip_ansi_len(&row);
        rows.push((row, vis_len));
    }

    let max_row_width = rows.iter().map(|(_, len)| *len).max().unwrap_or(40);
    let w = (max_row_width + 6).clamp(40, term_w);

    // Render
    println!();
    render_alerts(&weather.alerts);

    panel_top(&title, w);
    for (row, _) in &rows {
        panel_row(row, w);
    }
    panel_bottom(w);

    println!();
}
