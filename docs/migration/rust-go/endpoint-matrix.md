# Endpoint matrix summary

- Row count: **403** data rows (header excluded). Three rows cover OpenAPI operations declared in `apiserver/src/openapi.rs` with no merged route (`create_signature_request`, `get_pending_signatures`, `sign_document`); these rows are explicitly marked `UNKNOWN — needs decision`.
- Enumeration: parsed every `.route(...)` registration and each Axum HTTP operation in all 19 route modules under `apiserver/src/routes/`; canonical served path prepends `/api/v1` from `apiserver/src/lib.rs:351-353`.
- Router registration: module merges at `apiserver/src/lib.rs:308-328`; global serving prefix/layers at `apiserver/src/lib.rs:351-365`.
- Route operations by module: attendance=34, auth=9, condition_types=5, custom_fields=7, dashboard=15, documents=10, employees=42, ess=16, feature_flags=7, files=9, leaves=37, lifecycle=15, notifications=4, payroll=31, reports=15, shifts=33, system=17, time_tracking=5, workspaces=89.
- OpenAPI cross-reference: metadata read from `apiserver/src/openapi.rs:7-481` and handler `#[utoipa::path]` attributes. OpenAPI-only unmounted handlers: create_signature_request, get_pending_signatures, sign_document; registered handlers absent from OpenAPI list: health_check, readiness_check, report_stream_ws, ws_handler.
- Unresolved handler sources: 0 for registered routes; unmounted OpenAPI rows are unresolved serving contracts and marked `UNKNOWN — needs decision`.
