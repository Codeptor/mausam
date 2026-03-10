use super::*;

pub fn render(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let cur = &weather.current;
    let daily = &weather.daily;
    let hourly = &weather.hourly;
    let (icon, desc, icon_color) = weather_icon(cur.weather_code, is_daytime_now(daily));
    let term_w = term_width();

    // ─── Phase 1: Pre-compute content & derive panel width ───

    let title = format!("{}, {}", loc.name, loc.country);
    let (tr, tg, tb) = temp_to_rgb(cur.temperature_2m);
    let temp_str = format!("{:.0}°", cur.temperature_2m);
    let today_rain = daily
        .precipitation_probability_max
        .first()
        .copied()
        .unwrap_or(0.0);
    let hint = clothing_hint(cur.apparent_temperature, today_rain, cur.uv_index);

    // Conditions content
    let wind_val = format!(
        "{} {:.0} {} {}",
        ICON_WIND.truecolor(150, 180, 210),
        cur.wind_speed_10m,
        wind_label(),
        wind_compass(cur.wind_direction_10m),
    );
    let hum_val = format!(
        "{} {:.0}%",
        ICON_HUMIDITY.truecolor(80, 170, 255),
        cur.relative_humidity_2m,
    );
    let pressure_val = if is_imperial() {
        format!(
            "{} {:.2} {}",
            ICON_GAUGE.truecolor(150, 150, 170),
            cur.surface_pressure,
            pressure_label(),
        )
    } else {
        format!(
            "{} {:.0} {}",
            ICON_GAUGE.truecolor(150, 150, 170),
            cur.surface_pressure,
            pressure_label(),
        )
    };
    let uv_val = format!("UV {:.0} {}", cur.uv_index, uv_label_str(cur.uv_index));
    let vis_val = format!(
        "{} {:.0} {}",
        ICON_EYE.truecolor(180, 180, 200),
        cur.visibility_km,
        visibility_label(),
    );
    let dew_val = format!("{:.0}° dew", cur.dewpoint_c).dimmed().to_string();
    let aqi_data = air.as_ref().map(|a| {
        let (r, g, b) = aqi_color(a.current.us_aqi);
        let aqi_val = format!(
            "{} AQI {:.0} {}",
            ICON_LEAF.truecolor(r, g, b),
            a.current.us_aqi,
            aqi_label_str(a.current.us_aqi),
        );
        let pm_val = format!("PM2.5 {:.0}  PM10 {:.0}", a.current.pm2_5, a.current.pm10)
            .dimmed()
            .to_string();
        (aqi_val, pm_val)
    });

    // Min column width for conditions (max of all padded columns + gap)
    let mut cond_pad_widths = vec![
        strip_ansi_len(&wind_val),
        strip_ansi_len(&hum_val),
        strip_ansi_len(&uv_val),
        strip_ansi_len(&vis_val),
    ];
    if let Some((ref av, _)) = aqi_data {
        cond_pad_widths.push(strip_ansi_len(av));
    }
    let min_cond_col = cond_pad_widths.into_iter().max().unwrap_or(0) + 2;

    // Astronomy content
    let astro_content = if !daily.sunrise.is_empty() {
        let rise = format_time(&daily.sunrise[0]);
        let set = format_time(&daily.sunset[0]);
        let dl = daylight_str(&rise, &set);
        let rise_str = format!("{} {} sunrise", ICON_SUNRISE.truecolor(255, 180, 50), rise,);
        let set_str = format!("{} {} sunset", ICON_SUNSET.truecolor(255, 100, 50), set,);
        let dl_str = dl.dimmed().to_string();
        let moon = daily.moon_phase.first().map(|phase| {
            let illum = daily.moon_illumination.first().copied().unwrap_or(0);
            let mrise = daily.moonrise.first().map(|s| s.as_str()).unwrap_or("—");
            let mset = daily.moonset.first().map(|s| s.as_str()).unwrap_or("—");
            let mrs = format!("\u{e3c1} {} moonrise", mrise).dimmed().to_string();
            let mss = format!("\u{e3c2} {} moonset", mset).dimmed().to_string();
            let ps = format!(
                "{} {} {}%",
                moon_icon(phase).truecolor(200, 200, 220),
                phase,
                illum,
            )
            .dimmed()
            .to_string();
            (mrs, mss, ps)
        });
        Some((rise_str, set_str, dl_str, moon))
    } else {
        None
    };

    // Min column width for astronomy
    let min_astro_col = if let Some((ref rs, ref ss, _, ref moon)) = astro_content {
        let mut widths = vec![strip_ansi_len(rs), strip_ansi_len(ss)];
        if let Some((mrs, mss, _)) = moon {
            widths.push(strip_ansi_len(mrs));
            widths.push(strip_ansi_len(mss));
        }
        widths.into_iter().max().unwrap_or(0) + 2
    } else {
        0
    };

    // Pre-compute full header line to measure its width
    let header_core = format!(
        "  {}  {}  {}  {}  {}",
        icon.truecolor(icon_color.0, icon_color.1, icon_color.2),
        temp_str.truecolor(tr, tg, tb).bold(),
        desc.white(),
        "·".dimmed(),
        format!("Feels {:.0}°", cur.apparent_temperature).dimmed(),
    );
    let mut header_full = header_core.clone();
    for part in hint.split(" · ") {
        header_full.push_str(&format!("  {}", format!("· {}", part).dimmed()));
    }
    if let Some(cmp) = tomorrow_comparison(daily) {
        header_full.push_str(&format!("  {}", format!("· {}", cmp).dimmed()));
    }
    let header_content = strip_ansi_len(&header_full);

    // Dynamic panel width: widest panel content drives all panels
    // 3*col+2 ensures (w-6)/3 >= col for even column distribution
    let hourly_content = 2 + 10 * 7;
    let max_content = [
        header_content,
        3 * min_cond_col + 2,
        3 * min_astro_col + 2,
        hourly_content,
    ]
    .into_iter()
    .max()
    .unwrap_or(72);
    let w = (max_content + 4).clamp(50, term_w);
    let col_w = (w.saturating_sub(6)) / 3;

    // ─── Phase 2: Render ─────────────────────────────────

    println!();
    render_alerts(&weather.alerts);

    // ─── Header Panel ─────────────────────────────────
    panel_top(&title, w);
    let inner = w.saturating_sub(4);

    let mut line = format!(
        "  {}  {}  {}  {}  {}",
        icon.truecolor(icon_color.0, icon_color.1, icon_color.2),
        temp_str.truecolor(tr, tg, tb).bold(),
        desc.white(),
        "·".dimmed(),
        format!("Feels {:.0}°", cur.apparent_temperature).dimmed(),
    );
    for part in hint.split(" · ") {
        let seg = format!("  {}", format!("· {}", part).dimmed());
        if strip_ansi_len(&line) + strip_ansi_len(&seg) <= inner {
            line.push_str(&seg);
        } else {
            break;
        }
    }
    if let Some(cmp) = tomorrow_comparison(daily) {
        let seg = format!("  {}", format!("· {}", cmp).dimmed());
        if strip_ansi_len(&line) + strip_ansi_len(&seg) <= inner {
            line.push_str(&seg);
        }
    }
    panel_row(&line, w);
    panel_bottom(w);

    // ─── Conditions Panel ─────────────────────────────
    panel_top("Conditions", w);
    panel_row(
        &format!(
            "  {}{}{}",
            pad_ansi(&wind_val, col_w),
            pad_ansi(&hum_val, col_w),
            pressure_val,
        ),
        w,
    );
    panel_row(
        &format!(
            "  {}{}{}",
            pad_ansi(&uv_val, col_w),
            pad_ansi(&vis_val, col_w),
            dew_val,
        ),
        w,
    );
    if let Some((aqi_val, pm_val)) = aqi_data {
        panel_row(&format!("  {}{}", pad_ansi(&aqi_val, col_w), pm_val), w);
    }
    panel_bottom(w);

    // ─── Hourly Table ─────────────────────────────────
    let now_h = current_hour();
    let col_count = ((w.saturating_sub(6)) / 7).clamp(4, 10);
    let cw = 7usize;

    let step = if col_count >= 8 { 3 } else { 24 / col_count };
    let hours: Vec<usize> = (0..col_count)
        .map(|i| {
            if i == 0 {
                now_h
            } else {
                (now_h + i * step) % 24
            }
        })
        .collect();

    panel_top("Next Hours", w);

    let mut time_row = String::from("  ");
    for (i, &h) in hours.iter().enumerate() {
        let label = if i == 0 {
            "Now".to_string()
        } else {
            format_hour_human(h)
        };
        time_row.push_str(&format!("{:^cw$}", label).dimmed().to_string());
    }
    panel_row(&time_row, w);

    let rise_mins = daily
        .sunrise
        .first()
        .map(|s| parse_time_mins(&format_time(s)));
    let set_mins = daily
        .sunset
        .first()
        .map(|s| parse_time_mins(&format_time(s)));

    let mut icon_row = String::from("  ");
    for &h in &hours {
        let idx = h % hourly.weather_code.len();
        let is_day = match (rise_mins, set_mins) {
            (Some(Some(r)), Some(Some(s))) => {
                let mins = (h as u32) * 60;
                mins >= r && mins < s
            }
            _ => (6..19).contains(&h),
        };
        let (ic, _, ic_color) = weather_icon(hourly.weather_code[idx], is_day);
        let colored_icon = ic.truecolor(ic_color.0, ic_color.1, ic_color.2).to_string();
        icon_row.push_str(&center_ansi(&colored_icon, 1, cw));
    }
    panel_row(&icon_row, w);

    let mut temp_row = String::from("  ");
    for &h in &hours {
        let idx = h % hourly.temperature_2m.len();
        let t = hourly.temperature_2m[idx];
        let tc = temp_colored(t).to_string();
        let vis_len = format!("{:.0}°", t).chars().count();
        temp_row.push_str(&center_ansi(&tc, vis_len, cw));
    }
    panel_row(&temp_row, w);

    let any_rain = hours.iter().any(|&h| {
        hourly.precipitation_probability[h % hourly.precipitation_probability.len()] > 0.0
    });
    if any_rain {
        let mut rain_row = String::from("  ");
        for &h in &hours {
            let idx = h % hourly.precipitation_probability.len();
            let r = hourly.precipitation_probability[idx];
            let rain_str = if r > 30.0 {
                format!("{:.0}%", r).truecolor(100, 160, 255).to_string()
            } else if r > 0.0 {
                format!("{:.0}%", r).dimmed().to_string()
            } else {
                " ".to_string()
            };
            let vis_len = if r > 0.0 {
                format!("{:.0}%", r).chars().count()
            } else {
                1
            };
            rain_row.push_str(&center_ansi(&rain_str, vis_len, cw));
        }
        panel_row(&rain_row, w);
    }

    panel_bottom(w);

    // ─── Forecast Panel ───────────────────────────────
    let days = 7.min(daily.time.len());
    let bar_width = w.saturating_sub(36).max(8);

    let abs_min = daily.temperature_2m_min[..days]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let abs_max = daily.temperature_2m_max[..days]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    panel_top(&format!("{}-Day Forecast", days), w);

    for i in 0..days {
        let (d_icon, _, d_color) = weather_icon(daily.weather_code[i], true);
        let bar = gradient_bar(
            daily.temperature_2m_min[i],
            daily.temperature_2m_max[i],
            abs_min,
            abs_max,
            bar_width,
        );
        let rain = if daily.precipitation_probability_max[i] > 0.0 {
            rain_indicator(daily.precipitation_probability_max[i])
        } else {
            String::new()
        };
        let fline = format!(
            "  {}  {}  {}  {}  {} {}",
            day_name(&daily.time[i]).dimmed(),
            d_icon.truecolor(d_color.0, d_color.1, d_color.2),
            temp_colored_dim(daily.temperature_2m_min[i]),
            bar,
            temp_colored_dim(daily.temperature_2m_max[i]),
            rain,
        );
        panel_row(&fline, w);
    }

    panel_bottom(w);

    // ─── Astronomy Panel ──────────────────────────────
    if let Some((rise_str, set_str, dl_str, moon)) = astro_content {
        panel_top("Astronomy", w);

        panel_row(
            &format!(
                "  {}{}{}",
                pad_ansi(&rise_str, col_w),
                pad_ansi(&set_str, col_w),
                dl_str,
            ),
            w,
        );

        if let Some((mrise_str, mset_str, phase_str)) = moon {
            panel_row(
                &format!(
                    "  {}{}{}",
                    pad_ansi(&mrise_str, col_w),
                    pad_ansi(&mset_str, col_w),
                    phase_str,
                ),
                w,
            );
        }

        panel_bottom(w);
    }

    println!();
}
