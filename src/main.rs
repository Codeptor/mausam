mod api;
mod display;
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

    let location = if let Some(ref city) = cli.city {
        api::geocode(city).await?
    } else {
        api::locate_ip().await?
    };

    // Fetch weather + AQI in parallel
    let (weather, air_quality) = tokio::join!(
        api::fetch_weather(&location),
        api::fetch_air_quality(&location),
    );

    let weather = weather?;
    let air_quality = air_quality.ok(); // AQI is optional, don't fail if unavailable

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
