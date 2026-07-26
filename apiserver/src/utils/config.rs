use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub env: String,
    pub port: u16,
    pub host: String,
    pub port_file_name: String,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub jwt_secret: String,
    pub allowed_origins: String,
    pub encryption_keys_json: Option<String>,
}

impl AppConfig {
    /// Build configuration from the centralized env registry.
    pub fn from_registry(
        registry: &crate::utils::env_registry::EnvRegistry,
    ) -> Result<Self, String> {
        let env = registry.get("APP_ENV").unwrap_or("development").to_string();

        let port = registry
            .get_parsed::<u16>("APP_PORT")
            .ok_or_else(|| "Invalid APP_PORT".to_string())?;

        let host = registry
            .get("APP_HOST")
            .unwrap_or("0.0.0.0")
            .trim()
            .to_string();

        let port_file_name = registry
            .get("PORT_FILE_NAME")
            .unwrap_or(".api_port")
            .to_string();

        let database_url = registry
            .get("DB_PRIMARY_URL")
            .ok_or_else(|| "DATABASE_URL (or DB_PRIMARY_URL, DB_URL) is required".to_string())?
            .to_string();

        let redis_url = registry
            .get("REDIS_URL")
            .map(std::string::ToString::to_string);

        let jwt_secret = if env == "production" {
            registry
                .get("JWT_SECRET")
                .ok_or_else(|| "JWT_SECRET must be set in production".to_string())?
                .to_string()
        } else {
            registry
                .get("JWT_SECRET")
                .unwrap_or("dev_secret_key_leeattend_2026")
                .to_string()
        };

        let allowed_origins = if env == "production" {
            registry
                .get("ALLOWED_ORIGINS")
                .ok_or_else(|| "ALLOWED_ORIGINS is required in production".to_string())?
                .to_string()
        } else {
            registry
                .get("ALLOWED_ORIGINS")
                .unwrap_or("http://localhost:5173,http://localhost:3000,http://127.0.0.1:5173,http://127.0.0.1:3000")
                .to_string()
        };

        let encryption_keys_json = registry
            .get("ENCRYPTION_KEYS_JSON")
            .map(std::string::ToString::to_string);
        if env == "production" && encryption_keys_json.is_none() {
            return Err("ENCRYPTION_KEYS_JSON must be set in production".to_string());
        }

        Ok(Self {
            env,
            port,
            host,
            port_file_name,
            database_url,
            redis_url,
            jwt_secret,
            allowed_origins,
            encryption_keys_json,
        })
    }

    /// Legacy: Build configuration from raw environment variables.
    /// Prefer [`AppConfig::from_registry`] for new code.
    pub fn from_env() -> Result<Self, String> {
        let env = std::env::var("APP_ENV")
            .or_else(|_| std::env::var("API_ENV"))
            .unwrap_or_else(|_| "development".to_string());

        let port_str = std::env::var("APP_PORT")
            .or_else(|_| std::env::var("API_PORT"))
            .unwrap_or_else(|_| "3000".to_string());
        let port = port_str
            .parse::<u16>()
            .map_err(|e| format!("Invalid APP_PORT / API_PORT: {e}"))?;

        let host = std::env::var("APP_HOST")
            .or_else(|_| std::env::var("API_HOST"))
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let port_file_name =
            std::env::var("PORT_FILE_NAME").unwrap_or_else(|_| ".api_port".to_string());

        let database_url = std::env::var("DATABASE_URL")
            .or_else(|_err| std::env::var("DATABASE_POOLER_TX_URL"))
            .or_else(|_err| std::env::var("DB_PRIMARY_URL"))
            .or_else(|_err| std::env::var("DB_URL"))
            .map_err(|_err| {
                "DATABASE_URL (or DATABASE_POOLER_TX_URL, DB_PRIMARY_URL, DB_URL) is required"
            })?;

        let redis_url = std::env::var("REDIS_URL").ok();

        let jwt_secret = if env == "production" {
            std::env::var("JWT_SECRET").map_err(|_err| "JWT_SECRET must be set in production")?
        } else {
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_err| "dev_secret_key_leeattend_2026".to_string())
        };

        let allowed_origins = if env == "production" {
            std::env::var("ALLOWED_ORIGINS")
                .map_err(|_err| "ALLOWED_ORIGINS is required in production")?
        } else {
            std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000,http://127.0.0.1:5173,http://127.0.0.1:3000".to_string())
        };

        let encryption_keys_json = std::env::var("ENCRYPTION_KEYS_JSON").ok();
        if env == "production" && encryption_keys_json.is_none() {
            return Err("ENCRYPTION_KEYS_JSON must be set in production".to_string());
        }

        Ok(Self {
            env,
            port,
            host,
            port_file_name,
            database_url,
            redis_url,
            jwt_secret,
            allowed_origins,
            encryption_keys_json,
        })
    }
}

pub static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[allow(clippy::panic)]
pub fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(|| {
        AppConfig::from_env().unwrap_or_else(|err| {
            panic!("FATAL: Configuration validation failed: {err}");
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_app_config_from_env_dev() {
        std::env::set_var("APP_ENV", "development");
        std::env::set_var("DATABASE_URL", "postgres://localhost/db");
        std::env::set_var("REDIS_URL", "redis://localhost");
        std::env::set_var("ALLOWED_ORIGINS", "http://localhost:3000");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.env, "development");
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.redis_url.as_deref(), Some("redis://localhost"));
        assert_eq!(config.allowed_origins, "http://localhost:3000");
    }

    #[test]
    #[serial]
    fn test_app_config_production_missing_jwt() {
        std::env::set_var("APP_ENV", "production");
        std::env::set_var("DATABASE_URL", "postgres://localhost/db");
        std::env::set_var("REDIS_URL", "redis://localhost");
        std::env::set_var("ALLOWED_ORIGINS", "http://localhost:3000");
        std::env::remove_var("JWT_SECRET");
        std::env::remove_var("ENCRYPTION_KEYS_JSON");

        let config_res = AppConfig::from_env();
        assert!(config_res.is_err());
    }
}
