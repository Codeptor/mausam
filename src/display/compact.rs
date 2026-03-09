use super::*;

pub fn render(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let (icon, desc) = weather_icon(cur.weather_code, is_daytime_now(daily));

    println!();
    render_alerts(&weather.alerts);

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

    // Metrics — 2×3 grid
    let c1 = 18usize;
    let c2 = 14usize;

    let wind_text = format!(
        "{:.0} {} {}",
        cur.wind_speed_10m,
        wind_label(),
        wind_compass(cur.wind_direction_10m),
    );
    let hum_text = format!("{:.0}%", cur.relative_humidity_2m);
    let uv_text = format!("UV {:.0} {}", cur.uv_index, uv_label_str(cur.uv_index));
    println!(
        "   {} {:<c1$} {} {:<c2$} {}",
        ICON_WIND.truecolor(150, 180, 210),
        wind_text,
        ICON_HUMIDITY.truecolor(80, 170, 255),
        hum_text,
        uv_text,
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

    // Sunrise/sunset + daylight
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
    }

    println!();
}
