use crate::utils::i18n::{Language, CURRENT_LOCALE};
use axum::{body::Body, http::Request, middleware::Next, response::Response};

/// Middleware to extract locale from HTTP request (query string or Accept-Language header)
/// and store it in Tokio task-local storage for context propagation.
pub async fn locale_middleware(req: Request<Body>, next: Next) -> Response {
    // 1. Try to extract from 'lang' query parameter
    let mut query_lang = None;
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                if k == "lang" {
                    if let Ok(lang) = v.parse::<Language>() {
                        query_lang = Some(lang.as_str().to_string());
                        break;
                    }
                }
            }
        }
    }

    // 2. Try to extract from Accept-Language header
    let mut header_lang = None;
    if query_lang.is_none() {
        if let Some(accept_lang) = req.headers().get(axum::http::header::ACCEPT_LANGUAGE) {
            if let Ok(accept_str) = accept_lang.to_str() {
                // Split by comma: vi-VN,vi;q=0.9,en-US;q=0.8,en;q=0.7
                for part in accept_str.split(',') {
                    let clean_part = part.split(';').next().unwrap_or("").trim();
                    if let Ok(lang) = clean_part.parse::<Language>() {
                        header_lang = Some(lang.as_str().to_string());
                        break;
                    }
                }
            }
        }
    }

    // 3. Fallback to default workspace/platform language ("vi")
    let resolved_lang = query_lang
        .or(header_lang)
        .unwrap_or_else(|| "vi".to_string());

    // Run request handlers within the task-local locale scope
    CURRENT_LOCALE
        .scope(resolved_lang, async move { next.run(req).await })
        .await
}
