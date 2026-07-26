//! Centralized Environment Variable Registry
//!
//! SINGLE SOURCE OF TRUTH for all environment variables consumed by apiserver.
//! Every env var is declared exactly once in [`ALL_ENV_VARS`] with its metadata.
//!
//! # Usage
//!
//! ```rust,ignore
//! use apiserver::utils::env_registry::EnvRegistry;
//!
//! let registry = EnvRegistry::resolve().expect("Missing required env vars");
//! registry.show_summary();
//!
//! let db_url = registry.get("DATABASE_URL").unwrap();
//! let max_conn: u32 = registry.get_parsed("DB_API_MAX_CONN").unwrap_or(20);
//! ```

use std::fmt;

// ── Data Structures ──────────────────────────────────────────────────────

/// How an env var value was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    /// Value came from the environment.
    Env,
    /// Value is a built-in default.
    Default,
    /// Not set and no default available (optional vars only).
    Missing,
}

impl fmt::Display for ValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueSource::Env => write!(f, "env"),
            ValueSource::Default => write!(f, "default"),
            ValueSource::Missing => write!(f, "not set"),
        }
    }
}

/// Sensitivity classification for display masking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sensitivity {
    /// Value is safe to display in full.
    Public,
    /// Value is a secret — show only length/redacted marker.
    Secret,
    /// Value is a connection URL — mask password portion.
    Connection,
}

/// Category for grouping in the startup display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvCategory {
    Core,
    Database,
    Redis,
    Security,
    Storage,
    Worker,
    RateLimit,
    Scan,
    Integration,
    FeatureFlag,
}

impl fmt::Display for EnvCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvCategory::Core => write!(f, "Core"),
            EnvCategory::Database => write!(f, "Database"),
            EnvCategory::Redis => write!(f, "Redis"),
            EnvCategory::Security => write!(f, "Security"),
            EnvCategory::Storage => write!(f, "Storage"),
            EnvCategory::Worker => write!(f, "Worker"),
            EnvCategory::RateLimit => write!(f, "Rate Limiting"),
            EnvCategory::Scan => write!(f, "Virus Scanning"),
            EnvCategory::Integration => write!(f, "Integration"),
            EnvCategory::FeatureFlag => write!(f, "Feature Flags"),
        }
    }
}

/// Declaration of a single environment variable — its name, aliases,
/// default, and metadata. This is the canonical definition used for
/// resolution, display, and `.env.example` generation.
#[derive(Debug, Clone)]
pub struct EnvVarDef {
    /// Primary (canonical) env var key.
    pub key: &'static str,
    /// Alternative keys checked in order if primary is not set.
    pub aliases: &'static [&'static str],
    /// Category for grouping in the startup display.
    pub category: EnvCategory,
    /// Human-readable description (used in `.env.example` generation).
    pub description: &'static str,
    /// Whether this MUST be set in all environments (fail-fast if missing).
    pub required_always: bool,
    /// Whether this MUST be set in production mode.
    pub required_in_prod: bool,
    /// Default value if not set in the environment.
    pub default_value: Option<&'static str>,
    /// How sensitive the value is (controls display masking).
    pub sensitivity: Sensitivity,
}

/// A resolved environment variable with its actual value and metadata.
#[derive(Debug, Clone)]
pub struct ResolvedEnv {
    pub def: &'static EnvVarDef,
    pub value: String,
    pub source: ValueSource,
    /// The key that actually resolved (primary or alias).
    pub resolved_key: &'static str,
}

/// The complete resolved environment — built once at startup.
#[derive(Debug, Clone)]
pub struct EnvRegistry {
    pub resolved: Vec<ResolvedEnv>,
    /// Resolved APP_ENV value (convenience accessor).
    pub app_env: String,
}

impl EnvRegistry {
    /// Resolve all environment variables declared in [`ALL_ENV_VARS`].
    ///
    /// For each declaration:
    /// 1. Check the primary key, then aliases in order.
    /// 2. Use the default if no env var is set.
    /// 3. Collect errors for `required_always` vars that are missing.
    /// 4. In production mode, also collect errors for `required_in_prod` vars.
    ///
    /// Returns `Ok(registry)` with all resolved values, or `Err(list_of_errors)`.
    pub fn resolve() -> Result<Self, Vec<String>> {
        let mut resolved = Vec::with_capacity(ALL_ENV_VARS.len());
        let mut errors: Vec<String> = Vec::new();

        // First pass: resolve all values
        for def in ALL_ENV_VARS {
            // Check primary key first, then aliases, then default
            let (resolved_key, value, source) = if let Ok(v) = std::env::var(def.key) {
                (def.key, Some(v), ValueSource::Env)
            } else {
                let mut alias_result = None;
                for alias in def.aliases {
                    if let Ok(v) = std::env::var(alias) {
                        alias_result = Some((alias, v));
                        break;
                    }
                }
                if let Some((a, v)) = alias_result {
                    (*a, Some(v), ValueSource::Env)
                } else if let Some(d) = def.default_value {
                    (def.key, Some(d.to_string()), ValueSource::Default)
                } else {
                    (def.key, None, ValueSource::Missing)
                }
            };

            if def.required_always && value.is_none() {
                let alias_hint = if def.aliases.is_empty() {
                    String::new()
                } else {
                    format!(" (aliases: {})", def.aliases.join(", "))
                };
                errors.push(format!(
                    "MISSING: {} is required but not set{}",
                    def.key, alias_hint
                ));
            }

            resolved.push(ResolvedEnv {
                def,
                value: value.unwrap_or_default(),
                source,
                resolved_key,
            });
        }

        // Second pass: check production-required vars
        let app_env = resolved
            .iter()
            .find(|r| r.def.key == "APP_ENV")
            .map(|r| r.value.clone())
            .unwrap_or_else(|| "development".to_string());

        if app_env == "production" {
            for r in &resolved {
                if r.def.required_in_prod && r.source == ValueSource::Missing {
                    let alias_hint = if r.def.aliases.is_empty() {
                        String::new()
                    } else {
                        format!(" (aliases: {})", r.def.aliases.join(", "))
                    };
                    errors.push(format!(
                        "MISSING (required in production): {} is not set{}",
                        r.def.key, alias_hint
                    ));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self { resolved, app_env })
    }

    /// Look up a resolved value by its primary key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.resolved
            .iter()
            .find(|r| r.def.key == key || r.def.aliases.contains(&key))
            .map(|r| r.value.as_str())
    }

    /// Look up a resolved value and parse it to the target type.
    /// Returns `None` if the key is not found or parsing fails.
    pub fn get_parsed<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// Get a resolved env entry by primary key (for accessing metadata).
    pub fn get_resolved(&self, key: &str) -> Option<&ResolvedEnv> {
        self.resolved
            .iter()
            .find(|r| r.def.key == key || r.def.aliases.contains(&key))
    }
}

// ── Display / Startup Summary ────────────────────────────────────────────

impl EnvRegistry {
    /// Print a formatted startup summary showing all env vars and their status.
    ///
    /// Modeled after `agent-main/src/config.rs:show_summary()`.
    /// Uses `tracing::info!` for structured log output.
    pub fn show_summary(&self) {
        // Use println! for startup display (before tracing is initialized,
        // this goes to stdout; after, it's captured by the subscriber).
        // We emit via tracing::info! for structured log integration.

        tracing::info!("============================================================");
        tracing::info!("  LeeAttend API Server — Environment Configuration");
        tracing::info!("============================================================");

        // Group by category
        let categories = [
            EnvCategory::Core,
            EnvCategory::Database,
            EnvCategory::Redis,
            EnvCategory::Security,
            EnvCategory::Storage,
            EnvCategory::Worker,
            EnvCategory::RateLimit,
            EnvCategory::Scan,
            EnvCategory::Integration,
            EnvCategory::FeatureFlag,
        ];

        for category in &categories {
            let vars: Vec<&ResolvedEnv> = self
                .resolved
                .iter()
                .filter(|r| r.def.category == *category)
                .collect();

            if vars.is_empty() {
                continue;
            }

            tracing::info!("------------------------------------------------------------");
            tracing::info!("  {}", category);

            for r in &vars {
                let status_icon = match r.source {
                    ValueSource::Env => "✅",
                    ValueSource::Default => "⬜",
                    ValueSource::Missing => {
                        if r.def.required_always || r.def.required_in_prod {
                            "❌"
                        } else {
                            "⚪"
                        }
                    }
                };

                let display_value = mask_value(&r.value, r.def.sensitivity);
                let source_tag = match r.source {
                    ValueSource::Env => {
                        if r.resolved_key == r.def.key {
                            "[env]".to_string()
                        } else {
                            format!("[env → {}]", r.resolved_key)
                        }
                    }
                    ValueSource::Default => "[default]".to_string(),
                    ValueSource::Missing => "[MISSING]".to_string(),
                };

                tracing::info!(
                    "  {} {:<35} = {:<40} {}",
                    status_icon,
                    r.def.key,
                    display_value,
                    source_tag,
                );
            }
        }

        // Summary line
        let missing_required: Vec<&str> = self
            .resolved
            .iter()
            .filter(|r| {
                (r.def.required_always || (self.app_env == "production" && r.def.required_in_prod))
                    && r.source == ValueSource::Missing
            })
            .map(|r| r.def.key)
            .collect();

        if missing_required.is_empty() {
            tracing::info!("============================================================");
            tracing::info!(
                "Status: ✓ All required env vars are set (mode: {})",
                self.app_env
            );
        } else {
            tracing::warn!("============================================================");
            tracing::warn!(
                "Status: ❌ {} required env var(s) MISSING: {}",
                missing_required.len(),
                missing_required.join(", ")
            );
        }
        tracing::info!("============================================================");
    }
}

// ── Value Masking ────────────────────────────────────────────────────────

/// Mask a value for safe display based on its sensitivity classification.
fn mask_value(value: &str, sensitivity: Sensitivity) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }

    match sensitivity {
        Sensitivity::Public => {
            // Truncate long values
            if value.len() > 60 {
                format!("{}...", &value[..57])
            } else {
                value.to_string()
            }
        }
        Sensitivity::Secret => {
            format!("***SECRET ({} chars)***", value.len())
        }
        Sensitivity::Connection => mask_connection_url(value),
    }
}

/// Mask the password portion of a connection URL.
fn mask_connection_url(url: &str) -> String {
    // Pattern: scheme://user:password@host:port/db
    // Mask the password part
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            // Check that the colon is after the scheme separator
            if let Some(scheme_end) = url.find("://") {
                if colon_pos > scheme_end + 2 {
                    let before = &url[..=colon_pos];
                    let after = &url[at_pos..];
                    return format!("{before}***{after}");
                }
            }
        }
    }
    // Fallback: if we can't parse the URL, show truncated
    if url.len() > 60 {
        format!("{}...", &url[..57])
    } else {
        url.to_string()
    }
}

// ── The Declaration Table (Single Source of Truth) ───────────────────────

/// ALL environment variables consumed by apiserver.
///
/// To add a new env var:
/// 1. Add an entry to this slice.
/// 2. Use `registry.get("YOUR_KEY")` in your code instead of `std::env::var()`.
/// 3. Run tests — `.env.example` generation and startup display update
///    automatically.
#[rustfmt::skip]
pub static ALL_ENV_VARS: &[EnvVarDef] = &[
    // ═══════════════════════════════════════════════════════════════════
    // CORE
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "APP_ENV",
        aliases: &["API_ENV"],
        category: EnvCategory::Core,
        description: "Application environment (development/production).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("development"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "APP_PORT",
        aliases: &["API_PORT"],
        category: EnvCategory::Core,
        description: "HTTP server port.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("3000"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "APP_HOST",
        aliases: &["API_HOST"],
        category: EnvCategory::Core,
        description: "HTTP server bind address.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("0.0.0.0"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "PORT_FILE_NAME",
        aliases: &[],
        category: EnvCategory::Core,
        description: "File to write the actual bound port number on startup.",
        required_always: false,
        required_in_prod: false,
        default_value: Some(".api_port"),
        sensitivity: Sensitivity::Public,
    },

    // ═══════════════════════════════════════════════════════════════════
    // DATABASE
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "DB_PRIMARY_URL",
        aliases: &["DB_URL", "DATABASE_URL"],
        category: EnvCategory::Database,
        description: "Primary PostgreSQL connection URL (for migrations and direct access).",
        required_always: true,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Connection,
    },
    EnvVarDef {
        key: "DB_POOLER_TX_URL",
        aliases: &["DATABASE_POOLER_TX_URL"],
        category: EnvCategory::Database,
        description: "Supavisor transaction pooler URL (port 6543). Required — Supavisor mode is enforced.",
        required_always: true,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Connection,
    },
    EnvVarDef {
        key: "DB_POOLER_SESS_URL",
        aliases: &["DATABASE_POOLER_SESS_URL"],
        category: EnvCategory::Database,
        description: "Supavisor session pooler URL (port 5452). Required — Supavisor mode is enforced.",
        required_always: true,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Connection,
    },
    EnvVarDef {
        key: "DB_TENANT_POOLER_TX_URL",
        aliases: &["DATABASE_TENANT_POOLER_TX_URL"],
        category: EnvCategory::Database,
        description: "Tenant transaction pooler URL. Falls back to DB_POOLER_TX_URL if unset.",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Connection,
    },
    EnvVarDef {
        key: "DB_TENANT_POOLER_SESS_URL",
        aliases: &["DATABASE_TENANT_POOLER_SESS_URL"],
        category: EnvCategory::Database,
        description: "Tenant session pooler URL. Falls back to DB_POOLER_SESS_URL if unset.",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Connection,
    },
    EnvVarDef {
        key: "DB_READER_URL",
        aliases: &["DATABASE_READER_URL"],
        category: EnvCategory::Database,
        description: "Optional read-replica database URL. Falls back to pooler TX URL if unset.",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Connection,
    },
    EnvVarDef {
        key: "DB_API_MAX_CONN",
        aliases: &[],
        category: EnvCategory::Database,
        description: "API database pool max connections.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("20"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "DB_API_ACQUIRE_TIMEOUT_SECS",
        aliases: &[],
        category: EnvCategory::Database,
        description: "API pool connection acquire timeout (seconds).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("5"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "DB_WORKER_ACQUIRE_TIMEOUT_SECS",
        aliases: &[],
        category: EnvCategory::Database,
        description: "Worker pool connection acquire timeout (seconds).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("30"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "DB_TENANT_MAX_CONN",
        aliases: &[],
        category: EnvCategory::Database,
        description: "Tenant database pool max connections.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("5"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "PGBOUNCER_HOST",
        aliases: &["DATABASE_HOST"],
        category: EnvCategory::Database,
        description: "PgBouncer/Database host (used by system tenant management UI).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("localhost"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "PGBOUNCER_PORT",
        aliases: &["PGPORT"],
        category: EnvCategory::Database,
        description: "PgBouncer/Database port (used by system tenant management UI).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("5432"),
        sensitivity: Sensitivity::Public,
    },

    // ═══════════════════════════════════════════════════════════════════
    // REDIS
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "REDIS_URL",
        aliases: &[],
        category: EnvCategory::Redis,
        description: "Redis connection URL. Distributed job processing and presence are disabled if unset.",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Connection,
    },

    // ═══════════════════════════════════════════════════════════════════
    // SECURITY
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "JWT_SECRET",
        aliases: &[],
        category: EnvCategory::Security,
        description: "JWT token signing secret. Must be at least 32 bytes in production.",
        required_always: false,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Secret,
    },
    EnvVarDef {
        key: "ENCRYPTION_KEYS_JSON",
        aliases: &[],
        category: EnvCategory::Security,
        description: "JSON-encoded encryption keys for database URL encryption. Required in production.",
        required_always: false,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Secret,
    },
    EnvVarDef {
        key: "ALLOWED_ORIGINS",
        aliases: &[],
        category: EnvCategory::Security,
        description: "CORS allowed origins (comma-separated). Required in production.",
        required_always: false,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "COOKIE_DOMAIN",
        aliases: &[],
        category: EnvCategory::Security,
        description: "Cookie domain for cross-subdomain auth (e.g. .vlee.dev).",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Public,
    },

    // ═══════════════════════════════════════════════════════════════════
    // STORAGE (S3 / MinIO)
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "S3_ENDPOINT_URL",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "S3-compatible storage endpoint URL.",
        required_always: false,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "S3_BUCKET_NAME",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "S3 bucket name for file storage.",
        required_always: false,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "S3_ACCESS_KEY",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "S3 access key.",
        required_always: false,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Secret,
    },
    EnvVarDef {
        key: "S3_SECRET_KEY",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "S3 secret key.",
        required_always: false,
        required_in_prod: true,
        default_value: None,
        sensitivity: Sensitivity::Secret,
    },
    EnvVarDef {
        key: "S3_REGION",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "S3 region.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("us-east-1"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "S3_USE_PATH_STYLE",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "Use path-style S3 addressing (true/false). Default: true (MinIO compatibility).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("true"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "FILE_PROXY_THRESHOLD_BYTES",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "Files larger than this (bytes) use presigned URLs instead of proxying. Default: 1MB.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("1048576"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "PRESIGNED_URL_TTL_SECS",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "Time-to-live for presigned URLs (seconds). Default: 900 (15 min).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("900"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "STORAGE_DIR",
        aliases: &[],
        category: EnvCategory::Storage,
        description: "Local storage directory for temporary files.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("storage"),
        sensitivity: Sensitivity::Public,
    },

    // ═══════════════════════════════════════════════════════════════════
    // WORKER
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "WORKER_MAX_CONCURRENT_JOBS",
        aliases: &[],
        category: EnvCategory::Worker,
        description: "Maximum concurrent background job workers.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("5"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "WORKER_POLL_TIMEOUT_SECS",
        aliases: &[],
        category: EnvCategory::Worker,
        description: "Worker Redis poll timeout in seconds (max 300).",
        required_always: false,
        required_in_prod: false,
        default_value: Some("30"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "WORKER_MAX_RETRIES",
        aliases: &[],
        category: EnvCategory::Worker,
        description: "Maximum job retry attempts.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("3"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "WORKER_JOB_TIMEOUT_SECS",
        aliases: &[],
        category: EnvCategory::Worker,
        description: "Job execution timeout in seconds.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("300"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "IMPORT_CHUNK_SIZE",
        aliases: &[],
        category: EnvCategory::Worker,
        description: "Number of rows per chunk during employee import.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("500"),
        sensitivity: Sensitivity::Public,
    },

    // ═══════════════════════════════════════════════════════════════════
    // RATE LIMITING
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "RATE_LIMIT_MAX_READ",
        aliases: &[],
        category: EnvCategory::RateLimit,
        description: "Maximum read requests (GET/HEAD/OPTIONS) per rate limit window.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("120"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "RATE_LIMIT_MAX_WRITE",
        aliases: &[],
        category: EnvCategory::RateLimit,
        description: "Maximum write requests (POST/PUT/PATCH/DELETE) per rate limit window.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("30"),
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "RATE_LIMIT_WINDOW_SECS",
        aliases: &[],
        category: EnvCategory::RateLimit,
        description: "Rate limit window duration in seconds.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("60"),
        sensitivity: Sensitivity::Public,
    },

    // ═══════════════════════════════════════════════════════════════════
    // VIRUS SCANNING
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "CLAMD_HOST",
        aliases: &[],
        category: EnvCategory::Scan,
        description: "ClamAV daemon host. Virus scanning is disabled if unset.",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "CLAMD_PORT",
        aliases: &[],
        category: EnvCategory::Scan,
        description: "ClamAV daemon port.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("3310"),
        sensitivity: Sensitivity::Public,
    },

    // ═══════════════════════════════════════════════════════════════════
    // INTEGRATION
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "SUPAVISOR_API_URL",
        aliases: &[],
        category: EnvCategory::Integration,
        description: "Supavisor admin API URL for automatic tenant registration on boot.",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Public,
    },
    EnvVarDef {
        key: "API_JWT_SECRET",
        aliases: &["SUPAVISOR_API_JWT"],
        category: EnvCategory::Integration,
        description: "JWT secret for Supavisor API authentication. Must match Supavisor config.",
        required_always: false,
        required_in_prod: false,
        default_value: None,
        sensitivity: Sensitivity::Secret,
    },

    // ═══════════════════════════════════════════════════════════════════
    // FEATURE FLAGS
    // ═══════════════════════════════════════════════════════════════════
    EnvVarDef {
        key: "FF_NO_SHOW_TIMELINE",
        aliases: &[],
        category: EnvCategory::FeatureFlag,
        description: "Emergency kill-switch for the no-show timeline scrubber. DB flag is source of truth; this is an override only.",
        required_always: false,
        required_in_prod: false,
        default_value: Some("false"),
        sensitivity: Sensitivity::Public,
    },
];

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Helper: clear all known env vars to ensure a clean test environment.
    fn clear_all_env_vars() {
        for def in ALL_ENV_VARS {
            std::env::remove_var(def.key);
            for alias in def.aliases {
                std::env::remove_var(alias);
            }
        }
    }

    /// Helper: set minimum required env vars so resolution succeeds.
    fn set_minimum_env() {
        std::env::set_var("DB_PRIMARY_URL", "postgres://u:p@localhost:5432/db");
        std::env::set_var("DB_POOLER_TX_URL", "postgres://u:p@localhost:6543/db");
        std::env::set_var("DB_POOLER_SESS_URL", "postgres://u:p@localhost:5452/db");
    }

    #[test]
    #[serial]
    fn test_all_env_vars_have_unique_primary_keys() {
        let mut seen = std::collections::HashSet::new();
        for def in ALL_ENV_VARS {
            assert!(seen.insert(def.key), "Duplicate primary key: {}", def.key);
        }
    }

    #[test]
    #[serial]
    fn test_resolve_all_with_defaults() {
        clear_all_env_vars();
        set_minimum_env();

        let registry = EnvRegistry::resolve().expect("Should resolve with minimum env");

        // Core defaults
        assert_eq!(registry.get("APP_ENV").unwrap(), "development");
        assert_eq!(registry.get("APP_PORT").unwrap(), "3000");
        assert_eq!(registry.get("APP_HOST").unwrap(), "0.0.0.0");

        // DB required vars
        assert!(registry.get("DB_PRIMARY_URL").is_some());
        assert!(registry.get("DB_POOLER_TX_URL").is_some());
        assert!(registry.get("DB_POOLER_SESS_URL").is_some());

        // Pool tuning defaults
        assert_eq!(registry.get("DB_API_MAX_CONN").unwrap(), "20");
        assert_eq!(registry.get("DB_TENANT_MAX_CONN").unwrap(), "5");

        // S3 optional — should be missing but not required
        let s3 = registry.get_resolved("S3_ENDPOINT_URL").unwrap();
        assert_eq!(s3.source, ValueSource::Missing);

        // REDIS_URL is optional
        let redis = registry.get_resolved("REDIS_URL").unwrap();
        assert_eq!(redis.source, ValueSource::Missing);
    }

    #[test]
    #[serial]
    fn test_resolve_fails_when_required_missing() {
        clear_all_env_vars();
        // Don't set DB vars

        let result = EnvRegistry::resolve();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("DB_PRIMARY_URL")));
        assert!(errors.iter().any(|e| e.contains("DB_POOLER_TX_URL")));
    }

    #[test]
    #[serial]
    fn test_production_requires_extra_vars() {
        clear_all_env_vars();
        set_minimum_env();
        std::env::set_var("APP_ENV", "production");

        let result = EnvRegistry::resolve();
        assert!(
            result.is_err(),
            "Production should require JWT_SECRET, ALLOWED_ORIGINS, ENCRYPTION_KEYS_JSON, S3 vars"
        );
        let errors = result.unwrap_err();
        // Check that production-required vars are flagged
        assert!(
            errors.iter().any(|e| e.contains("JWT_SECRET")),
            "Expected JWT_SECRET to be flagged as missing in production. Errors: {:?}",
            errors
        );
        assert!(
            errors.iter().any(|e| e.contains("ALLOWED_ORIGINS")),
            "Expected ALLOWED_ORIGINS to be flagged. Errors: {:?}",
            errors
        );
    }

    #[test]
    #[serial]
    fn test_aliases_resolve_correctly() {
        clear_all_env_vars();
        set_minimum_env();

        // Set via alias
        std::env::set_var("API_PORT", "8080");
        std::env::remove_var("APP_PORT");

        let registry = EnvRegistry::resolve().unwrap();
        assert_eq!(registry.get("APP_PORT").unwrap(), "8080");

        let resolved = registry.get_resolved("APP_PORT").unwrap();
        assert_eq!(resolved.resolved_key, "API_PORT");
        assert_eq!(resolved.source, ValueSource::Env);
    }

    #[test]
    #[serial]
    fn test_primary_key_takes_precedence_over_alias() {
        clear_all_env_vars();
        set_minimum_env();

        std::env::set_var("APP_ENV", "staging");
        std::env::set_var("API_ENV", "development");

        let registry = EnvRegistry::resolve().unwrap();
        assert_eq!(registry.get("APP_ENV").unwrap(), "staging");
    }

    #[test]
    #[serial]
    fn test_db_url_alias_chain() {
        clear_all_env_vars();
        std::env::set_var("DATABASE_URL", "postgres://u:p@localhost:5432/db");
        std::env::set_var("DB_POOLER_TX_URL", "postgres://u:p@localhost:6543/db");
        std::env::set_var("DB_POOLER_SESS_URL", "postgres://u:p@localhost:5452/db");

        let registry = EnvRegistry::resolve().unwrap();
        // DB_PRIMARY_URL should resolve from DATABASE_URL alias
        assert_eq!(
            registry.get("DB_PRIMARY_URL").unwrap(),
            "postgres://u:p@localhost:5432/db"
        );
        let resolved = registry.get_resolved("DB_PRIMARY_URL").unwrap();
        assert_eq!(resolved.resolved_key, "DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_get_parsed_numeric() {
        clear_all_env_vars();
        set_minimum_env();
        std::env::set_var("DB_API_MAX_CONN", "42");

        let registry = EnvRegistry::resolve().unwrap();
        assert_eq!(registry.get_parsed::<u32>("DB_API_MAX_CONN"), Some(42));
        assert_eq!(registry.get_parsed::<u32>("DB_TENANT_MAX_CONN"), Some(5)); // default
        assert_eq!(registry.get_parsed::<u32>("NONEXISTENT"), None);
    }

    #[test]
    #[serial]
    fn test_get_parsed_invalid_falls_back() {
        clear_all_env_vars();
        set_minimum_env();
        std::env::set_var("DB_API_MAX_CONN", "not_a_number");

        let registry = EnvRegistry::resolve().unwrap();
        // parse fails → None
        assert_eq!(registry.get_parsed::<u32>("DB_API_MAX_CONN"), None);
    }

    #[test]
    #[serial]
    fn test_masking_secret() {
        let masked = mask_value("super-secret-jwt-key-12345", Sensitivity::Secret);
        assert!(masked.contains("SECRET"));
        assert!(masked.contains("26 chars"));
        assert!(!masked.contains("super-secret"));
    }

    #[test]
    #[serial]
    fn test_masking_connection_url() {
        let masked = mask_value(
            "postgres://admin:password123@db.example.com:5432/mydb",
            Sensitivity::Connection,
        );
        assert!(masked.contains("admin:***@"));
        assert!(!masked.contains("password123"));
        assert!(masked.contains("db.example.com"));
    }

    #[test]
    #[serial]
    fn test_masking_public_truncates_long() {
        let long_value = "a".repeat(100);
        let masked = mask_value(&long_value, Sensitivity::Public);
        assert!(masked.ends_with("..."));
        assert!(masked.len() <= 60);
    }

    #[test]
    #[serial]
    fn test_masking_empty_value() {
        assert_eq!(mask_value("", Sensitivity::Public), "<empty>");
        assert_eq!(mask_value("", Sensitivity::Secret), "<empty>");
    }

    #[test]
    #[serial]
    fn test_show_summary_does_not_panic() {
        clear_all_env_vars();
        set_minimum_env();

        let registry = EnvRegistry::resolve().unwrap();
        // Just verify it doesn't panic — output goes to tracing
        registry.show_summary();
    }

    #[test]
    #[serial]
    fn test_env_registry_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EnvRegistry>();
    }

    #[test]
    #[serial]
    fn test_value_source_tracking() {
        clear_all_env_vars();
        set_minimum_env();
        std::env::set_var("APP_PORT", "9090");
        std::env::remove_var("REDIS_URL");

        let registry = EnvRegistry::resolve().unwrap();

        // Env set
        let port = registry.get_resolved("APP_PORT").unwrap();
        assert_eq!(port.source, ValueSource::Env);
        assert_eq!(port.value, "9090");

        // Default
        let host = registry.get_resolved("APP_HOST").unwrap();
        assert_eq!(host.source, ValueSource::Default);

        // Optional missing
        let redis = registry.get_resolved("REDIS_URL").unwrap();
        assert_eq!(redis.source, ValueSource::Missing);
    }

    #[test]
    #[serial]
    fn test_supavisor_api_jwt_alias() {
        clear_all_env_vars();
        set_minimum_env();
        std::env::set_var("SUPAVISOR_API_JWT", "supa-jwt-token");

        let registry = EnvRegistry::resolve().unwrap();
        assert_eq!(registry.get("API_JWT_SECRET").unwrap(), "supa-jwt-token");
        let resolved = registry.get_resolved("API_JWT_SECRET").unwrap();
        assert_eq!(resolved.resolved_key, "SUPAVISOR_API_JWT");
    }
}
