use super::*;

pub fn render(loc: &Location, air: &AirQualityResponse) {
    let cur = &air.current;
    let (r, g, b) = aqi_color(cur.us_aqi);
    let term_w = term_width();

    let title = format!("Air Quality · {}, {}", loc.name, loc.country);

    // AQI bar
    let bar_width = 40usize;
    let aqi_val = cur.us_aqi as u32;
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

    let bar_line = format!(
        "  {} {}",
        bar,
        format!("{:.0}", cur.us_aqi).truecolor(r, g, b).bold()
    );
    let label_line = format!("  {}", aqi_label_str(cur.us_aqi).truecolor(r, g, b));
    let pm25_line = format!(
        "  {}  {:.1} µg/m³",
        "PM2.5".truecolor(180, 180, 190).bold(),
        cur.pm2_5
    );
    let pm10_line = format!(
        "  {}   {:.1} µg/m³",
        "PM10".truecolor(180, 180, 190).bold(),
        cur.pm10
    );

    // Measure widest content for panel width
    let max_content = [
        strip_ansi_len(&bar_line),
        strip_ansi_len(&label_line),
        strip_ansi_len(&pm25_line),
        strip_ansi_len(&pm10_line),
        title.chars().count() + 4,
    ]
    .into_iter()
    .max()
    .unwrap_or(50);
    let w = (max_content + 6).clamp(40, term_w);

    // Render
    println!();

    panel_top(&title, w);
    panel_row("", w);
    panel_row(&bar_line, w);
    panel_row(&label_line, w);
    panel_row("", w);
    panel_row(&pm25_line, w);
    panel_row(&pm10_line, w);
    panel_row("", w);
    panel_bottom(w);

    println!();
}
