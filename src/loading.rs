use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const WEATHER_ICONS: &[(&str, (u8, u8, u8))] = &[
    ("\u{f0599}", (255, 200, 80)),  // sunny
    ("\u{f0590}", (144, 152, 152)), // cloudy
    ("\u{f0597}", (80, 128, 192)),  // rainy
    ("\u{f0598}", (176, 200, 224)), // snowy
    ("\u{f0595}", (208, 200, 120)), // partly
    ("\u{f0593}", (200, 160, 48)),  // thunder
];

pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(use_color: bool) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();

        let handle = std::thread::spawn(move || {
            let mut frame = 0usize;
            let mut icon_idx = 0usize;
            let mut tick = 0usize;

            // Hide cursor (only with color)
            if use_color {
                print!("\x1b[?25l");
                let _ = io::stdout().flush();
            }

            while !stop_clone.load(Ordering::Relaxed) {
                let spinner = SPINNER_FRAMES[frame];

                if use_color {
                    let (icon, (r, g, b)) = WEATHER_ICONS[icon_idx];
                    print!(
                        "\r  \x1b[38;2;{r};{g};{b}m{icon}\x1b[0m  \x1b[38;2;122;162;247m{spinner}\x1b[0m \x1b[38;2;136;136;136mFetching weather data...\x1b[0m"
                    );
                } else {
                    print!("\r  {spinner} Fetching weather data...");
                }
                let _ = io::stdout().flush();

                std::thread::sleep(std::time::Duration::from_millis(80));

                frame = (frame + 1) % SPINNER_FRAMES.len();
                tick += 1;
                // Cycle weather icon every ~600ms (7-8 spinner frames)
                if tick % 8 == 0 {
                    icon_idx = (icon_idx + 1) % WEATHER_ICONS.len();
                }
            }

            // Clear line and show cursor
            if use_color {
                print!("\r\x1b[2K\x1b[?25h");
            } else {
                print!("\r\x1b[2K");
            }
            let _ = io::stdout().flush();
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
