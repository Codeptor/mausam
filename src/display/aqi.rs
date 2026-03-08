use super::*;

pub fn render(loc: &Location, air: &AirQualityResponse) {
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

    println!(
        "   {} {}",
        bar,
        format!("{:.0}", cur.us_aqi).truecolor(r, g, b).bold()
    );
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
