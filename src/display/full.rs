use super::*;

pub fn render(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let hourly = &weather.hourly;
    let (icon, desc, icon_color) = weather_icon(cur.weather_code, is_daytime_now(daily));

    println!();
    render_alerts(&weather.alerts);

    // Header
    println!(
        "   {}  {}  {}",
        icon.truecolor(icon_color.0, icon_color.1, icon_color.2),
        temp_colored(cur.temperature_2m),
        format!("{}, {}", loc.name, loc.country).bold()
    );
    println!(
        "      {} {} {}",
        desc.white(),
        "·".dimmed(),
        format!("{} Feels like {:.0}°", temp_icon(cur.apparent_temperature), cur.apparent_temperature).dimmed()
    );
    let today_rain = daily
        .precipitation_probability_max
        .first()
        .copied()
        .unwrap_or(0.0);
    println!(
        "      {}",
        clothing_hint(cur.apparent_temperature, today_rain, cur.uv_index).dimmed()
    );
    if let Some(cmp) = tomorrow_comparison(daily) {
        println!("      {}", cmp.dimmed());
    }

    println!();
    divider();
    println!();

    // Metrics — 3×3 grid (pad plain text, then color)
    let c1 = 18usize; // column 1 width
    let c2 = 14usize; // column 2 width

    let wind_text = format!(
        "{:.0} {} {}",
        cur.wind_speed_10m,
        wind_label(),
        wind_compass(cur.wind_direction_10m),
    );
    let hum_text = format!("{:.0}%", cur.relative_humidity_2m);
    let uv_text = format!("UV {:.0} {}", cur.uv_index, uv_label_str(cur.uv_index));
    let pressure_text = if is_imperial() {
        format!("{:.2} {}", cur.surface_pressure, pressure_label())
    } else {
        format!("{:.0} {}", cur.surface_pressure, pressure_label())
    };
    let vis_text = format!("{:.0} {}", cur.visibility_km, visibility_label());
    let dew_text = format!("{:.0}° dew", cur.dewpoint_c);

    println!(
        "   {} {:<c1$} {} {:<c2$} {}",
        ICON_WIND.truecolor(150, 180, 210),
        wind_text,
        ICON_HUMIDITY.truecolor(80, 170, 255),
        hum_text,
        uv_text,
    );
    println!(
        "   {} {:<c1$} {} {:<c2$} {}",
        ICON_GAUGE.truecolor(150, 150, 170),
        pressure_text,
        ICON_EYE.truecolor(180, 180, 200),
        vis_text,
        dew_text.dimmed(),
    );

    if let Some(air) = air {
        let (r, g, b) = aqi_color(air.current.us_aqi);
        let aqi_padded = format!(
            "{:<c1$}",
            format!(
                "AQI {:.0} {}",
                air.current.us_aqi,
                aqi_label_str(air.current.us_aqi)
            )
        );
        let pm25_padded = format!("{:<c2$}", format!("PM2.5 {:.0}", air.current.pm2_5));
        println!(
            "   {} {} {} {} {}",
            ICON_LEAF.truecolor(r, g, b),
            aqi_padded.truecolor(r, g, b),
            ICON_EYE.truecolor(r, g, b),
            pm25_padded.dimmed(),
            format!("PM10 {:.0}", air.current.pm10).dimmed(),
        );
    }

    // Hourly sparkline
    println!();
    divider();
    println!();
    println!("   {}", "Next 24 Hours".bold());
    println!();

    let w = term_width();
    let now = current_hour();
    let step = 3;
    // Fit columns to terminal: each col is 6 chars + 3 prefix padding
    let count = ((w.saturating_sub(3)) / 6).clamp(3, 8);

    let mut hours: Vec<String> = Vec::new();
    let mut temps: Vec<f64> = Vec::new();
    let mut rains: Vec<f64> = Vec::new();

    for j in 0..count {
        let idx = now + j * step;
        if idx < hourly.time.len() {
            if j == 0 {
                hours.push("Now".to_string());
            } else {
                let h = (now + j * step) % 24;
                hours.push(format_hour_human(h));
            }
            temps.push(hourly.temperature_2m[idx]);
            rains.push(hourly.precipitation_probability[idx]);
        }
    }

    let col = 6;
    let hour_str: String = hours.iter().map(|h| format!("{:^col$}", h)).collect();
    let spark_str = colored_sparkline(&temps, col);
    let temp_str: String = temps
        .iter()
        .map(|t| {
            let (r, g, b) = temp_to_rgb(*t);
            let visible = format!("{:.0}°", t);
            let colored = visible.truecolor(r, g, b).to_string();
            center_ansi(&colored, visible.chars().count(), col)
        })
        .collect::<String>();

    println!("   {}", hour_str.dimmed());
    println!("   {}", spark_str);
    println!("   {}", temp_str);

    // Only show rain row if there's meaningful rain
    let has_rain = rains.iter().any(|r| *r > 0.0);
    if has_rain {
        let rain_str: String = rains
            .iter()
            .map(|r| {
                if *r > 30.0 {
                    let visible = format!("{:.0}%", r);
                    let colored = visible.truecolor(100, 160, 255).to_string();
                    center_ansi(&colored, visible.chars().count(), col)
                } else if *r > 0.0 {
                    let visible = format!("{:.0}%", r);
                    let colored = visible.dimmed().to_string();
                    center_ansi(&colored, visible.chars().count(), col)
                } else {
                    " ".repeat(col)
                }
            })
            .collect::<String>();
        println!("   {}", rain_str);
    }

    // 7-day forecast
    println!();
    divider();
    println!();
    println!("   {}", "7-Day".bold());
    println!();

    let bar_width = if w >= 60 { w - 34 } else { 10 };
    let show_rain = w >= 50;

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
        let (d_icon, _, d_color) = weather_icon(daily.weather_code[i], true);
        let bar = gradient_bar(
            daily.temperature_2m_min[i],
            daily.temperature_2m_max[i],
            abs_min,
            abs_max,
            bar_width,
        );
        let rain = if show_rain {
            rain_indicator(daily.precipitation_probability_max[i])
        } else {
            String::new()
        };
        println!(
            "   {}  {} {} {} {} {}",
            day_name(&daily.time[i]).dimmed(),
            d_icon.truecolor(d_color.0, d_color.1, d_color.2),
            temp_colored_dim(daily.temperature_2m_min[i]),
            bar,
            temp_colored_dim(daily.temperature_2m_max[i]),
            rain,
        );
    }

    // Sunrise/sunset + moon
    if !daily.sunrise.is_empty() {
        let rise = format_time(&daily.sunrise[0]);
        let set = format_time(&daily.sunset[0]);
        let daylight = daylight_str(&rise, &set);

        println!();
        println!(
            "   {} {}     {} {}     {}",
            ICON_SUNRISE.truecolor(255, 180, 50),
            rise.dimmed(),
            ICON_SUNSET.truecolor(255, 100, 50),
            set.dimmed(),
            daylight.dimmed(),
        );

        if let Some(phase) = daily.moon_phase.first() {
            let illum = daily.moon_illumination.first().copied().unwrap_or(0);
            let mrise = daily.moonrise.first().map(|s| s.as_str()).unwrap_or("—");
            let mset = daily.moonset.first().map(|s| s.as_str()).unwrap_or("—");
            println!(
                "   {} {} {}     {} {}",
                moon_icon(phase).truecolor(200, 200, 220),
                format!("{phase} {illum}%").dimmed(),
                "".dimmed(),
                format!("\u{e3c1} {mrise}").dimmed(),
                format!("\u{e3c2} {mset}").dimmed(),
            );
        }
    }

    println!();
}
