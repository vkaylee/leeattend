use axum::{
    extract::Request, http::HeaderName, http::HeaderValue, middleware::Next, response::Response,
};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware to manage and propagate `x-request-id` headers for telemetry correlation.
///
/// 1. Extracts `x-request-id` from incoming request headers, fallback to a new UUID v7.
/// 2. Inserts/updates `x-request-id` in request headers for downstream consumption.
/// 3. Instruments handler execution with a tracing span containing `request_id`.
/// 4. Injects `x-request-id` into the response headers.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let header_name = HeaderName::from_static(REQUEST_ID_HEADER);

    // Extract existing or generate new UUID v7
    let request_id = request
        .headers()
        .get(&header_name)
        .and_then(|val| val.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::now_v7);

    let request_id_str = request_id.to_string();

    // Propagate request ID header downstream
    if let Ok(val) = HeaderValue::from_str(&request_id_str) {
        request.headers_mut().insert(header_name.clone(), val);
    }

    // Wrap the request in a tracing span with the correlation ID
    let span = tracing::info_span!("request", request_id = %request_id_str);

    let mut response = {
        use tracing::Instrument;
        next.run(request).instrument(span).await
    };

    // Inject request ID into response header
    if let Ok(val) = HeaderValue::from_str(&request_id_str) {
        response.headers_mut().insert(header_name, val);
    }

    response
}
