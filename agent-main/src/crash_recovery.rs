use std::fs;
use std::path::Path;
use log::{info, warn};

const CRASH_MARKER_FILE: &str = ".agent_running";

pub struct CrashMarker {
    path: String,
}

impl CrashMarker {
    pub fn new(db_path: &str) -> Self {
        // Extract directory from database URL
        // "sqlite:foo.db?mode=rwc" -> "."
        // "sqlite:/path/to/foo.db" -> "/path/to"
        let dir = if db_path.starts_with("sqlite:") {
            let path = db_path.strip_prefix("sqlite:").unwrap_or(db_path);
            let path = path.split('?').next().unwrap_or(path);
            Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        } else {
            ".".to_string()
        };

        let path = format!("{}/{}", dir, CRASH_MARKER_FILE);
        Self { path }
    }

    /// Check if crash marker exists (indicates previous crash)
    pub fn check_crash(&self) -> bool {
        Path::new(&self.path).exists()
    }

    /// Read crash marker content (for logging)
    pub fn read_crash_info(&self) -> Option<String> {
        fs::read_to_string(&self.path).ok()
    }

    /// Write crash marker (called on startup)
    pub fn write(&self) -> std::io::Result<()> {
        let content = format!(
            "pid={}\nstarted={}\n",
            std::process::id(),
            chrono::Utc::now().to_rfc3339()
        );
        // Ensure parent directory exists
        if let Some(parent) = Path::new(&self.path).parent() {
            if !parent.as_os_str().is_empty() && parent != Path::new("") {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&self.path, content)
    }

    /// Remove crash marker (called on clean shutdown)
    pub fn remove(&self) -> std::io::Result<()> {
        if Path::new(&self.path).exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// Recover from crash by resetting in-flight logs and stale syncing states
pub async fn recover_from_crash(db: &crate::database::Database) -> Result<u64, sqlx::Error> {
    info!("Crash recovery: Resetting in-flight logs...");

    // Reset is_synced=2 to is_synced=0
    let reset_count = db.reset_syncing_status().await?;

    // Also reset any logs stuck in stale syncing state
    let stale_count = db.reset_stale_syncing(300).await?; // 5 minute timeout

    let total = reset_count + stale_count;
    if total > 0 {
        warn!("Crash recovery: Reset {} in-flight logs to pending", total);
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_marker_path_extraction() {
        // Test various database URL formats
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
        // For :memory:, parent is "." so path should be "./.agent_running"
        assert!(marker.path.ends_with("/.agent_running") || marker.path.ends_with(".agent_running"));
    }

    #[test]
    fn test_crash_marker_query_string_stripped() {
        // Test that query strings are stripped from URL
        let marker1 = CrashMarker::new("sqlite:foo.db?mode=rwc");
        let marker2 = CrashMarker::new("sqlite:foo.db");
        assert_eq!(marker1.path, marker2.path);

        let marker3 = CrashMarker::new("sqlite:/path/to/db.db?journal=wal&timeout=1000");
        let marker4 = CrashMarker::new("sqlite:/path/to/db.db");
        assert_eq!(marker3.path, marker4.path);
    }

    #[test]
    fn test_crash_marker_absolute_paths() {
        // Test absolute path extraction
        let marker1 = CrashMarker::new("sqlite:/absolute/path/to/db.db");
        assert!(marker1.path.contains("/absolute/path/to"));

        let marker2 = CrashMarker::new("sqlite:/home/user/data/db.db");
        assert!(marker2.path.contains("/home/user/data"));
    }

    #[test]
    fn test_crash_marker_relative_paths() {
        // Test relative path extraction
        let marker1 = CrashMarker::new("sqlite:data/db.db");
        assert!(marker1.path.contains(".agent_running"));

        let marker2 = CrashMarker::new("sqlite:./db.db");
        assert!(marker2.path.contains(".agent_running"));
    }

    #[test]
    fn test_crash_marker_nested_relative_paths() {
        // Test nested relative paths
        let marker1 = CrashMarker::new("sqlite:data/subdir/db.db");
        assert!(marker1.path.contains("data/subdir"));

        let marker2 = CrashMarker::new("sqlite:../data/db.db");
        assert!(marker2.path.contains("../data"));
    }
}

