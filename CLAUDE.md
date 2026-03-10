# mausam — Project Guide

## What is this?
A Rust CLI weather app. Uses WeatherAPI.com for data. Displays beautiful terminal output with true color and Nerd Font icons.

## Owner
GitHub: codeptor (Bhanu)

## Build & Test
```sh
cargo build           # dev build
cargo build --release # release build
cargo clippy          # lint
cargo fmt --check     # format check
cargo test            # tests (minimal currently)
```

## Key Architecture
- `src/main.rs` — CLI args (clap derive), async entry point
- `src/api.rs` — WeatherAPI fetch + response parsing
- `src/types.rs` — All data structs (CurrentWeather, DailyWeather, HourlyWeather, etc.)
- `src/display/mod.rs` — Shared display utils (icons, colors, terminal width, formatters)
- `src/display/compact.rs` — Default view (7-day forecast, astronomy, hourly)
- `src/display/hourly.rs` — Hourly forecast (-H)
- `src/display/aqi.rs` — Air quality view (-a)
- `src/display/json.rs` — JSON output (-j)
- `src/config.rs` — TOML config (~/.config/mausam/config.toml)
- `src/cache.rs` — JSON response cache (~/.cache/mausam/)
- `src/loading.rs` — Spinner animation

## Important Patterns
- ANSI-aware alignment: pad plain text FIRST, then colorize
- Icon colors are context-aware (day=yellow, night=blue, rain=blue, snow=white, etc.)
- Terminal width: detected via `terminal_size` crate (ioctl TIOCGWINSZ), COLUMNS env fallback
- Two-phase rendering: pre-compute content widths, then set all panels to the widest
- Bordered panels: panel_top/panel_row/panel_bottom with dynamic width
- Day/night detection: uses actual sunrise/sunset times from API, not hardcoded hours
- reqwest uses rustls-tls (not native-tls) to avoid OpenSSL cross-compilation issues

## Distribution
- crates.io: `cargo install mausam`
- Homebrew: `brew tap codeptor/tap && brew install mausam` (repo: codeptor/homebrew-tap)
- AUR: `yay -S mausam` (pushed to aur.archlinux.org/mausam.git)
- GitHub Releases: automated via .github/workflows/release.yml

## Preferences
- No co-author lines in commits
- Keep output pretty
- Rust edition 2024
