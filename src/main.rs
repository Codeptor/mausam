mod api;
mod cache;
mod config;
mod display;
mod loading;
mod types;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

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

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("\n  {}\n", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Respect NO_COLOR env var and --no-color flag
    let use_color = !cli.no_color && std::env::var("NO_COLOR").is_err();
    if !use_color {
        colored::control::set_override(false);
    }

    // Handle config commands (print and exit)
    let mut cfg = config::Config::load();

    if let Some(key) = &cli.set_key {
        cfg.api_key = Some(key.clone());
        cfg.save()?;
        println!("  API key saved.");
        return Ok(());
    }
    if let Some(city) = &cli.set_city {
        cfg.default_city = Some(city.clone());
        cfg.save()?;
        println!("  Default city set to {}.", city);
        return Ok(());
    }
    if let Some(units) = &cli.units {
        cfg.units = Some(units.clone());
        cfg.save()?;
        println!("  Units set to {}.", units);
        return Ok(());
    }
    if cli.config {
        println!("  Config: {}", config::Config::config_path().display());
        println!("  API key: {}", if cfg.resolve_api_key().is_some() { "set" } else { "not set" });
        println!("  Default city: {}", cfg.default_city.as_deref().unwrap_or("auto-detect"));
        println!("  Units: {}", cfg.units.as_deref().unwrap_or("metric"));
        println!("  Cache TTL: {}s", cfg.cache_ttl);
        return Ok(());
    }

    // Resolve API key
    let api_key = match cfg.resolve_api_key() {
        Some(key) => key,
        None => {
            // Interactive first-run setup
            use std::io::{self, Write, BufRead};

            println!();
            println!("  {}", "Welcome to mausam!".bold());
            println!();
            println!("  To get started, you need a free WeatherAPI key:");
            println!("  1. Sign up at {}", "https://weatherapi.com/signup".dimmed());
            println!("  2. Copy your API key");
            println!();
            print!("  Paste your API key: ");
            io::stdout().flush()?;

            let key = io::stdin().lock().lines().next()
                .ok_or_else(|| anyhow::anyhow!("No input"))??.trim().to_string();

            if key.is_empty() {
                anyhow::bail!("No API key provided.");
            }

            cfg.api_key = Some(key.clone());
            cfg.save()?;
            println!("  {} Key saved to {}", "✓".green(), config::Config::config_path().display().to_string().dimmed());

            // Ask for default city
            println!();
            print!("  Set a default city? (leave blank for auto-detect): ");
            io::stdout().flush()?;

            let city = io::stdin().lock().lines().next()
                .ok_or_else(|| anyhow::anyhow!("No input"))??.trim().to_string();

            if !city.is_empty() {
                cfg.default_city = Some(city.clone());
                cfg.save()?;
                println!("  {} Default city set to {}", "✓".green(), city.bold());
            }
            println!();

            key
        }
    };

    // Determine cities
    let cities: Vec<String> = if !cli.city.is_empty() {
        cli.city.clone()
    } else {
        vec![cfg.default_city.clone().unwrap_or_else(|| "auto:ip".to_string())]
    };

    let cache = cache::Cache::new(cfg.cache_ttl);
    cache.cleanup();

    if cities.len() > 1 {
        // Multi-city mode: show compact view for each city
        for (i, city) in cities.iter().enumerate() {
            let query = city.clone();
            let cache_key = cache::Cache::key_for(&query);

            let (location, weather, air_quality) = if !cli.refresh {
                if let Some(cached) = cache.get(&cache_key) {
                    if let Ok(data) = api::parse_cached(&cached) {
                        data
                    } else {
                        let spinner = loading::Spinner::start(use_color);
                        let (loc, w, aq, raw) = api::fetch_all(&api_key, &query).await?;
                        spinner.stop();
                        cache.set(&cache_key, &raw);
                        (loc, w, aq)
                    }
                } else {
                    let spinner = loading::Spinner::start(use_color);
                    let (loc, w, aq, raw) = api::fetch_all(&api_key, &query).await?;
                    spinner.stop();
                    cache.set(&cache_key, &raw);
                    (loc, w, aq)
                }
            } else {
                let spinner = loading::Spinner::start(use_color);
                let (loc, w, aq, raw) = api::fetch_all(&api_key, &query).await?;
                spinner.stop();
                cache.set(&cache_key, &raw);
                (loc, w, aq)
            };

            display::compact(&location, &weather, &air_quality);

            if i < cities.len() - 1 {
                println!("  {}", "─".repeat(53).dimmed());
            }
        }
    } else {
        // Single-city mode: all view modes available
        let query = cities.into_iter().next().unwrap();
        let cache_key = cache::Cache::key_for(&query);

        let (location, weather, air_quality) = if !cli.refresh {
            if let Some(cached) = cache.get(&cache_key) {
                if let Ok(data) = api::parse_cached(&cached) {
                    data
                } else {
                    let spinner = loading::Spinner::start(use_color);
                    let (loc, w, aq, raw) = api::fetch_all(&api_key, &query).await?;
                    spinner.stop();
                    cache.set(&cache_key, &raw);
                    (loc, w, aq)
                }
            } else {
                let spinner = loading::Spinner::start(use_color);
                let (loc, w, aq, raw) = api::fetch_all(&api_key, &query).await?;
                spinner.stop();
                cache.set(&cache_key, &raw);
                (loc, w, aq)
            }
        } else {
            let spinner = loading::Spinner::start(use_color);
            let (loc, w, aq, raw) = api::fetch_all(&api_key, &query).await?;
            spinner.stop();
            cache.set(&cache_key, &raw);
            (loc, w, aq)
        };

        // Render
        if cli.json {
            display::json(&location, &weather, &air_quality);
        } else if cli.full {
            display::full(&location, &weather, &air_quality);
        } else if cli.hourly {
            display::hourly(&location, &weather);
        } else if cli.aqi {
            if let Some(ref aq) = air_quality {
                display::aqi_detail(&location, aq);
            } else {
                eprintln!("  Air quality data not available for this location.");
            }
        } else {
            display::compact(&location, &weather, &air_quality);
        }
    }

    Ok(())
}
