/// Handler files that legitimately use `Path(ws_id)` instead of MemberContext.
///
/// Keep this list MINIMAL. Each entry must have a documented reason.
pub const CONTEXT_ALLOWLIST: &[(&str, &str)] = &[
    // Admin-level workspace CRUD: user is authenticated, ws_id from URL
    // is validated against user's workspace list in the handler
    (
        "workspaces/crud.rs",
        "workspace CRUD by authenticated admin",
    ),
    // Debug/dev endpoints: only available in debug builds
    ("debug.rs", "dev-only endpoints, compiled out in release"),
];
