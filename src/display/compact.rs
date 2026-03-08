use super::*;

pub fn render(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let (icon, desc) = weather_icon(cur.weather_code, cur.is_day != 0);

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
