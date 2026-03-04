use std::fs;
use std::path::Path;
use log::{info, warn};

const CRASH_MARKER_FILE: &str = ".agent_running";

pub struct CrashMarker {
    path: String,
}

impl CrashMarker {
    pub fn new(db_path: &str) -> Self {
        let dir = if db_path.starts_with("sqlite:") {
            let path = db_path.strip_prefix("sqlite:").unwrap_or(db_path);
            let path = path.split('?').next().unwrap_or(path);
            Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        } else {
            // For Redb paths, strip .redb extension
            let path = db_path.split('?').next().unwrap_or(db_path);
            Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        };

        let path = format!("{dir}/{CRASH_MARKER_FILE}");
        Self { path }
    }

    pub fn check_crash(&self) -> bool {
        Path::new(&self.path).exists()
    }

    pub fn read_crash_info(&self) -> Option<String> {
        fs::read_to_string(&self.path).ok()
    }

    pub fn write(&self) -> std::io::Result<()> {
        let content = format!(
            "pid={}\nstarted={}\n",
            std::process::id(),
            chrono::Utc::now().to_rfc3339()
        );
        if let Some(parent) = Path::new(&self.path).parent()
            && !parent.as_os_str().is_empty() && parent != Path::new("") {
                fs::create_dir_all(parent)?;
            }
        fs::write(&self.path, content)
    }

    pub fn remove(&self) -> std::io::Result<()> {
        if Path::new(&self.path).exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// Recover from crash by resetting in-flight logs
pub fn recover_from_crash(store: &crate::redb_store::RedbStore) -> Result<u64, crate::redb_store::StoreError> {
    info!("Crash recovery: Resetting in-flight logs...");

    let reset_count = store.reset_syncing_status()?;
    let stale_count = store.reset_stale_syncing(300)?; // 5 minute timeout

    let total = reset_count + stale_count;
    if total > 0 {
        warn!("Crash recovery: Reset {total} in-flight logs to pending");
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_marker_path_extraction() {
        let marker1 = CrashMarker::new("sqlite:test.db");
        assert!(marker1.path.contains(".agent_running"));

        let marker2 = CrashMarker::new("sqlite:test.db?mode=rwc");
        assert!(marker2.path.contains(".agent_running"));

        let marker3 = CrashMarker::new("sqlite:/tmp/test.db");
        assert!(marker3.path.contains("/tmp/.agent_running"));

        let marker4 = CrashMarker::new("sqlite:/var/lib/agent/test.db");
        assert!(marker4.path.contains("/var/lib/agent/.agent_running"));

        let marker5 = CrashMarker::new("sqlite:./data/test.db");
        assert!(marker5.path.contains(".agent_running"));
    }

    #[test]
    fn test_crash_marker_memory_database_format() {
        let marker = CrashMarker::new("sqlite::memory:");
        assert!(marker.path.contains(".agent_running"));
    }

    #[test]
    fn test_crash_marker_query_string_stripped() {
        let marker1 = CrashMarker::new("sqlite:foo.db?mode=rwc");
        let marker2 = CrashMarker::new("sqlite:foo.db");
        assert_eq!(marker1.path, marker2.path);
    }

    #[test]
    fn test_crash_marker_redb_paths() {
        let marker = CrashMarker::new("/var/lib/agent/data.redb");
        assert!(marker.path.contains("/var/lib/agent/.agent_running"));
    }
}
