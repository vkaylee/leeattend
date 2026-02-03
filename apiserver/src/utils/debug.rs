use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use chrono::NaiveDate;

pub struct DebugContext {
    pub mock_date: Option<NaiveDate>,
}

#[async_trait]
impl<S> FromRequestParts<S> for DebugContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        #[cfg(debug_assertions)]
        {
            if let Some(date_str) = parts.headers.get("X-Debug-Mock-Date").and_then(|h| h.to_str().ok()) {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(DebugContext { mock_date: Some(date) });
                }
            }
        }

        #[cfg(not(debug_assertions))]
        let _ = parts; // Suppress unused warning in release mode

        Ok(DebugContext { mock_date: None })
    }
}
