use super::*;

pub fn render(loc: &Location, weather: &WeatherResponse) {
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
        let (icon, _, ic) = weather_icon(hourly.weather_code[i], is_day);
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
            format!(" {} ", hour)
                .on_truecolor(60, 60, 80)
                .white()
                .bold()
                .to_string()
        } else {
            format!("  {}  ", hour).dimmed().to_string()
        };

        let colored_icon = icon.truecolor(ic.0, ic.1, ic.2);
        println!(
            "  {}{}  {}{}",
            time_str,
            colored_icon,
            temp_colored(temp),
            rain_str,
        );
    }

    println!();
}
