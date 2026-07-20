use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

use crate::utils::i18n::I18nKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalizedMessage {
    Key(I18nKey),
    KeyWithArgs(I18nKey, Vec<(String, String)>),
}

impl std::fmt::Display for LocalizedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(k) => write!(f, "{}", k.as_str()),
            Self::KeyWithArgs(k, args) => {
                let args_str = args
                    .iter()
                    .map(|(name, val)| format!("{name}={val}"))
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "{}:{}", k.as_str(), args_str)
            }
        }
    }
}

impl LocalizedMessage {
    pub fn translate(&self, lang: &str) -> String {
        match self {
            Self::Key(k) => crate::utils::i18n::translate(lang, k.as_str()),
            Self::KeyWithArgs(k, args) => {
                let borrowed_args: Vec<(&str, &str)> =
                    args.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                crate::utils::i18n::translate_args(lang, k.as_str(), &borrowed_args)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Diesel(#[from] diesel::result::Error),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Unauthorized: {0}")]
    Auth(LocalizedMessage),

    #[error("Forbidden: {0}")]
    Forbidden(LocalizedMessage),

    #[error("Not Found: {0}")]
    NotFound(LocalizedMessage),

    #[error("Validation error: {0}")]
    Validation(LocalizedMessage),

    #[error("Internal Server Error: {0}")]
    Internal(String),

    #[error("Conflict: {0}")]
    Conflict(LocalizedMessage, Option<serde_json::Value>),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(LocalizedMessage),

    #[error("Background Job Error: {0}")]
    BackgroundJobError(String),

    #[error("Template Error: {0}")]
    TemplateError(String),
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::Internal(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::Internal(s.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        Self::Redis(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<rust_xlsxwriter::XlsxError> for AppError {
    fn from(err: rust_xlsxwriter::XlsxError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<(StatusCode, String)> for AppError {
    fn from((status, message): (StatusCode, String)) -> Self {
        let msg = LocalizedMessage::KeyWithArgs(
            I18nKey::Validation,
            vec![("details".to_string(), message)],
        );
        match status.as_u16() {
            401 => Self::Auth(msg),
            403 => Self::Forbidden(msg),
            404 => Self::NotFound(msg),
            409 => Self::Conflict(msg, None),
            400 | 422 => Self::Validation(msg),
            _ => Self::Internal(msg.to_string()),
        }
    }
}

impl AppError {
    pub fn auth(key: I18nKey) -> Self {
        Self::Auth(LocalizedMessage::Key(key))
    }

    pub fn forbidden(key: I18nKey) -> Self {
        Self::Forbidden(LocalizedMessage::Key(key))
    }

    pub fn not_found(key: I18nKey) -> Self {
        Self::NotFound(LocalizedMessage::Key(key))
    }

    pub fn validation(key: I18nKey) -> Self {
        Self::Validation(LocalizedMessage::Key(key))
    }

    pub fn conflict(key: I18nKey) -> Self {
        Self::Conflict(LocalizedMessage::Key(key), None)
    }

    pub fn service_unavailable(key: I18nKey) -> Self {
        Self::ServiceUnavailable(LocalizedMessage::Key(key))
    }

    pub fn auth_args<I, K, V>(key: I18nKey, args: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::Auth(LocalizedMessage::KeyWithArgs(
            key,
            args.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        ))
    }

    pub fn forbidden_args<I, K, V>(key: I18nKey, args: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::Forbidden(LocalizedMessage::KeyWithArgs(
            key,
            args.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        ))
    }

    pub fn not_found_args<I, K, V>(key: I18nKey, args: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::NotFound(LocalizedMessage::KeyWithArgs(
            key,
            args.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        ))
    }

    pub fn validation_args<I, K, V>(key: I18nKey, args: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::Validation(LocalizedMessage::KeyWithArgs(
            key,
            args.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        ))
    }

    pub fn conflict_args<I, K, V>(key: I18nKey, args: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::Conflict(
            LocalizedMessage::KeyWithArgs(
                key,
                args.into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            ),
            None,
        )
    }

    /// Conflict that carries a structured `retry` hint for the frontend
    /// (e.g. a retired master/reference row the client can re-activate).
    pub fn conflict_with_retry<I, K, V>(key: I18nKey, args: I, retry: serde_json::Value) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::Conflict(
            LocalizedMessage::KeyWithArgs(
                key,
                args.into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            ),
            Some(retry),
        )
    }

    pub fn service_unavailable_args<I, K, V>(key: I18nKey, args: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::ServiceUnavailable(LocalizedMessage::KeyWithArgs(
            key,
            args.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        ))
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Diesel(_)
            | Self::Redis(_)
            | Self::Internal(_)
            | Self::BackgroundJobError(_)
            | Self::TemplateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_, _) => StatusCode::CONFLICT,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Diesel(_) => "DATABASE_ERROR",
            Self::Redis(_) => "REDIS_ERROR",
            Self::Auth(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Conflict(_, _) => "CONFLICT",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::BackgroundJobError(_) => "BACKGROUND_JOB_ERROR",
            Self::TemplateError(_) => "TEMPLATE_ERROR",
        }
    }

    pub fn translate(&self, lang: &str) -> String {
        match self {
            Self::Diesel(_)
            | Self::Redis(_)
            | Self::Internal(_)
            | Self::BackgroundJobError(_)
            | Self::TemplateError(_) => crate::utils::i18n::translate(lang, "error.internal"),
            Self::Auth(msg)
            | Self::Forbidden(msg)
            | Self::NotFound(msg)
            | Self::Conflict(msg, _)
            | Self::ServiceUnavailable(msg) => match msg {
                LocalizedMessage::KeyWithArgs(k, args) => {
                    let prefix = crate::utils::i18n::translate(lang, k.as_str());
                    if let Some((_, val)) = args.iter().find(|(name, _)| name == "details") {
                        if prefix == k.as_str() || prefix.is_empty() {
                            val.clone()
                        } else {
                            format!("{prefix}: {val}")
                        }
                    } else {
                        msg.translate(lang)
                    }
                }
                LocalizedMessage::Key(_) => msg.translate(lang),
            },
            Self::Validation(msg) => match msg {
                LocalizedMessage::KeyWithArgs(k, args) => {
                    let prefix = crate::utils::i18n::translate(lang, k.as_str());
                    if let Some((_, val)) = args.iter().find(|(name, _)| name == "details") {
                        if prefix == k.as_str() || prefix.is_empty() {
                            val.clone()
                        } else {
                            format!("{prefix}: {val}")
                        }
                    } else {
                        msg.translate(lang)
                    }
                }
                LocalizedMessage::Key(_) => msg.translate(lang),
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();

        // Log the full error for internal errors
        match &self {
            Self::Diesel(_)
            | Self::Redis(_)
            | Self::Internal(_)
            | Self::BackgroundJobError(_)
            | Self::TemplateError(_) => {
                error!(error = ?self, "Internal API Error");
            }
            _ => {}
        }

        let lang = crate::utils::i18n::get_locale();
        let message = self.translate(&lang);

        let retry = match &self {
            Self::Conflict(_, retry) => retry.as_ref(),
            _ => None,
        };

        let body = if let Some(retry) = retry {
            Json(json!({
                "error": {
                    "code": code,
                    "message": message,
                    "retry": retry
                }
            }))
        } else {
            Json(json!({
                "error": {
                    "code": code,
                    "message": message
                }
            }))
        };

        (status, body).into_response()
    }
}

// Convert common result types to AppError
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::auth(I18nKey::InvalidToken).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::forbidden(I18nKey::Forbidden).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::not_found(I18nKey::NotFound).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::validation(I18nKey::Validation).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Internal("x".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::conflict(I18nKey::Conflict).status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            AppError::service_unavailable(I18nKey::ServiceUnavailable).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            AppError::Redis("x".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            AppError::auth(I18nKey::InvalidToken).error_code(),
            "UNAUTHORIZED"
        );
        assert_eq!(
            AppError::forbidden(I18nKey::Forbidden).error_code(),
            "FORBIDDEN"
        );
        assert_eq!(
            AppError::not_found(I18nKey::NotFound).error_code(),
            "NOT_FOUND"
        );
        assert_eq!(
            AppError::validation(I18nKey::Validation).error_code(),
            "VALIDATION_ERROR"
        );
        assert_eq!(
            AppError::Internal("x".to_string()).error_code(),
            "INTERNAL_ERROR"
        );
        assert_eq!(
            AppError::conflict(I18nKey::Conflict).error_code(),
            "CONFLICT"
        );
        assert_eq!(
            AppError::service_unavailable(I18nKey::ServiceUnavailable).error_code(),
            "SERVICE_UNAVAILABLE"
        );
        assert_eq!(AppError::Redis("x".to_string()).error_code(), "REDIS_ERROR");
    }

    #[test]
    fn test_from_string() {
        let err = AppError::from("oops".to_string());
        assert!(matches!(err, AppError::Internal(_)));
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_from_str_slice() {
        let err = AppError::from("bad things");
        assert!(matches!(err, AppError::Internal(_)));
    }

    #[test]
    fn test_from_status_code_tuple() {
        let cases: &[(u16, &str)] = &[
            (401, "Unauthorized"),
            (403, "Forbidden"),
            (404, "Not found"),
            (409, "Conflict"),
            (400, "Bad request"),
            (500, "Internal"),
        ];
        let expected_variants: &[&str] = &[
            "Auth",
            "Forbidden",
            "NotFound",
            "Conflict",
            "Validation",
            "Internal",
        ];
        for (&(code, msg), &variant) in cases.iter().zip(expected_variants.iter()) {
            let sc = StatusCode::from_u16(code).expect("valid status code");
            let err = AppError::from((sc, msg.to_string()));
            let name = format!("{err:?}");
            assert!(name.starts_with(variant), "Expected {variant}, got {name}");
        }
    }

    #[test]
    fn test_display_shows_message() {
        let err = AppError::not_found(I18nKey::UserNotFound);
        assert!(err.to_string().contains("error.user_not_found"));
        let err = AppError::validation(I18nKey::InvalidStatus);
        assert!(err.to_string().contains("error.invalid_status"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::Internal(_)));
        assert_eq!(app_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_from_status_code_422() {
        let err = AppError::from((StatusCode::UNPROCESSABLE_ENTITY, "invalid data".to_string()));
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn test_into_response_for_client_errors() {
        // Client errors should expose the message
        let errors = vec![
            AppError::auth(I18nKey::InvalidToken),
            AppError::forbidden(I18nKey::Forbidden),
            AppError::not_found(I18nKey::NotFound),
            AppError::validation(I18nKey::Validation),
            AppError::conflict(I18nKey::Conflict),
            AppError::service_unavailable(I18nKey::ServiceUnavailable),
        ];
        for err in errors {
            let response = err.into_response();
            assert!(response.status().is_client_error() || response.status().is_server_error());
        }
    }

    #[test]
    fn test_into_response_for_internal_errors() {
        // Internal errors should NOT expose the actual message
        let errors = vec![
            AppError::Internal("secret db info".to_string()),
            AppError::Redis("redis connection failed".to_string()),
        ];
        for err in errors {
            let response = err.into_response();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    #[test]
    fn test_status_code_service_unavailable() {
        let err = AppError::service_unavailable(I18nKey::ServiceUnavailable);
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.error_code(), "SERVICE_UNAVAILABLE");
    }

    #[test]
    fn test_all_error_variants_display() {
        let errors: Vec<AppError> = vec![
            AppError::auth(I18nKey::InvalidToken),
            AppError::forbidden(I18nKey::Forbidden),
            AppError::not_found(I18nKey::NotFound),
            AppError::validation(I18nKey::Validation),
            AppError::Internal("e".to_string()),
            AppError::conflict(I18nKey::Conflict),
            AppError::service_unavailable(I18nKey::ServiceUnavailable),
            AppError::Redis("h".to_string()),
        ];
        for err in &errors {
            let display = err.to_string();
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_i18n_key_errors() {
        let err = AppError::not_found(I18nKey::UserNotFound);
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.error_code(), "NOT_FOUND");
        assert_eq!(err.to_string(), "Not Found: error.user_not_found");

        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
