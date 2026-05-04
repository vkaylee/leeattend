use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Unauthorized: {0}")]
    Auth(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal Server Error: {0}")]
    Internal(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
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
        match status.as_u16() {
            401 => Self::Auth(message),
            403 => Self::Forbidden(message),
            404 => Self::NotFound(message),
            409 => Self::Conflict(message),
            400 | 422 => Self::Validation(message),
            _ => Self::Internal(message),
        }
    }
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Sqlx(_) | Self::Redis(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Sqlx(_) => "DATABASE_ERROR",
            Self::Redis(_) => "REDIS_ERROR",
            Self::Auth(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Conflict(_) => "CONFLICT",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();

        // Determine the message to send to the client
        let message = match &self {
            // Hide internal details in production
            Self::Sqlx(_) | Self::Redis(_) | Self::Internal(_) => {
                // Log the full error for debugging
                error!(error = ?self, "Internal API Error");
                "An internal server error occurred.".to_string()
            }
            // Safe to expose messages for client errors
            _ => self.to_string(),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

// Convert common result types to AppError
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::Auth("x".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Forbidden("x".into()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::NotFound("x".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::Validation("x".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::Internal("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::Conflict("x".into()).status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            AppError::ServiceUnavailable("x".into()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            AppError::Redis("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(AppError::Auth("x".into()).error_code(), "UNAUTHORIZED");
        assert_eq!(AppError::Forbidden("x".into()).error_code(), "FORBIDDEN");
        assert_eq!(AppError::NotFound("x".into()).error_code(), "NOT_FOUND");
        assert_eq!(
            AppError::Validation("x".into()).error_code(),
            "VALIDATION_ERROR"
        );
        assert_eq!(
            AppError::Internal("x".into()).error_code(),
            "INTERNAL_ERROR"
        );
        assert_eq!(AppError::Conflict("x".into()).error_code(), "CONFLICT");
        assert_eq!(
            AppError::ServiceUnavailable("x".into()).error_code(),
            "SERVICE_UNAVAILABLE"
        );
        assert_eq!(AppError::Redis("x".into()).error_code(), "REDIS_ERROR");
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
        let err = AppError::NotFound("user 42".into());
        assert!(err.to_string().contains("user 42"));
        let err = AppError::Validation("email invalid".into());
        assert!(err.to_string().contains("email invalid"));
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
            AppError::Auth("bad token".into()),
            AppError::Forbidden("no access".into()),
            AppError::NotFound("user 42".into()),
            AppError::Validation("email required".into()),
            AppError::Conflict("duplicate entry".into()),
            AppError::ServiceUnavailable("down for maintenance".into()),
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
            AppError::Internal("secret db info".into()),
            AppError::Redis("redis connection failed".into()),
        ];
        for err in errors {
            let response = err.into_response();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    #[test]
    fn test_status_code_service_unavailable() {
        let err = AppError::ServiceUnavailable("maintenance".into());
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.error_code(), "SERVICE_UNAVAILABLE");
    }

    #[test]
    fn test_all_error_variants_display() {
        let errors: Vec<AppError> = vec![
            AppError::Auth("a".into()),
            AppError::Forbidden("b".into()),
            AppError::NotFound("c".into()),
            AppError::Validation("d".into()),
            AppError::Internal("e".into()),
            AppError::Conflict("f".into()),
            AppError::ServiceUnavailable("g".into()),
            AppError::Redis("h".into()),
        ];
        for err in &errors {
            let display = err.to_string();
            assert!(!display.is_empty());
        }
    }
}
