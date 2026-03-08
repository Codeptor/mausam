# Mausam v1.0 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform mausam from a personal CLI into a publish-ready, polished weather tool with config management, caching, error handling, new features, and distribution via crates.io/Homebrew/GitHub Releases.

**Architecture:** Config-first approach — build the config system first since API key management blocks everything else. Then refactor display into modules, add caching, error handling, new features, and finally packaging. Each task produces a working build.

**Tech Stack:** Rust, clap (CLI), reqwest (HTTP), serde/serde_json (serialization), toml (config), dirs (XDG paths), colored (ANSI), chrono (time), tokio (async), anyhow (errors)

---

## Phase 1: Foundation

### Task 1: Update Cargo.toml with new dependencies and publish metadata

**Files:**
- Modify: `Cargo.toml`

**Step 1: Update Cargo.toml**

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
readme = "README.md"

[dependencies]
anyhow = "1"
chrono = "0.4"
clap = { version = "4", features = ["derive"] }
colored = "2"
dirs = "6"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
toml = "0.8"

[profile.release]
opt-level = 3
lto = true
strip = true
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: compiles with new deps downloaded

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add publish metadata and new dependencies (dirs, toml, serde_json)"
```

---

### Task 2: Create config module

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs` (add `mod config`)

**Step 1: Create `src/config.rs`**

Implements:
- `Config` struct with `api_key`, `default_city`, `units`, `cache_ttl` fields
- `Config::load()` — reads from XDG config path via `dirs::config_dir()`, falls back to defaults
- `Config::save()` — writes config to disk, creating parent dirs
- `Config::config_path()` — returns platform-appropriate config file path
- `Config::resolve_api_key()` — checks env var `MAUSAM_API_KEY` first, then config file
- All fields `Option<String>` except `cache_ttl` which defaults to 900

Config file format:
```toml
api_key = "..."
default_city = "jaipur"
units = "metric"
cache_ttl = 900
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: add config module with XDG-aware config file support"
```

---

### Task 3: Create cache module

**Files:**
- Create: `src/cache.rs`
- Modify: `src/main.rs` (add `mod cache`)

**Step 1: Create `src/cache.rs`**

Implements:
- `Cache::new(ttl_secs: u64)` — creates cache using `dirs::cache_dir()/mausam/`
- `Cache::get(key: &str) -> Option<String>` — returns cached JSON if file exists and mtime < TTL
- `Cache::set(key: &str, data: &str)` — writes JSON to cache file
- `Cache::key_for(query: &str) -> String` — normalizes query to lowercase filename (e.g. "New York" → "new_york", "auto:ip" → "auto_ip")
- `Cache::cleanup()` — deletes files older than 24h
- All operations are best-effort (errors silently ignored — cache is optional)

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/cache.rs src/main.rs
git commit -m "feat: add file-based response cache with configurable TTL"
```

---

### Task 4: Integrate config + cache into main flow and add new CLI flags

**Files:**
- Modify: `src/main.rs`
- Modify: `src/api.rs` (remove hardcoded API key, accept key as parameter)

**Step 1: Update `src/api.rs`**

- Remove `const API_KEY` line
- Change `fetch_all(query: &str)` to `fetch_all(key: &str, query: &str, units: &str)`
- Add `alerts=yes` to the API query
- Add `WapiAlert` type to `types.rs` and parse alerts from response
- Support imperial units: when `units == "imperial"`, convert temps to F, wind to mph, pressure to inHg in the conversion layer
- Return raw JSON string alongside parsed data for caching

**Step 2: Update `src/main.rs`**

New CLI struct:
```rust
#[derive(Parser)]
#[command(name = "mausam", about = "Beautiful weather in your terminal", version)]
struct Cli {
    /// City name(s) — auto-detects from IP if omitted
    city: Vec<String>,

    /// Full dashboard with hourly and 7-day forecast
    #[arg(short, long)]
    full: bool,

    /// Show hourly forecast
    #[arg(short = 'H', long)]
    hourly: bool,

    /// Show air quality details
    #[arg(short, long)]
    aqi: bool,

    /// Output as JSON
    #[arg(short, long)]
    json: bool,

    /// Force refresh, skip cache
    #[arg(short, long)]
    refresh: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Save API key to config
    #[arg(long, value_name = "KEY")]
    set_key: Option<String>,

    /// Save default city to config
    #[arg(long, value_name = "CITY")]
    set_city: Option<String>,

    /// Set preferred units (metric/imperial)
    #[arg(long, value_name = "UNITS")]
    units: Option<String>,

    /// Show current config
    #[arg(long)]
    config: bool,
}
```

Main flow:
1. Parse CLI args
2. Handle `--set-key`, `--set-city`, `--units`, `--config` (print and exit)
3. Load config
4. Resolve API key (env → flag → config → first-run prompt)
5. If no key, run interactive first-run setup
6. Determine query (CLI city arg → config default_city → "auto:ip")
7. Check cache (skip if `--refresh`)
8. If cache miss, start spinner, fetch from API, cache response, stop spinner
9. If `--json`, output JSON and exit
10. Render appropriate view
11. Handle `NO_COLOR` env var and `--no-color` flag

**Step 3: Verify it compiles and runs**

Run: `cargo run -- --help`
Expected: shows all new flags

**Step 4: Commit**

```bash
git add src/main.rs src/api.rs src/types.rs
git commit -m "feat: integrate config, cache, new CLI flags, remove hardcoded API key"
```

---

### Task 5: Add friendly error handling

**Files:**
- Modify: `src/api.rs`
- Modify: `src/main.rs`

**Step 1: Update `src/api.rs`**

- Instead of `.json().await.context("Failed...")`, manually check response status first
- Match on status codes:
  - 200: parse JSON normally
  - 401/403: return `anyhow!("Invalid API key...")` with helpful message
  - 400 with "No matching location": return `anyhow!("City \"{}\" not found...")`
  - 429: return `anyhow!("API rate limit reached...")`
  - Other: return generic error with status code
- For reqwest connection errors: catch and return "Could not connect..." message

**Step 2: Update `src/main.rs`**

- Wrap main logic in a helper function
- In `main()`, catch errors and print them nicely (no "Error:" prefix, just dimmed hint text)
- Ensure spinner stops before error prints (already handled by Drop)

**Step 3: Test error scenarios**

Run: `MAUSAM_API_KEY=badkey cargo run`
Expected: "Invalid API key. Check your key at https://weatherapi.com"

Run: `cargo run -- asdfghjkl`
Expected: "City \"asdfghjkl\" not found. Try a different spelling."

**Step 4: Commit**

```bash
git add src/api.rs src/main.rs
git commit -m "feat: add friendly error messages for common failure modes"
```

---

### Task 6: Add first-run interactive setup

**Files:**
- Modify: `src/main.rs`
- Modify: `src/config.rs`

**Step 1: Add interactive setup to `src/main.rs`**

When no API key is found:
```
  Welcome to mausam!

  To get started, you need a free WeatherAPI key:
  1. Sign up at https://weatherapi.com/signup
  2. Copy your API key

  Paste your API key:
```

- Read key from stdin
- Validate by making a test API call (fetch `auto:ip` forecast)
- If valid: save to config, ask for default city (optional), save, continue
- If invalid: print error, exit

Use `colored` crate for the welcome text styling (bold title, dimmed instructions).

**Step 2: Test first-run flow**

Delete config file, run `cargo run`, paste key, verify it saves and shows weather.

**Step 3: Commit**

```bash
git add src/main.rs src/config.rs
git commit -m "feat: add interactive first-run setup for API key"
```

---

## Phase 2: Refactor Display Module

### Task 7: Split display.rs into module directory

**Files:**
- Delete: `src/display.rs`
- Create: `src/display/mod.rs` (shared helpers: icons, colors, formatting functions)
- Create: `src/display/compact.rs` (compact view)
- Create: `src/display/full.rs` (full dashboard)
- Create: `src/display/hourly.rs` (hourly detail view)
- Create: `src/display/aqi.rs` (AQI detail view)
- Create: `src/display/json.rs` (JSON output)

**Step 1: Create `src/display/mod.rs`**

Move from current `display.rs`:
- All icon constants (lines 8-23)
- All color/formatting helpers: `temp_to_rgb`, `temp_colored`, `temp_colored_dim`, `gradient_bar`, `colored_sparkline`, `center_ansi`, `weather_icon`, `wind_arrow`, `uv_label`, `aqi_color`, `aqi_label`, `format_time`, `day_name`, `current_hour`, `daylight_str`, `format_hour_human`, `divider`, `rain_indicator` (lines 27-271)
- Make all helpers `pub(crate)`
- Add `pub mod compact; pub mod full; pub mod hourly; pub mod aqi; pub mod json;`
- Re-export view functions: `pub use compact::render as compact;` etc.

**Step 2: Create each view file**

Move each view function to its own file:
- `compact.rs`: `pub fn render(...)` (current `compact` function, lines 275-377)
- `full.rs`: `pub fn render(...)` (current `full` function, lines 381-563)
- `hourly.rs`: `pub fn render(...)` (current `hourly` function, lines 567-615)
- `aqi.rs`: `pub fn render(...)` (current `aqi_detail` function, lines 619-667)

Each file uses `use super::*;` to access shared helpers.

**Step 3: Create `json.rs`**

```rust
use serde_json::json;
use crate::types::*;

pub fn render(loc: &Location, weather: &WeatherResponse, air: &Option<AirQualityResponse>) {
    let output = json!({
        "location": { "name": &loc.name, "country": &loc.country },
        "current": {
            "temperature": weather.current.temperature_2m,
            "feels_like": weather.current.apparent_temperature,
            "humidity": weather.current.relative_humidity_2m,
            "wind_speed": weather.current.wind_speed_10m,
            "wind_direction": weather.current.wind_direction_10m,
            "pressure": weather.current.surface_pressure,
            "uv_index": weather.current.uv_index,
            "is_day": weather.current.is_day != 0,
        },
        // ... hourly, daily, aqi sections
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
```

**Step 4: Update `src/main.rs`**

- Change `display::compact(...)` → `display::compact(...)` (same interface, just re-exported)
- Add JSON branch: `if cli.json { display::json(...); }`

**Step 5: Delete old `src/display.rs`**

**Step 6: Verify all views work**

Run: `cargo run -- jaipur` (compact)
Run: `cargo run -- -f jaipur` (full)
Run: `cargo run -- -H jaipur` (hourly)
Run: `cargo run -- -a jaipur` (AQI)
Run: `cargo run -- --json jaipur` (JSON)

**Step 7: Commit**

```bash
git add src/display/ src/main.rs
git rm src/display.rs
git commit -m "refactor: split display.rs into focused module directory"
```

---

## Phase 3: New Features

### Task 8: Add weather alerts

**Files:**
- Modify: `src/types.rs` (add alert types)
- Modify: `src/api.rs` (parse alerts from response)
- Modify: `src/display/mod.rs` (add `render_alerts` helper)
- Modify: `src/display/compact.rs`, `src/display/full.rs` (show alerts at top)

**Step 1: Add alert types to `types.rs`**

Internal type:
```rust
pub struct Alert {
    pub headline: String,
    pub severity: String,    // "Moderate", "Severe", "Extreme"
    pub event: String,       // "Heat Advisory", "Flood Warning"
    pub expires: String,     // ISO datetime
}
```

WeatherAPI raw type:
```rust
#[derive(Deserialize)]
pub struct WapiAlert {
    pub headline: Option<String>,
    pub severity: Option<String>,
    pub event: Option<String>,
    pub expires: Option<String>,
}
```

Add `alerts` field to `WapiResponse`:
```rust
pub struct WapiResponse {
    pub location: WapiLocation,
    pub current: WapiCurrent,
    pub forecast: WapiForecast,
    pub alerts: Option<WapiAlerts>,
}

#[derive(Deserialize)]
pub struct WapiAlerts {
    pub alert: Vec<WapiAlert>,
}
```

Add `alerts: Vec<Alert>` to `WeatherResponse`.

**Step 2: Parse and convert alerts in `api.rs`**

**Step 3: Add alert rendering in `display/mod.rs`**

```rust
pub fn render_alerts(alerts: &[Alert]) {
    for alert in alerts {
        let color = match alert.severity.as_str() {
            "Extreme" => (255, 50, 50),
            "Severe" => (255, 140, 0),
            _ => (230, 200, 0),
        };
        println!(
            "   {} {}",
            "!".truecolor(color.0, color.1, color.2).bold(),
            alert.headline.truecolor(color.0, color.1, color.2),
        );
    }
    if !alerts.is_empty() { println!(); }
}
```

Call `render_alerts` at the top of compact and full views.

**Step 4: Commit**

```bash
git add src/types.rs src/api.rs src/display/
git commit -m "feat: show weather alerts from WeatherAPI"
```

---

### Task 9: Add clothing hint

**Files:**
- Modify: `src/display/mod.rs` (add `clothing_hint` function)
- Modify: `src/display/compact.rs`, `src/display/full.rs` (show hint below "Feels like")

**Step 1: Add `clothing_hint` to `display/mod.rs`**

```rust
pub fn clothing_hint(feels_like: f64, rain_chance: f64, uv: f64) -> String {
    let base = match feels_like as i32 {
        i32::MIN..=-1 => "Bundle up, it's freezing",
        0..=9 => "Grab a warm jacket",
        10..=17 => "Light jacket weather",
        18..=24 => "Comfortable, no layers needed",
        25..=32 => "Light clothes, stay cool",
        33..=39 => "Stay hydrated, it's scorching",
        _ => "Dangerously hot, limit outdoor exposure",
    };
    let mut hint = base.to_string();
    if rain_chance > 60.0 { hint.push_str(" · Carry an umbrella"); }
    if uv > 7.0 { hint.push_str(" · Wear sunscreen"); }
    hint
}
```

**Step 2: Show hint in compact and full views**

Below the "Feels like" line:
```rust
let today_rain = daily.precipitation_probability_max.first().copied().unwrap_or(0.0);
println!("      {}", clothing_hint(cur.apparent_temperature, today_rain, cur.uv_index).dimmed());
```

**Step 3: Commit**

```bash
git add src/display/
git commit -m "feat: add clothing hint based on temperature, rain, and UV"
```

---

### Task 10: Add tomorrow comparison

**Files:**
- Modify: `src/display/mod.rs` (add `tomorrow_comparison` function)
- Modify: `src/display/compact.rs`, `src/display/full.rs`

**Step 1: Add `tomorrow_comparison` to `display/mod.rs`**

```rust
pub fn tomorrow_comparison(today_temp: f64, daily: &DailyWeather) -> Option<String> {
    if daily.temperature_2m_max.len() < 2 { return None; }
    let today_high = daily.temperature_2m_max[0];
    let tomorrow_high = daily.temperature_2m_max[1];
    let diff = (today_high - tomorrow_high).round() as i32;
    if diff.abs() < 2 { return None; }
    if diff > 0 {
        Some(format!("{diff}° warmer than tomorrow"))
    } else {
        Some(format!("{}° cooler than tomorrow", diff.abs()))
    }
}
```

**Step 2: Show in header area of compact and full views**

After the "Feels like" line, if comparison exists:
```rust
if let Some(cmp) = tomorrow_comparison(cur.temperature_2m, daily) {
    println!("      {}", cmp.dimmed());
}
```

**Step 3: Commit**

```bash
git add src/display/
git commit -m "feat: add tomorrow temperature comparison in header"
```

---

### Task 11: Add multi-city support

**Files:**
- Modify: `src/main.rs`

**Step 1: Update main flow for multiple cities**

When `cli.city.len() > 1`:
- Loop through each city
- Fetch weather for each (sequentially to avoid rate limiting)
- Show compact view for each with a divider between
- If view flags (`-f`, `-H`, `-a`) are set, warn "Full/hourly/AQI view only available for single city" and fall back to compact

When `cli.city.len() == 0`:
- Use config default_city or "auto:ip"

When `cli.city.len() == 1`:
- Normal single-city behavior with all view modes available

**Step 2: Test**

Run: `cargo run -- jaipur london tokyo`
Expected: three stacked compact views with dividers

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add multi-city support (mausam jaipur london tokyo)"
```

---

### Task 12: Add NO_COLOR and --no-color support

**Files:**
- Modify: `src/main.rs`
- Modify: `src/loading.rs`

**Step 1: Detect NO_COLOR**

In `main()`, before any output:
```rust
if cli.no_color || std::env::var("NO_COLOR").is_ok() {
    colored::control::set_override(false);
}
```

The `colored` crate respects `set_override(false)` globally — all `.truecolor()`, `.bold()`, `.dimmed()` calls become no-ops.

**Step 2: Update spinner**

In `loading.rs`, check if color is disabled. If so, use plain ASCII spinner without ANSI color codes:
```
\r  Fetching weather data...
```

**Step 3: Suppress spinner for --json**

Don't start the spinner when `--json` is set (JSON output should be clean).

**Step 4: Commit**

```bash
git add src/main.rs src/loading.rs
git commit -m "feat: support NO_COLOR env var and --no-color flag"
```

---

### Task 13: Add imperial units support

**Files:**
- Modify: `src/api.rs` (convert units in conversion layer)
- Modify: `src/display/mod.rs` (pass units context for labels like km/h vs mph)

**Step 1: Unit conversion in api.rs**

When units == "imperial":
- Temperatures: already in C from API, convert to F: `f = c * 9.0/5.0 + 32.0`
- Wind: km/h to mph: `mph = kph * 0.621371`
- Pressure: mb to inHg: `inhg = mb * 0.02953`

Apply conversions when building `WeatherResponse` from `WapiResponse`.

**Step 2: Update display labels**

Pass a `units: &str` parameter to view functions. When imperial:
- Show "mph" instead of "km/h"
- Show "inHg" instead of "hPa"
- Temperature gradient stops adjusted for F scale

**Step 3: Commit**

```bash
git add src/api.rs src/display/ src/main.rs
git commit -m "feat: add imperial units support (--units imperial)"
```

---

## Phase 4: Polish & Packaging

### Task 14: Add LICENSE file

**Files:**
- Create: `LICENSE`

**Step 1: Create MIT license**

Standard MIT license with "codeptor" as copyright holder, year 2026.

**Step 2: Commit**

```bash
git add LICENSE
git commit -m "chore: add MIT license"
```

---

### Task 15: Add README.md

**Files:**
- Create: `README.md`

**Step 1: Write README**

Structure:
- Hero: `mausam` title + one-line description
- Install: cargo, brew, binary download
- Quick Start: get API key + first run
- Usage: all CLI flags with examples
- Views: compact, full, hourly, AQI descriptions
- Config: file location, all options, env vars
- JSON output: example with jq
- Contributing + License

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with install, usage, and config reference"
```

---

### Task 16: Add GitHub Actions CI

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Create CI workflow**

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo build --release
```

**Step 2: Commit**

```bash
git add .github/
git commit -m "ci: add GitHub Actions workflow for fmt, clippy, and build"
```

---

### Task 17: Add GitHub Actions release workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Create release workflow**

Triggers on version tags (`v*`). Cross-compiles for:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Uses `cross` for Linux cross-compilation. Uploads binaries to GitHub Release.

**Step 2: Commit**

```bash
git add .github/
git commit -m "ci: add cross-platform release workflow"
```

---

### Task 18: Final polish and cleanup

**Files:**
- All source files

**Step 1: Run clippy and fix all warnings**

Run: `cargo clippy -- -D warnings`
Fix any issues.

**Step 2: Run fmt**

Run: `cargo fmt`

**Step 3: Suppress dead_code warnings**

Add `#[allow(dead_code)]` where appropriate (e.g. `latitude`/`longitude` on Location).

**Step 4: Test all views and modes**

Run each combination:
- `cargo run -- jaipur` (compact)
- `cargo run -- -f jaipur` (full)
- `cargo run -- -H jaipur` (hourly)
- `cargo run -- -a jaipur` (AQI)
- `cargo run -- --json jaipur` (JSON)
- `cargo run -- --no-color jaipur` (no color)
- `cargo run -- jaipur london` (multi-city)
- `cargo run -- --config` (show config)
- `cargo run -- --refresh jaipur` (skip cache)

**Step 5: Commit**

```bash
git add .
git commit -m "chore: final polish, clippy fixes, fmt"
```

---

## Task Dependency Order

```
Task 1 (deps) → Task 2 (config) → Task 3 (cache) → Task 4 (integrate)
                                                         ↓
Task 5 (errors) → Task 6 (first-run) → Task 7 (refactor display)
                                              ↓
Task 8 (alerts) ─┐
Task 9 (hints)  ─┤
Task 10 (tmrw)  ─┼→ Task 13 (imperial) → Task 14 (LICENSE)
Task 11 (multi) ─┤                              ↓
Task 12 (color) ─┘                     Task 15 (README) → Task 16 (CI) → Task 17 (release) → Task 18 (polish)
```

Tasks 8-12 are independent and can be done in parallel or any order after Task 7.
