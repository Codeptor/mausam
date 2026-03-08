# mausam

Beautiful weather in your terminal.

```
   󰖙  37°                              Jaipur, India
      Sunny · Feels like 35°
      Light clothes, stay cool
      3° warmer than tomorrow

  ╶─────────────────────────────────────────────────────╴

   󰖝 12 km/h NW     󰖎 42%        UV 8 Very High
   󰉁 1012 hPa       10 km       󰖕 18° dew
   󰌒 AQI 88 Moderate · PM2.5 30 µg/m³ · PM10 118 µg/m³

  ╶─────────────────────────────────────────────────────╴

   Mon  󰖙   26°  ▃▄▅▆▇█▇▅▃   37°
   Tue  󰖕   24°  ▂▃▄▅▆▇▅▃▂   34°
   Wed  󰖙   25°  ▃▄▅▆▇█▆▄▂   36°

   󰖜 06:32     󰖛 18:28     11h 56m daylight
```

## Features

- True color gradients and Nerd Font icons
- Current conditions with feels-like, clothing hint, UV index
- Wind speed with compass direction, visibility, dew point
- 3-day and 7-day forecast with temperature range bars
- Hourly sparkline charts with rain probability
- Air quality index (AQI) with PM2.5/PM10 breakdown
- Weather alerts display
- Multi-word cities (`mausam new york`) and city,country (`mausam delhi, india`)
- Multi-city comparison (`mausam london / tokyo / paris`)
- Auto-detect location from IP
- JSON output for scripting
- Response caching (15 min TTL)
- Imperial and metric units
- `NO_COLOR` standard compliance
- Interactive first-run setup
- Cross-platform (Linux, macOS, Windows)

## Install

### From crates.io

```sh
cargo install mausam
```

### From source

```sh
git clone https://github.com/codeptor/mausam.git
cd mausam
cargo install --path .
```

### GitHub Releases

Pre-built binaries for Linux, macOS, and Windows are available on the [Releases](https://github.com/codeptor/mausam/releases) page.

## Setup

On first run, mausam will prompt you for a free [WeatherAPI](https://weatherapi.com/signup) key. Or configure it manually:

```sh
mausam --set-key YOUR_API_KEY
```

You can also set the key via environment variable:

```sh
export MAUSAM_API_KEY=YOUR_API_KEY
```

## Usage

```
mausam                         # Current weather (auto-detect city)
mausam london                  # Weather for London
mausam new york                # Multi-word city names work naturally
mausam delhi, india            # Disambiguate with country
mausam london / tokyo / paris  # Compare multiple cities
mausam -f                      # Full dashboard with hourly + 7-day
mausam -f new delhi            # Full view for a specific city
mausam -H                      # 24-hour detailed forecast
mausam -a                      # Air quality details
mausam -j                      # JSON output
mausam -r                      # Skip cache, force refresh
```

### Options

```
  [CITY]...              City name(s) — use / to separate multiple cities
  -f, --full             Full dashboard with hourly and 7-day forecast
  -H, --hourly           Show hourly forecast
  -a, --aqi              Show air quality details
  -j, --json             Output as JSON
  -r, --refresh          Force refresh, skip cache
      --no-color         Disable colored output
      --set-key <KEY>    Save API key to config
      --set-city <CITY>  Save default city to config
      --units <UNITS>    Set preferred units (metric/imperial)
      --config           Show current config
```

## Configuration

Config is stored at `~/.config/mausam/config.toml` (XDG-compliant):

```toml
api_key = "your_key_here"
default_city = "New Delhi"
units = "metric"        # or "imperial"
cache_ttl = 900         # seconds (default: 15 min)
```

View your current config:

```sh
mausam --config
```

## Requirements

- A terminal with true color support (most modern terminals)
- A [Nerd Font](https://www.nerdfonts.com/) for weather icons
- Free API key from [WeatherAPI.com](https://weatherapi.com/signup)

## License

MIT
