mod api;
mod config;
mod display;
mod loading;
mod types;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "mausam", about = "Beautiful weather in your terminal ⛅", version)]
struct Cli {
    /// City name (auto-detects from IP if omitted)
    city: Option<String>,

    /// Full dashboard with hourly and 7-day forecast
    #[arg(short, long)]
    full: bool,

    /// Show hourly forecast
    #[arg(short = 'H', long)]
    hourly: bool,

    /// Show air quality details
    #[arg(short, long)]
    aqi: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let spinner = loading::Spinner::start();

    let query = cli.city.as_deref().unwrap_or("auto:ip");
    let (location, weather, air_quality) = api::fetch_all(query).await?;

    spinner.stop();

    if cli.full {
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

    Ok(())
}
