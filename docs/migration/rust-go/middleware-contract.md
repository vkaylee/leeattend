# Middleware and Extractor Contract — Rust apiserver Inventory

Scope: Middleware pipeline, route layers, extractor traits, and rejection behavior in `apiserver/src`.  
Evidence for Ledger Task: **P1-02**.

---

## 1. Ordering table

The table below specifies the **exact execution order** of the HTTP middleware pipeline in `apiserver`, evaluated from incoming HTTP wire request (**outermost**) to business handler execution (**innermost**), and returning up the stack to the HTTP wire response.

### Architectural note on Axum / Tower registration order
In Axum v0.7 (`axum/src/docs/middleware.md:63-116`), calling `.layer(A).layer(B)` wraps the inner service with `A` first, then wraps that with `B`. The request hits `B` (outermost, registered last) before reaching `A` (innermost, registered first).

In `apiserver/src/lib.rs:351-366`:
```rust
Router::new()
    .nest("/api/v1", router)                                       // Line 352
    .layer(axum::extract::DefaultBodyLimit::max(max_body_bytes))   // Line 353 (L1 - innermost global)
    .layer(cors)                                                    // Line 354 (L2)
    .layer(tower_http::trace::TraceLayer::new_for_http())           // Line 355 (L3)
    .layer(axum::middleware::from_fn(crate::mw::i18n::locale_middleware)) // Line 356 (L4)
    .layer(crate::mw::RateLimitLayer::new(...))                     // Line 359 (L5)
    .layer(axum::middleware::from_fn(crate::mw::request_id_middleware))    // Line 364 (L6 - outermost global)
    .with_state(state)                                              // Line 365
```

### Complete request processing pipeline (outermost → innermost)

| Step | Scope / Stage | Layer / Extractor Name | Rust Symbol & Registration Site | Execution Type | Summary |
|---|---|---|---|---|---|
| **1** | Global | Request ID | `crate::mw::request_id_middleware` (`apiserver/src/lib.rs:364`, `apiserver/src/mw/request_id.rs:14-49`) | Middleware (`from_fn`) | Extracts or generates `x-request-id` (UUIDv7); binds `CURRENT_TRACE_CONTEXT` task-local and `tracing::info_span!("request")`; injects `x-request-id` into response. |
| **2** | Global | Rate Limiting (Global) | `crate::mw::RateLimitLayer` (`apiserver/src/lib.rs:359-363`, `apiserver/src/mw/rate_limit.rs:83-227`) | Tower Layer & Service | Checks Redis bucket counter (`rate:{identity}:{tier}:{path}`). Skips `/health` and `/health/ready`. Rejects with 429 and `Retry-After`. Falls back to in-memory Moka cache if Redis unavailable. |
| **3** | Global | Locale / i18n | `crate::mw::i18n::locale_middleware` (`apiserver/src/lib.rs:356-358`, `apiserver/src/mw/i18n.rs:6-49`) | Middleware (`from_fn`) | Inspects `?lang=` query parameter, then `Accept-Language` header; falls back to `"vi"`. Binds `CURRENT_LOCALE` Tokio task-local storage. |
| **4** | Global | HTTP Tracing | `tower_http::trace::TraceLayer::new_for_http()` (`apiserver/src/lib.rs:355`) | Tower Layer & Service | Creates default HTTP request span at `DEBUG` level; logs response status and latency on completion. Uses `SpanContextLayer` (`apiserver/src/utils/logging.rs:46-119`) for JSON logs. |
| **5** | Global | CORS | `tower_http::cors::CorsLayer` (`apiserver/src/lib.rs:354`, defined `apiserver/src/lib.rs:286-305`) | Tower Layer & Service | Short-circuits `OPTIONS` preflight with CORS headers; intercepts responses to attach `access-control-allow-origin`, `vary: origin`, and credentials headers. |
| **6** | Global | Body Limit (Global Config) | `axum::extract::DefaultBodyLimit::max(max_body_bytes)` (`apiserver/src/lib.rs:353`) | Tower Layer & Service | Configures default body limit (default 2MB = 2,097,152 bytes) into request extensions (`DefaultBodyLimitKind::Limit(max_body_bytes)`). Enforced when streaming/buffering body. |
| **7** | Route-level | Auth Route Rate Limiting | `crate::mw::RateLimitLayer` (`apiserver/src/routes/auth.rs:17-21`) | Route Layer (`route_layer`) | Dedicated tier applied only to `POST /auth/login` and `POST /auth/register`. Routes declared after line 21 (`/auth/forgot-password`, `/auth/refresh`, etc.) bypass this route layer. |
| **8** | Route-level | Route Body Limit Overrides | `axum::extract::DefaultBodyLimit::max(...)` (`attendance.rs:31`, `files.rs:27`, `employees.rs:23`, `leaves.rs:23`) | Route Layer (`route_layer`) | Overrides Step 6 limit in request extensions: 20MB for `/files/*` (`MAX_FILE_UPLOAD_BODY_BYTES`), 10MB for bulk uploads (`MAX_BULK_UPLOAD_BODY_BYTES`). |
| **9** | Route-level | WebSocket Upgrade Auth | `handlers::report_websocket::ws_auth_middleware` (`apiserver/src/routes/reports.rs:38-43`, `handlers/report_websocket.rs:33-68`) | Method Layer (`middleware::from_fn_with_state`) | Applied specifically to `GET /ws/reports/:job_id`. Validates JWT from `?token=` query param or `access_token` cookie before HTTP connection upgrade. |
| **10** | Handler Extractor | System / Debug Header | `DebugContext` (`apiserver/src/utils/debug.rs:13-40`) | `FromRequestParts` | Debug builds only (`#[cfg(debug_assertions)]`): reads `X-Debug-Mock-Date`. Release mode is no-op. |
| **11** | Handler Extractor | Route Parameters (Path) | `axum::extract::Path<T>` (e.g. `apiserver/src/handlers/files.rs:2`) | `FromRequestParts` | Deserializes route captures (e.g. `ws_id`, `job_id`, `file_id`). Rejects with 400 Bad Request if URL segment cannot be parsed. |
| **12** | Handler Extractor | Query Parameters (Query) | `axum::extract::Query<T>` (e.g. `apiserver/src/handlers/shifts.rs:172`) | `FromRequestParts` | Deserializes URL query string into strongly typed filter/pagination struct. Rejects with 400 Bad Request on serde deserialize error. |
| **13** | Handler Extractor | Identity Authentication | `UserId` (`apiserver/src/lib.rs:120,159-257`) | `FromRequestParts` | Validates JWT from `Authorization: Bearer <token>`, `cookie: access_token=<token>`, or query param `?access_token=<token>`. Checks Redis revocation blocklists (`jwt:revoked:{sig}` and `user:revoked_at:{user_id}`). In debug assertions, falls back to `X-Debug-Impersonate-Id` or `X-User-Id`. |
| **14** | Handler Extractor | Workspace Scope Guard | `WorkspaceId` (`apiserver/src/handlers/common.rs:24,79-111`) | `FromRequestParts` | Extracts `ws_id` from URL path map; falls back to `X-Workspace-Id` header. Rejects with 400 (`error.missing_workspace` / `error.invalid_workspace_id`). |
| **15** | Handler Extractor | Routing API Boundary Guard | `ApiContext` (`apiserver/src/handlers/common.rs:49-75`) | `FromRequestParts` | Dispatches to `ApiContext::Tenant(WorkspaceId)` or `ApiContext::System` based on whether request matches `/system` or `/api/v1/system`. Rejects with 400 `error.missing_workspace` if tenant route lacks workspace. |
| **16** | Handler Extractor | Tenant Membership & RBAC Cache | `MemberContext` (`apiserver/src/handlers/common.rs:114,198-252`) | `FromRequestParts` | Depends on `UserId` + `ApiContext`. Rejects with 403 `error.system_route_tenant_context` if used on `/system` route. Queries Redis cache `cache:auth:permissions:{workspace_id}:{user_id}` (1h TTL) or DB `find_primary_role`, `find_extra_roles`, `get_permissions_for_roles`, `find_extra_permissions`, `get_managed_departments`. Injects authorized `TenantAuthToken`. |
| **17** | Handler Extractor | System Admin Roles Context | `SystemContext` (`apiserver/src/handlers/system_auth.rs:44,117-161`) | `FromRequestParts` | Depends on `UserId`. Queries Redis cache `cache:system:roles:{user_id}` (30m TTL) or DB `get_system_roles_for_user`. Holds set of `SystemRoleType` (`PlatformAdmin`, `Support`, `BillingAdmin`). |
| **18** | Handler Extractor | Employee Self-Service Context | `EmployeeContext` (`apiserver/src/handlers/common.rs:374,382-415`) | `FromRequestParts` | Runs `MemberContext` extractor first (verifying membership and creating DB connection token); then queries employee record via `get_employee_id_by_user_id`. Rejects with 403 `error.no_employee_record` if user is not an employee in this workspace. |
| **19** | Handler Extractor | Request Body (Terminal) | `axum::Json<T>` or `axum::extract::Multipart` (e.g. `apiserver/src/handlers/auth.rs:34`) | `FromRequest` | MUST be the final extractor argument (consumes `Request<Body>`). Enforces Step 6/Step 8 body limits. Rejects with 400, 413, or 422 if invalid JSON, invalid boundary, or payload exceeds limit. |
| **20** | In-Handler Guard | Declarative RBAC Check | `ctx.require_permission(Permission::...)` / `ctx.require_scoped_permission(...)` | Handler Method Call | Checks `MemberContext.permissions` or department scopes. Rejects with 403 `error.forbidden` via `AppError::Forbidden`. |
| **21** | In-Handler Guard | System Role Check | `ctx.require_role(SystemRoleType::PlatformAdmin)` | Handler Method Call | Checks `SystemContext.roles`. Rejects with 403 `error.forbidden` via `AppError::Forbidden`. |
| **22** | In-Handler / Service | Audit Logging Event | `AuditHelper::log(...)` (`apiserver/src/services/audit_service.rs:24-69`) | Service Helper Call | Creates and inserts `AuditLog` into DB asynchronously. Does not interrupt request flow on persistence error (fail-open for audit logging). |

---

## 2. Detailed layer specifications

### 2.1 Request ID (`request_id_middleware`)
- **Rust symbol & source**: `apiserver::mw::request_id::request_id_middleware` (`apiserver/src/mw/request_id.rs:14-49`).
- **Registration**: `apiserver/src/lib.rs:364` via `axum::middleware::from_fn(crate::mw::request_id_middleware)`.
- **Inputs read**: Incoming `x-request-id` header (`HeaderName::from_static("x-request-id")`).
- **Outputs / inserted extensions**:
  - Validates format via `Uuid::parse_str`; if absent or invalid UUID, generates `Uuid::now_v7()`.
  - Mutates `request.headers_mut()` to ensure `x-request-id` header is set downstream.
  - Injects `x-request-id` into `response.headers_mut()`.
  - Sets Tokio task-local `crate::utils::logging::CURRENT_TRACE_CONTEXT` with the request ID string (`apiserver/src/mw/request_id.rs:36-41`).
  - Creates and instruments an execution tracing span: `tracing::info_span!("request", request_id = %request_id_str)`.
- **Short-circuit rejection**: None. Always passes to `next.run(request)`.
- **Ordering constraints**: Must be the outermost middleware (Step 1). All downstream layers (rate limiting, logging, panic recovery, error translation) require `x-request-id` in logs and HTTP headers.

---

### 2.2 Rate limit (`RateLimitLayer`)
- **Rust symbol & source**: `apiserver::mw::rate_limit::RateLimitLayer`, `RateLimitService`, `RateLimitConfig` (`apiserver/src/mw/rate_limit.rs:23-227`).
- **Registration**:
  - Global layer: `apiserver/src/lib.rs:359-363`.
  - Route layer: `apiserver/src/routes/auth.rs:17-21` (applied to `/auth/login` and `/auth/register`).
- **Inputs read**:
  - Authorization: inspects `Authorization: Bearer <token>` header; decodes claims with `AuthService::verify_token(token, jwt_secret)` (pure cryptographic HMAC-SHA256, no DB call) to extract `user_id` (`mw/rate_limit.rs:237-247`).
  - Debug header: `#[cfg(debug_assertions)]` checks `x-user-id` (`mw/rate_limit.rs:250-257`).
  - Client IP: extracts `ConnectInfo<SocketAddr>` from request extensions; falls back to first IP in `x-forwarded-for` header; falls back to `"unknown"` (`mw/rate_limit.rs:260-274`).
  - Matched path: reads `req.extensions().get::<axum::extract::MatchedPath>()`; falls back to `req.uri().path()` (`mw/rate_limit.rs:161-165`).
  - HTTP method: classifies method into read tier (`r`: GET, HEAD, OPTIONS) or write tier (`w`: POST, PUT, PATCH, DELETE) (`mw/rate_limit.rs:68-73`).
- **Configuration & Defaults**:
  - Read limit: `RATE_LIMIT_MAX_READ` (default `120` req/window).
  - Write limit: `RATE_LIMIT_MAX_WRITE` (default `30` req/window).
  - Window size: `RATE_LIMIT_WINDOW_SECS` (default `60` seconds).
- **Outputs / inserted extensions**: None.
- **Short-circuit rejection**:
  - Health check bypass: requests matching `/health` or `/health/ready` bypass rate limiting completely (`mw/rate_limit.rs:168-170`).
  - When allowed requests exceeded:
    - HTTP Status: `429 Too Many Requests` (`StatusCode::TOO_MANY_REQUESTS`).
    - Response Headers: `Retry-After: <window_secs>` (`mw/rate_limit.rs:214`).
    - Response Body (JSON):
      ```json
      {
        "error": {
          "code": "RATE_LIMITED",
          "message": "Too many requests. Please try again later."
        }
      }
      ```
- **Fallback policy**: When Redis is `None` or Redis script execution fails, logs warning/error and falls back to an in-memory Moka cache (`max_capacity(10000)`, TTL `window_secs`) (`mw/rate_limit.rs:178-199`).
- **Ordering constraints**:
  - Must run after Step 1 (`request_id_middleware`) so 429 responses carry `x-request-id`.
  - Must run before expensive JSON body reading and database lookups to prevent DoS.
  - Notice precedence: Rate limiting runs *before* authentication extractors (`UserId`), but performs an opportunistic, lightweight JWT signature verification inside `resolve_identity` to group rate limits by user ID rather than IP.

---

### 2.3 Locale / i18n (`locale_middleware`)
- **Rust symbol & source**: `apiserver::mw::i18n::locale_middleware` (`apiserver/src/mw/i18n.rs:6-49`), `apiserver::utils::i18n::{CURRENT_LOCALE, Language}` (`apiserver/src/utils/i18n.rs:223-256`).
- **Registration**: `apiserver/src/lib.rs:356-358` via `axum::middleware::from_fn(crate::mw::i18n::locale_middleware)`.
- **Inputs read**:
  1. `?lang=` query parameter from `req.uri().query()`.
  2. `Accept-Language` HTTP header, split on `,` and `;` (e.g. `vi-VN,vi;q=0.9,en-US;q=0.8`).
  3. Default fallback: `"vi"` (Vietnamese).
- **Outputs / inserted extensions**:
  - Binds Tokio task-local `CURRENT_LOCALE` with the resolved language code (`"vi"`, `"en"`, or `"zh"`).
  - Used downstream by `AppError::translate` (`apiserver/src/errors.rs:379-381`) to localize error response messages.
- **Short-circuit rejection**: None. Always passes to `next.run(req)`.
- **Ordering constraints**: Must wrap all layers and extractors that can return an `AppError` so that any downstream rejection message (such as 400 Bad Request, 401 Unauthorized, 403 Forbidden) is properly localized.

---

### 2.4 Tracing (`TraceLayer`)
- **Rust symbol & source**: `tower_http::trace::TraceLayer::new_for_http()` (`tower-http 0.6.8`, `apiserver/src/lib.rs:355`).
- **Registration**: `apiserver/src/lib.rs:355`.
- **Inputs read**: HTTP request method, URI, headers.
- **Outputs / inserted extensions**:
  - Creates a tracing span for the request lifecycle at `tracing::Level::DEBUG`.
  - Coupled with `SpanContextLayer` (`apiserver/src/utils/logging.rs:46-119`) registered in `main.rs:89`: copies `request_id`, `workspace_id`, and `actor_id` from span attributes into log events.
- **Short-circuit rejection**: None.
- **Ordering constraints**: Must run inside `request_id_middleware` so the span inherits the correlation ID, and outside `CorsLayer` / business routers.

---

### 2.5 CORS (`CorsLayer`)
- **Rust symbol & source**: `tower_http::cors::CorsLayer` (`tower-http 0.6.8`, `apiserver/src/lib.rs:286-305`).
- **Registration**: `apiserver/src/lib.rs:354`.
- **Inputs read**:
  - Request headers: `Origin`, `Access-Control-Request-Method`, `Access-Control-Request-Headers`.
  - Config: `config.allowed_origins` parsed via `parse_allowed_origins` (`apiserver/src/lib.rs:262-275`).
- **Configuration values**:
  - Allowed Origins: parsed list from env config (comma-separated, whitespace trimmed).
  - Credentials: `allow_credentials(true)` (`apiserver/src/lib.rs:288`).
  - Methods: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `OPTIONS` (`apiserver/src/lib.rs:289-296`).
  - Headers: `Authorization`, `Content-Type`, `X-Requested-With`, `X-Workspace-Id`, `X-User-Id`, `Accept`, `Origin` (`apiserver/src/lib.rs:297-305`).
- **Outputs / inserted extensions**:
  - Response headers attached: `Access-Control-Allow-Origin: <matching-origin>`, `Access-Control-Allow-Credentials: true`, `Vary: origin`.
- **Short-circuit rejection**:
  - Preflight `OPTIONS` requests: intercepts and returns immediately with status `200 OK` (empty body) and headers `Access-Control-Allow-Methods`, `Access-Control-Allow-Headers`, `Access-Control-Max-Age` (`tower-http/src/cors/mod.rs:678-689`).
  - Origin mismatch: if `Origin` is present and does not match the allowed origin list, CORS response headers are omitted (browser blocks cross-origin reading).
- **Ordering constraints**: Runs outside the body limit and routing layers. Allows preflight `OPTIONS` requests to return before hitting route handlers or payload buffers.

---

### 2.6 Global & route body limits (`DefaultBodyLimit`)
- **Rust symbol & source**: `axum::extract::DefaultBodyLimit` (`axum-core 0.4.5/src/extract/default_body_limit.rs:77-168`).
- **Registration**:
  - Global default: `apiserver/src/lib.rs:353` — `.layer(axum::extract::DefaultBodyLimit::max(max_body_bytes))`. Sourced from env `MAX_REQUEST_BODY_BYTES` (default 2MB = `2097152` bytes, `env_registry.rs:759-767`).
  - File routes override: `apiserver/src/routes/files.rs:27` — `.route_layer(DefaultBodyLimit::max(max_upload_bytes))`. Sourced from env `MAX_FILE_UPLOAD_BODY_BYTES` (default 20MB = `20971520` bytes, `env_registry.rs:769-777`).
  - Bulk upload routes overrides:
    - Attendance uploads: `apiserver/src/routes/attendance.rs:31` (10MB default).
    - Employee import: `apiserver/src/routes/employees.rs:23` (10MB default).
    - Leave import: `apiserver/src/routes/leaves.rs:23` (10MB default).
    Sourced from env `MAX_BULK_UPLOAD_BODY_BYTES` (default 10MB = `10485760` bytes, `env_registry.rs:779-787`).
- **Inputs read**: Request body byte stream during body buffering / multipart chunking.
- **Outputs / inserted extensions**: Inserts `DefaultBodyLimitKind::Limit(bytes)` into `req.extensions_mut()`. Innermost layer wins because `DefaultBodyLimitService` overwrites previous extension value.
- **Short-circuit rejection**:
  - Transport layer (`LengthLimitError` in `axum-core 0.4.5/src/extract/rejection.rs:40-48`):
    - HTTP Status: `413 Payload Too Large` (`StatusCode::PAYLOAD_TOO_LARGE`).
    - Body: `"Failed to buffer the request body"` (plain text).
  - Business streaming layer (`read_field_limited` in `apiserver/src/utils/upload.rs:31-59`):
    - When multipart field exceeds cap, maps to `AppError::payload_too_large(I18nKey::UploadFileTooLarge)`.
    - HTTP Status: `413 Payload Too Large`.
    - Body (JSON):
      ```json
      {
        "error": {
          "code": "PAYLOAD_TOO_LARGE",
          "message": "File tải lên vượt quá dung lượng cho phép"
        }
      }
      ```
- **Ordering constraints**: Must be applied to router before extractors consume request body.

---

### 2.7 WebSocket upgrade authentication (`ws_auth_middleware`)
- **Rust symbol & source**: `apiserver::handlers::report_websocket::ws_auth_middleware` (`apiserver/src/handlers/report_websocket.rs:33-68`).
- **Registration**: `apiserver/src/routes/reports.rs:38-43` via `.route("/ws/reports/:job_id", get(report_stream_ws).layer(middleware::from_fn_with_state(Arc::clone(state), ws_auth_middleware)))`.
- **Inputs read**:
  1. `?token=` query parameter (`Query<WsQuery>`).
  2. `cookie: access_token=<token>` fallback header.
- **Outputs / inserted extensions**: Inserts verified `user_id` (`Uuid`) into `request.extensions_mut()`. Extracted in handler via `axum::Extension(user_id)`.
- **Short-circuit rejection**:
  - Missing token: returns `(StatusCode::UNAUTHORIZED, "Missing authentication token")`.
  - Invalid/expired token: returns `(StatusCode::UNAUTHORIZED, "Invalid or expired token")`.
- **Ordering constraints**: Must execute *before* Axum's `WebSocketUpgrade` extractor runs; otherwise, unauthenticated requests would upgrade the TCP socket to WebSocket protocol before checking credentials.

---

### 2.8 Identity authentication (`UserId` extractor)
- **Rust symbol & source**: `apiserver::UserId` (`ax_auth::UserId` in `apiserver/src/lib.rs:120,159-257`).
- **Registration**: Implements `FromRequestParts<S>`. Extracted directly by handlers or by downstream context extractors (`MemberContext`, `SystemContext`).
- **Inputs read**:
  1. Debug override (`#[cfg(debug_assertions)]`): `X-Debug-Impersonate-Id` header (`lib.rs:167-178`).
  2. Native header: `Authorization: Bearer <token>` (`lib.rs:180-197`).
  3. Web cookie: `cookie: access_token=<token>` (`lib.rs:199-221`).
  4. Stream query param: `?access_token=<token>` (for SSE/WebSocket fallback) (`lib.rs:223-239`).
  5. Debug header: `#[cfg(debug_assertions)]` `X-User-Id` header (`lib.rs:241-250`).
- **Token validation & revocation check** (`is_token_revoked`, `lib.rs:122-156`):
  - Validates cryptographic signature and claims using `AuthService::verify_token_claims(token, jwt_secret)`.
  - Extracts token signature `sig` (last segment after `.`) and parses `claims.sub` as `Uuid`.
  - Checks Redis signature blocklist: `EXISTS jwt:revoked:{sig}`. If exists, token is revoked.
  - Checks Redis global session revocation: `GET user:revoked_at:{user_id}`. If `claims.iat < revoked_at`, token is revoked.
  - If Redis is unavailable or unconfigured, revocation check fails open (`false`), allowing validly signed tokens.
- **Outputs / inserted extensions**: Yields `UserId(pub Uuid)`.
- **Short-circuit rejection**:
  - Rejection type: `(StatusCode, String)` (`lib.rs:164`).
  - HTTP Status: `401 Unauthorized` (`StatusCode::UNAUTHORIZED`).
  - Response Body: `"Authentication required"` (plain text).
- **Ordering constraints**: Runs after global middleware (Step 1-6) and before domain context extractors (`MemberContext`, `SystemContext`, `EmployeeContext`).

---

### 2.9 Workspace context (`WorkspaceId` extractor)
- **Rust symbol & source**: `apiserver::handlers::common::WorkspaceId` (`apiserver/src/handlers/common.rs:24,78-111`).
- **Registration**: Implements `FromRequestParts<S>`.
- **Inputs read**:
  1. Route path params: inspects Axum path parameters for `'ws_id'` (e.g. `/workspaces/:ws_id/...`) (`common.rs:88-98`).
  2. Fallback header: `X-Workspace-Id` HTTP header (`common.rs:101-110`).
- **Outputs / inserted extensions**: Yields `WorkspaceId(Uuid)`.
- **Short-circuit rejection**:
  - Missing parameter/header: `AppError::validation(I18nKey::MissingWorkspace)`.
    - HTTP Status: `400 Bad Request`.
    - Error code: `"VALIDATION_ERROR"`.
    - Message: `"Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header."` (localized).
  - Malformed UUID: `AppError::validation(I18nKey::InvalidWorkspaceId)`.
    - HTTP Status: `400 Bad Request`.
    - Error code: `"VALIDATION_ERROR"`.
    - Message: `"Invalid Workspace ID format"` (localized).
- **Ordering constraints**: Evaluated before `ApiContext` and `MemberContext`.

---

### 2.10 API routing boundary guard (`ApiContext` extractor)
- **Rust symbol & source**: `apiserver::handlers::common::ApiContext` (`apiserver/src/handlers/common.rs:49-76`).
- **Registration**: Implements `FromRequestParts<S>`.
- **Inputs read**: Evaluates `WorkspaceId::from_request_parts`; then inspects `parts.uri.path()`.
- **Outputs / inserted extensions**:
  - Returns `ApiContext::Tenant(WorkspaceId)` if workspace context is present.
  - Returns `ApiContext::System` if request path starts with `"/system"` or `"/api/v1/system"`.
- **Short-circuit rejection**:
  - If not a system route and workspace context extraction failed: `AppError::validation(I18nKey::MissingWorkspace)`. Status 400.
- **Ordering constraints**: Used internally by `MemberContext` and in system route handlers (`system_admin.rs`, `system_tenants.rs`, `system_db_clusters.rs`).

---

### 2.11 Workspace member & RBAC context (`MemberContext` extractor)
- **Rust symbol & source**: `apiserver::handlers::common::MemberContext` (`apiserver/src/handlers/common.rs:114-195,198-252`).
- **Registration**: Implements `FromRequestParts<S>`.
- **Inputs read**:
  1. Extracts `UserId` via `UserId::from_request_parts(parts, state)` (`common.rs:206`).
  2. Extracts `ApiContext` via `ApiContext::from_request_parts(parts, state)` (`common.rs:207`).
- **Outputs / inserted extensions**: Yields `MemberContext` struct:
  - `user_id: Uuid`
  - `user_name: Option<String>`
  - `user_email: String`
  - `workspace_id: Uuid`
  - `roles: Vec<Uuid>`
  - `permissions: HashSet<Permission>`
  - `managed_department_ids: Option<HashSet<Uuid>>` (`None` = global manager, `Some(...)` = department scoped)
  - `auth_token: TenantAuthToken::new_authorized(workspace_id)`
- **Caching & DB lookup** (`common.rs:218-233,264-367`):
  - Cache key: `cache:auth:permissions:{workspace_id}:{user_id}`.
  - TTL: 3600 seconds (1 hour).
  - Cache miss queries System PostgreSQL DB: `user_repo.get_user_by_id`, `workspace_repo.find_primary_role`, `workspace_repo.find_extra_roles`, `workspace_repo.get_permissions_for_roles`, `workspace_repo.find_extra_permissions`, `workspace_repo.is_any_role_global`, `employees_repo.get_managed_departments`.
- **Short-circuit rejection**:
  - System route misuse: `AppError::forbidden(I18nKey::SystemRouteTenantContext)` if extracted on `/system` route. Status 403.
  - User not found: `AppError::auth(I18nKey::UserNotFound)`. Status 401.
  - User not in workspace: `AppError::forbidden(I18nKey::NotWorkspaceMember)`. Status 403.
  - All errors format as JSON `{"error": {"code": ..., "message": ...}}`.
- **Ordering constraints**: Must run after `UserId` and `WorkspaceId`. Evaluated prior to handler execution.

---

### 2.12 System administration context (`SystemContext` extractor)
- **Rust symbol & source**: `apiserver::handlers::system_auth::SystemContext` (`apiserver/src/handlers/system_auth.rs:44-114,117-161`).
- **Registration**: Implements `FromRequestParts<S>`.
- **Inputs read**: Extracts `UserId` via `UserId::from_request_parts`.
- **Outputs / inserted extensions**: Yields `SystemContext`:
  - `user_id: Uuid`
  - `roles: HashSet<SystemRoleType>` (`PlatformAdmin`, `Support`, `BillingAdmin`).
- **Caching & DB lookup**:
  - Cache key: `cache:system:roles:{user_id}`.
  - TTL: 1800 seconds (30 minutes, `SYSTEM_ROLE_CACHE_TTL`).
  - Cache miss queries `user_repo.get_system_roles_for_user(user_id)`.
- **Short-circuit rejection**:
  - Authentication failure: maps `UserId` error to `AppError::auth(I18nKey::AuthenticationRequired)`. Status 401.
  - Database error: maps to `AppError::Diesel` / `AppError::Internal`. Status 500.
  - Note: Any authenticated user can extract `SystemContext`. Role enforcement occurs inside handler calls via `ctx.require_role(SystemRoleType::PlatformAdmin)` (`system_auth.rs:64-70`), which rejects with 403 Forbidden.
- **Ordering constraints**: Used exclusively on `/system/*` endpoints and `/workspaces` creation.

---

### 2.13 Employee self-service context (`EmployeeContext` extractor)
- **Rust symbol & source**: `apiserver::handlers::common::EmployeeContext` (`apiserver/src/handlers/common.rs:374-415`).
- **Registration**: Implements `FromRequestParts<S>`.
- **Inputs read**:
  1. Runs `MemberContext::from_request_parts(parts, state)` (`common.rs:394`). Verifies workspace membership first before accessing tenant DB.
  2. Queries tenant DB via `employees().repo().get_employee_id_by_user_id(&member_ctx.auth_token, user_id)` (`common.rs:401-406`).
- **Outputs / inserted extensions**: Yields `EmployeeContext`:
  - `user_id: Uuid`
  - `employee_id: Uuid`
  - `workspace_id: Uuid`
  - `auth_token: TenantAuthToken`
- **Short-circuit rejection**:
  - Lacks linked employee profile: `AppError::forbidden(I18nKey::NoEmployeeRecord)`.
    - HTTP Status: `403 Forbidden`.
    - Error code: `"FORBIDDEN"`.
    - Message: `"No employee record linked to your account"` (localized).
- **Ordering constraints**: Must execute after `MemberContext`.

---

### 2.14 Audit logging (`AuditHelper`)
- **Rust symbol & source**: `apiserver::services::audit_service::AuditHelper` (`apiserver/src/services/audit_service.rs:20-69`), `AuditLogRepository` (`apiserver/src/repositories/audit_log_repository.rs:9-22`).
- **Registration**: Direct asynchronous service call within mutating handlers and services (`AuditHelper::log(...)`).
- **Inputs read**: `AuditLogArgs`: repository reference, workspace ID, actor ID, action string, entity name, entity ID, changes diff (JSON), client IP, user agent (`audit_service.rs:6-18`).
- **Outputs / inserted extensions**: Asynchronously inserts record into PostgreSQL table `audit_logs`.
- **Short-circuit rejection**: **Fail-open**. Persistence failures log an error (`"Failed to persist audit log — audit trail may have a gap"`) but do not abort or fail the user request (`audit_service.rs:66-68`).
- **Ordering constraints**: Triggered after successful state mutations or after sensitive data reads (e.g. `handlers/dashboard_metrics.rs:92` logs reads when payroll cost data is surfaced).

---

### 2.15 Timeout, Panic Recovery, and Request Metrics (Absent in Rust apiserver)
- **Audit findings across codebase**:
  - **Timeout layer**: `TimeoutLayer` is **not registered** anywhere in `apiserver/src` (0 occurrences). Timeouts are managed exclusively on specific internal IO operations via `tokio::time::timeout`.
  - **Panic recovery layer**: Neither `tower_http::catch_panic::CatchPanicLayer` nor custom recovery middleware is registered. Unhandled panics in handlers abort the Tokio worker task and cause Hyper/Axum to drop the TCP connection (closing with empty HTTP response or 500 error from Hyper).
  - **HTTP Metrics middleware**: Prometheus / metrics middleware is **not registered** in the HTTP pipeline. Operational metrics are collected via background heartbeat reporting and queue inspect handlers (`handlers/worker_monitor.rs`).
- **Go migration requirement**: Go server must maintain behavioral compatibility; any addition of global timeouts or panic recovery middleware must be documented as an intentional architectural enhancement in `decisions.md`.

---

## 3. Precedence conflicts and interaction evidence

### 3.1 Rate limit before vs. after authentication
- **Registration structure**:
  - Global `RateLimitLayer` is registered at Step 2 (`lib.rs:359`), running *before* the `UserId` extractor runs at Step 13.
  - Dedicated route-level `RateLimitLayer` is registered at Step 7 (`routes/auth.rs:17`) for `/auth/login` and `/auth/register`.
- **Identity resolution precedence** (`apiserver/src/mw/rate_limit.rs:235-274`):
  1. Lightweight JWT check: extracts `Authorization: Bearer <token>`, verifies signature with `AuthService::verify_token(token, jwt_secret)`. If valid, identity is `"u:<user_id>"`.
  2. Debug header check: in debug assertions, checks `x-user-id`. If valid UUID, identity is `"u:<user_id>"`.
  3. Client IP fallback: extracts IP from `ConnectInfo<SocketAddr>` or `x-forwarded-for`. Identity is `"ip:<client_ip>"`.
- **Architectural consequence**:
  - Unauthenticated attackers hammering invalid credentials or probing routes are rate-limited per IP.
  - Authenticated users sharing a corporate NAT/VPN have independent rate limit buckets based on `"u:<user_id>"`.
  - Malformed/expired JWTs fall back to IP rate limiting without rejecting at the rate limiter level; the request proceeds to the `UserId` extractor where it is formally rejected with 401.

### 3.2 Body limit vs. authentication precedence
- **Registration structure**:
  - Global `DefaultBodyLimit` is registered at Step 6 (`lib.rs:353`).
  - Route overrides are registered at Step 8 (`attendance.rs:31`, `files.rs:27`, `employees.rs:23`, `leaves.rs:23`).
  - Extractor ordering: in Axum handler signatures, extractors execute left-to-right (`axum/src/docs/extract.md:130-137`).
- **Handler execution order**:
  - In `create_upload_url_handler` (`apiserver/src/handlers/files.rs:49-53`):
    ```rust
    pub async fn create_upload_url_handler(
        State(state): State<Arc<AppState>>,
        ctx: MemberContext,
        Json(payload): Json<CreateUploadUrlRequest>,
    )
    ```
    `MemberContext` runs before `Json(payload)`.
    - If `Authorization` header is missing: rejected with **401 Unauthorized** before reading body.
    - If user is not in workspace: rejected with **403 Forbidden** before reading body.
    - If user is authorized but body is 5MB (> 2MB limit): rejected with **413 Payload Too Large**.
  - In `upload_attendance_excel` (`apiserver/src/handlers/attendance/import.rs:32-36`):
    ```rust
    pub async fn upload_attendance_excel(
        State(state): State<Arc<AppState>>,
        ctx: MemberContext,
        mut multipart: Multipart,
    )
    ```
    `MemberContext` runs before `Multipart`. If unauthenticated or forbidden, rejects immediately without buffering incoming multipart chunks.
  - In unauthenticated routes (`/auth/login`, `/auth/register` in `apiserver/src/handlers/auth.rs:32-35`):
    ```rust
    pub async fn register(
        State(state): State<Arc<AppState>>,
        Json(req): Json<RegisterRequest>,
    )
    ```
    No auth extractor present. If body exceeds 2MB, `Json` extractor trips `LengthLimitError` and returns **413 Payload Too Large** before any handler code executes.

### 3.3 Nested router MatchedPath preservation
- `RateLimitLayer` reads `axum::extract::MatchedPath` to avoid key fragmentation from dynamic path parameters (`rate:{id}:{tier}:/api/v1/workspaces/:ws_id/employees` instead of a key per UUID).
- Axum nested routers (`Router::new().nest("/api/v1", router)`) append nested prefixes to `MatchedPath` in request extensions (`axum/src/extract/matched_path.rs:113-128`).
- As tested in `apiserver/src/mw/rate_limit.rs:443-505`, route templates retain parameter placeholders (e.g. `:id`), keeping Redis key cardinality bounded.

---

## 4. Extractor rejection behavior & response shapes

Every handler in `apiserver` uses either direct Axum extractors or custom domain extractors. Axum halts handler evaluation at the first extractor that returns an `Err(Rejection)`.

### 4.1 Handler extractors audit summary
- **Result-wrapped extractors**:
  - Zero handlers wrap body extractors in `Result<Json<T>, JsonRejection>` or `Result<Path<T>, PathRejection>`.
  - All handlers rely on standard rejection propagation or return `AppResult<T>`.
  - Optional query parameters use standard `Option<Query<T>>` or default deserialization (`handlers/report_websocket.rs:35`).

### 4.2 Rejection mapping matrix

| Failure Scenario | Failing Extractor | Rejection Type & Source | HTTP Status Code | Response Content-Type | Response Body Format |
|---|---|---|---|---|---|
| **Missing Auth** (no header/cookie/query) | `UserId` | `(StatusCode, String)` (`apiserver/src/lib.rs:164`) | `401 Unauthorized` | `text/plain; charset=utf-8` | `"Authentication required"` |
| **Invalid JWT** (signature/expired/malformed) | `UserId` | `(StatusCode, String)` (`apiserver/src/lib.rs:164`) | `401 Unauthorized` | `text/plain; charset=utf-8` | `"Authentication required"` |
| **Revoked JWT** (signature in Redis blocklist) | `UserId` | `(StatusCode, String)` (`apiserver/src/lib.rs:164`) | `401 Unauthorized` | `text/plain; charset=utf-8` | `"Authentication required"` |
| **Revoked Session** (issued before `user:revoked_at`) | `UserId` | `(StatusCode, String)` (`apiserver/src/lib.rs:164`) | `401 Unauthorized` | `text/plain; charset=utf-8` | `"Authentication required"` |
| **System Context Missing Auth** | `SystemContext` | `AppError::Auth` (`apiserver/src/handlers/system_auth.rs:127`) | `401 Unauthorized` | `application/json` | `{"error": {"code": "UNAUTHORIZED", "message": "Yêu cầu đăng nhập"}}` |
| **Missing Workspace Context** (no path/header) | `WorkspaceId` | `AppError::Validation` (`apiserver/src/handlers/common.rs:105`) | `400 Bad Request` | `application/json` | `{"error": {"code": "VALIDATION_ERROR", "message": "Thiếu thông tin Workspace..."}}` |
| **Invalid Workspace UUID** | `WorkspaceId` | `AppError::Validation` (`apiserver/src/handlers/common.rs:108`) | `400 Bad Request` | `application/json` | `{"error": {"code": "VALIDATION_ERROR", "message": "Định dạng ID Workspace không hợp lệ"}}` |
| **Not a Workspace Member** | `MemberContext` | `AppError::Forbidden` (`apiserver/src/handlers/common.rs:282`) | `403 Forbidden` | `application/json` | `{"error": {"code": "FORBIDDEN", "message": "Bạn không phải là thành viên của workspace này"}}` |
| **MemberContext on System Route** | `MemberContext` | `AppError::Forbidden` (`apiserver/src/handlers/common.rs:212`) | `403 Forbidden` | `application/json` | `{"error": {"code": "FORBIDDEN", "message": "Không thể sử dụng context Tenant... trong route hệ thống"}}` |
| **No Employee Profile Linked** | `EmployeeContext` | `AppError::Forbidden` (`apiserver/src/handlers/common.rs:406`) | `403 Forbidden` | `application/json` | `{"error": {"code": "FORBIDDEN", "message": "Không có thông tin nhân viên nào liên kết với tài khoản..."}}` |
| **Missing System Role** | `SystemContext` | `AppError::Forbidden` (`apiserver/src/handlers/system_auth.rs:68`) | `403 Forbidden` | `application/json` | `{"error": {"code": "FORBIDDEN", "message": "Không có quyền truy cập"}}` |
| **Missing RBAC Permission** | `MemberContext` | `AppError::Forbidden` (`apiserver/src/handlers/common.rs:142`) | `403 Forbidden` | `application/json` | `{"error": {"code": "FORBIDDEN", "message": "Không có quyền truy cập"}}` |
| **Invalid Path UUID** (e.g. `/files/abc`) | `axum::extract::Path<Uuid>` | `FailedToDeserializePathParams` (`axum/src/extract/path/mod.rs:399`) | `400 Bad Request` | `text/plain; charset=utf-8` | `"Invalid URL parameter: ..."` (axum path error message) |
| **Malformed Query String** | `axum::extract::Query<T>` | `QueryRejection` (`axum/src/extract/rejection.rs:93-108`) | `400 Bad Request` | `text/plain; charset=utf-8` | `"Failed to deserialize query string: ..."` |
| **Missing `Content-Type: application/json`** | `axum::Json<T>` | `MissingJsonContentType` (`axum/src/extract/rejection.rs:33-39`) | `415 Unsupported Media Type` | `text/plain; charset=utf-8` | `"Expected request with Content-Type: application/json"` |
| **Invalid JSON Syntax** | `axum::Json<T>` | `JsonSyntaxError` (`axum/src/extract/rejection.rs:21-29`) | `400 Bad Request` | `text/plain; charset=utf-8` | `"Failed to parse the request body as JSON: ..."` |
| **JSON Schema / Deser Error** | `axum::Json<T>` | `JsonDataError` (`axum/src/extract/rejection.rs:9-19`) | `422 Unprocessable Entity` | `text/plain; charset=utf-8` | `"Failed to deserialize the JSON body into the target type: ..."` |
| **Oversized Body (Buffering)** | `axum::Json<T>` | `LengthLimitError` (`axum-core/src/extract/rejection.rs:40-48`) | `413 Payload Too Large` | `text/plain; charset=utf-8` | `"Failed to buffer the request body"` |
| **Invalid Multipart Boundary** | `axum::extract::Multipart` | `InvalidBoundary` (`axum/src/extract/multipart.rs:301-307`) | `400 Bad Request` | `text/plain; charset=utf-8` | `"Invalid boundary for multipart/form-data request"` |
| **Multipart Body Too Large** | `read_field_limited` | `AppError::PayloadTooLarge` (`apiserver/src/utils/upload.rs:24,53`) | `413 Payload Too Large` | `application/json` | `{"error": {"code": "PAYLOAD_TOO_LARGE", "message": "File tải lên vượt quá dung lượng cho phép"}}` |
| **Rate Limit Exceeded** | `RateLimitLayer` | Custom `into_response` (`apiserver/src/mw/rate_limit.rs:212-223`) | `429 Too Many Requests` | `application/json` | `{"error": {"code": "RATE_LIMITED", "message": "Too many requests. Please try again later."}}` |

### 4.3 Rejection implementation code locations
- **`AppError::into_response`**: `apiserver/src/errors.rs:362-405`.
  - Formats JSON body with `"error": {"code": ..., "message": ...}`.
  - If `Conflict(msg, retry)` carries retry payload, appends `"retry": retry` (`errors.rs:388-394`).
  - Calls `crate::utils::i18n::get_locale()` to translate messages into Vietnamese (`"vi"`), English (`"en"`), or Chinese (`"zh"`).
- **`UserId::from_request_parts`**: `apiserver/src/lib.rs:164-257`.
  - Rejection is tuple `(StatusCode::UNAUTHORIZED, "Authentication required".to_string())`.
- **`WorkspaceId::from_request_parts`**: `apiserver/src/handlers/common.rs:79-111`.
  - Rejection is `AppError`.
- **`MemberContext::from_request_parts`**: `apiserver/src/handlers/common.rs:198-252`.
  - Rejection is `AppError`.
- **`SystemContext::from_request_parts`**: `apiserver/src/handlers/system_auth.rs:117-161`.
  - Rejection is `AppError`.
- **`EmployeeContext::from_request_parts`**: `apiserver/src/handlers/common.rs:382-415`.
  - Rejection is `AppError`.
- **`DebugContext::from_request_parts`**: `apiserver/src/utils/debug.rs:13-40`.
  - Rejection is tuple `(StatusCode, String)`.

---

## 5. Go migration architecture recommendations

To maintain exact behavioral parity with the Rust implementation without adding unnecessary frameworks:

1. **Standard `net/http` Middleware Chain**:
   - Chain order in Go must match the table in Section 1:
     `RequestID -> RateLimiter -> Locale -> Tracing -> CORS -> BodyLimit -> RouteHandler`.
2. **Context Propagation (`context.Context`)**:
   - `RequestID`: attach to `r.Context()` and set header `x-request-id` in response writer.
   - `CURRENT_LOCALE`: attach resolved string to `r.Context()` so error formatters can translate keys.
   - `CURRENT_TRACE_CONTEXT`: attach correlation ID to `r.Context()`.
3. **Identity & Context Extractors**:
   - In Go handlers, extract `user_id`, `workspace_id`, and `MemberContext` via helper functions reading `r.Context()`, `r.Header`, and `r.PathValue(...)`.
   - Maintain the dual-rejection contract:
     - Plain-text 401 (`"Authentication required"`) for primitive auth extraction failures.
     - Structured JSON `{"error": {"code": "VALIDATION_ERROR", "message": ...}}` for domain context failures.
4. **Rate Limiting with Redis & In-Memory Fallback**:
   - Port the Redis Lua script (`mw/rate_limit.rs:288-305`) verbatim to execute atomic `INCR` + conditional `EXPIRE`.
   - Implement an in-memory LRU cache fallback when Redis ping fails.
   - Maintain the `/health` and `/health/ready` bypass.

---

## 6. Audit conclusion & unknown items

- **Layer count**: 6 global middleware layers, 2 route-layer overrides, 1 method-layer WebSocket auth, 9 extractor/guard stages. Total **18 discrete pipeline stages**.
- **Provability**: Every layer, registration line, rejection status, and header has been cited directly from `apiserver/src` and checked against vendored crate sources (`axum 0.7.9`, `axum-core 0.4.5`, `tower-http 0.6.8`).
- **Unknown items**: **NONE**.
  - Confirmed: Global timeout layers, panic recovery layers, and HTTP metrics middleware are absent in Rust `apiserver`.
