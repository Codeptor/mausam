# Mausam v1.0 — Publish-Ready Design

## Overview

Transform mausam from a personal CLI tool into a polished, publish-ready weather CLI that anyone can install and use. This covers config management, caching, better error handling, new features (alerts, multi-city, clothing hints, JSON output), codebase refactoring, and distribution via crates.io, Homebrew, and GitHub Releases.

## 1. Project Structure & Refactor

Split the monolithic `display.rs` (650+ lines) into focused modules:

```
src/
├── main.rs           # CLI parsing, orchestration
├── config.rs         # Config file + API key management
├── api.rs            # WeatherAPI client
├── cache.rs          # Response caching (TTL-based)
├── types.rs          # API response types + internal types
├── display/
│   ├── mod.rs        # Shared helpers (colors, icons, formatting)
│   ├── compact.rs    # Default view
│   ├── full.rs       # Full dashboard
│   ├── hourly.rs     # Hourly detail view
│   ├── aqi.rs        # AQI detail view
│   └── json.rs       # JSON output mode
└── loading.rs        # Spinner animation
```

## 2. Config System

### Config file

Location: XDG-aware via `dirs` crate.
- Linux: `$XDG_CONFIG_HOME/mausam/config.toml` (default `~/.config/mausam/config.toml`)
- macOS: `~/Library/Application Support/mausam/config.toml`
- Windows: `%APPDATA%/mausam/config.toml`

```toml
api_key = "your_key_here"
default_city = "jaipur"
units = "metric"          # metric | imperial
cache_ttl = 900           # seconds (15 min default)
```

### Key resolution order (highest priority wins)

1. `MAUSAM_API_KEY` env var
2. `--key` CLI flag
3. Config file `api_key`

### First-run flow

When no API key is configured:

```
$ mausam

  Welcome to mausam!

  To get started, you need a free WeatherAPI key:
  1. Sign up at https://weatherapi.com/signup
  2. Copy your API key

  Paste your API key: ████████████████████████

  Key saved to ~/.config/mausam/config.toml

  Set a default city? (leave blank for auto-detect): jaipur
  Default city set to jaipur
```

### New CLI flags

```
mausam --set-key KEY        # save key to config
mausam --set-city CITY      # save default city
mausam --units imperial     # save preferred units
mausam --config             # print current config path & values
```

### Dependencies

- `dirs = "6"` for XDG paths
- `toml = "0.8"` for config parsing

## 3. Caching

File-based cache using XDG cache directory:

```
~/.cache/mausam/
├── jaipur.json
├── london.json
└── auto_ip.json
```

### Rules

- Cache key = lowercase city name (or `auto_ip` for IP detection)
- Default TTL = 15 minutes, configurable via `cache_ttl` in config
- `mausam --refresh` force-skips cache
- Uses XDG cache dir (platform-aware via `dirs`)
- Stale files (>24h old) auto-cleaned on each run
- No new dependencies — `std::fs` + file mtime check

## 4. Error Handling

Replace raw anyhow error dumps with friendly, actionable messages.

| Condition | Message |
|-----------|---------|
| No API key | `No API key configured. Run 'mausam --set-key YOUR_KEY'` + signup link |
| Invalid key (401/403) | `Invalid API key. Check your key at https://weatherapi.com` |
| City not found | `City "xyz" not found. Try a different spelling.` |
| No internet | `Could not connect. Check your internet connection.` |
| Rate limited (429) | `API rate limit reached. Try again in a few minutes.` |

Implementation: check HTTP status codes and reqwest error kinds before JSON parsing. Spinner stops cleanly before any error output.

## 5. New Features

### Weather Alerts

- Add `alerts=yes` to WeatherAPI query
- Show active alerts at the top of any view in bold yellow/red
- Format: `! Heat Advisory - Extreme heat expected until Wed 6pm`

### Multi-City

- `mausam jaipur london tokyo` — stacked vertically with dividers
- Each city gets its own compact view
- Full/hourly/AQI modes only apply with a single city

### Clothing Hint

One-liner below "Feels like" based on apparent temperature:

| Range | Hint |
|-------|------|
| <0 | Bundle up, it's freezing |
| 0-10 | Grab a warm jacket |
| 10-18 | Light jacket weather |
| 18-25 | Comfortable, no layers needed |
| 25-33 | Light clothes, stay cool |
| 33-40 | Stay hydrated, it's scorching |
| 40+ | Dangerously hot, limit outdoor exposure |

Appended conditions:
- Rain >60%: `Carry an umbrella`
- UV >7: `Wear sunscreen`

### Tomorrow Comparison

In the header: `3 warmer than tomorrow` or `5 cooler than tomorrow`. Only shown when difference is >= 2 degrees.

### --json Flag

Structured JSON to stdout. No colors, no spinner. Fields: location, current, hourly, daily, aqi. Useful for jq, status bars, scripts.

### --no-color + NO_COLOR Env

- Respect `NO_COLOR` env var standard (no-color.org)
- `--no-color` CLI flag
- Strips all ANSI codes — plain text output

## 6. Publishing & Distribution

### Cargo.toml

```toml
[package]
name = "mausam"
version = "1.0.0"
edition = "2021"
description = "Beautiful weather in your terminal"
license = "MIT"
repository = "https://github.com/codeptor/mausam"
keywords = ["weather", "cli", "terminal", "forecast"]
categories = ["command-line-utilities"]
```

### New dependencies

```toml
dirs = "6"
toml = "0.8"
serde_json = "1"
```

### Distribution channels

1. **crates.io** — `cargo install mausam`
2. **GitHub Releases** — prebuilt binaries for Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)
3. **Homebrew tap** — `brew install codeptor/tap/mausam`
4. **GitHub Actions CI** — build + test on push, cross-compile releases on version tag

### Files to add

- `LICENSE` (MIT)
- `README.md` — hero screenshot, install instructions, all flags, config reference, JSON examples
- `.github/workflows/ci.yml` — test + clippy + fmt on push
- `.github/workflows/release.yml` — cross-compile + upload on version tag

## 7. Refactoring Notes

- Remove hardcoded API key from api.rs — key comes from config/env/flag
- Extract display helpers (colors, icons, formatting) into `display/mod.rs`
- Each view in its own file under `display/`
- Imperial unit support: convert temps (F), wind (mph), pressure (inHg) in display layer based on config
- Add `Serialize` derive to internal types for JSON output + caching
