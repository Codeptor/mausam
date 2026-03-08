# mausam

Beautiful weather CLI built in Rust. Prints weather data and exits (no TUI/interactive mode).

## Architecture

```
src/
  main.rs      CLI entry (clap), orchestration, parallel API calls
  types.rs     Serde structs for Open-Meteo + ip-api responses
  api.rs       HTTP client: weather, AQI, geocoding, IP geolocation
  display.rs   All rendering: icons, colors, layout, sparklines
```

## APIs (no keys required)

- **Weather**: `api.open-meteo.com/v1/forecast` — current, hourly, daily
- **Air Quality**: `air-quality-api.open-meteo.com/v1/air-quality` — AQI, PM2.5, PM10
- **Geocoding**: `geocoding-api.open-meteo.com/v1/search` — city name → lat/lon
- **IP Location**: `ip-api.com/json/` — auto-detect user location

## Display Modes

- **Compact** (default): current conditions + 3-day forecast + sunrise/sunset
- **Full** (`-f`): adds hourly sparkline, 7-day forecast, pressure, PM details
- **Hourly** (`-H`): 24-hour breakdown with per-hour icons and rain
- **AQI** (`-a`): air quality bar + PM2.5/PM10

## Design Rules

- Nerd Font icons only (nf-md-weather_* range, U+F058E–U+F059D)
- True color throughout (`truecolor(r, g, b)`) — no 256-color fallback
- Temperature uses gradient: blue → cyan → green → yellow → orange → red
- No right-side box borders (avoids ANSI width alignment issues)
- Section dividers: `╶─────╴` style, dimmed
- Labels dimmed, values bold/colored
- AQI uses its own color scale: green → yellow → orange → red → purple
- Rain indicators only shown when probability > 0%

## Build

```sh
cargo build --release
cp target/release/mausam ~/.local/bin/
```

## Dependencies

- `clap` — CLI parsing
- `reqwest` + `tokio` — async HTTP (parallel API calls)
- `serde` — JSON deserialization
- `colored` — ANSI true color output
- `chrono` — time/date handling
- `anyhow` — error handling

## Conventions

- No interactive/TUI mode — static print and exit
- AQI is optional (`.ok()` on failure, never crashes)
- Weather + AQI fetched in parallel via `tokio::join!`
- All icon constants defined at top of display.rs
- Color helpers (`temp_to_rgb`, `aqi_color`) use interpolated gradients
- Release builds use LTO + strip for minimal binary size
