use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub struct Cache {
    dir: PathBuf,
    ttl: Duration,
}

impl Cache {
    pub fn new(ttl_secs: u64) -> Self {
        let dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("mausam");

        // Best-effort: create cache directory if it doesn't exist
        let _ = fs::create_dir_all(&dir);

        Self {
            dir,
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let path = self.dir.join(format!("{key}.json"));
        let metadata = fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        let elapsed = SystemTime::now().duration_since(modified).ok()?;

        if elapsed > self.ttl {
            return None;
        }

        fs::read_to_string(&path).ok()
    }

    pub fn set(&self, key: &str, data: &str) {
        let path = self.dir.join(format!("{key}.json"));
        let _ = fs::write(&path, data);
    }

    pub fn key_for(query: &str) -> String {
        query.to_lowercase().replace([' ', ':'], "_")
    }

    pub fn cleanup(&self) {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        let cutoff = Duration::from_secs(24 * 60 * 60);

        for entry in entries.flatten() {
            let ok = (|| {
                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().ok()?;
                let elapsed = SystemTime::now().duration_since(modified).ok()?;
                if elapsed > cutoff {
                    let _ = fs::remove_file(entry.path());
                }
                Some(())
            })();
            let _ = ok;
        }
    }
}
