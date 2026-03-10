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
            if let Some(date_str) = parts
                .headers
                .get("X-Debug-Mock-Date")
                .and_then(|h| h.to_str().ok())
            {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return Ok(DebugContext {
                        mock_date: Some(date),
                    });
                }
            }
        }

        #[cfg(not(debug_assertions))]
        let _ = parts; // Suppress unused warning in release mode

        Ok(DebugContext { mock_date: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequestParts;
    use axum::http::Request;

    #[tokio::test]
    async fn test_debug_context_no_header() {
        let req = Request::builder().body(()).unwrap();
        let (mut parts, _body) = req.into_parts();
        let ctx = DebugContext::from_request_parts(&mut parts, &()).await.unwrap();
        assert!(ctx.mock_date.is_none());
    }

    #[tokio::test]
    async fn test_debug_context_invalid_date() {
        let req = Request::builder()
            .header("X-Debug-Mock-Date", "not-a-date")
            .body(())
            .unwrap();
        let (mut parts, _body) = req.into_parts();
        let ctx = DebugContext::from_request_parts(&mut parts, &()).await.unwrap();
        // In both debug and release, invalid date → None
        assert!(ctx.mock_date.is_none());
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn test_debug_context_valid_date_debug_mode() {
        let req = Request::builder()
            .header("X-Debug-Mock-Date", "2026-03-15")
            .body(())
            .unwrap();
        let (mut parts, _body) = req.into_parts();
        let ctx = DebugContext::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(
            ctx.mock_date,
            Some(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap())
        );
    }

    #[cfg(not(debug_assertions))]
    #[tokio::test]
    async fn test_debug_context_valid_date_release_mode() {
        let req = Request::builder()
            .header("X-Debug-Mock-Date", "2026-03-15")
            .body(())
            .unwrap();
        let (mut parts, _body) = req.into_parts();
        let ctx = DebugContext::from_request_parts(&mut parts, &()).await.unwrap();
        // In release mode, mock date is always None
        assert!(ctx.mock_date.is_none());
    }
}
