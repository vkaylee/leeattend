use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/ess/profile",
            get(handlers::ess::get_my_profile).put(handlers::ess::update_my_profile),
        )
        .route(
            "/ess/emergency-contacts",
            get(handlers::ess::get_my_emergency_contacts)
                .post(handlers::ess::add_emergency_contact),
        )
        .route(
            "/ess/emergency-contacts/:contact_id",
            delete(handlers::ess::delete_emergency_contact),
        )
        .route("/ess/documents", get(handlers::ess::get_my_documents))
        .route("/ess/payslips", get(handlers::ess::get_my_payslips))
        .route("/ess/attendance", get(handlers::ess::get_my_attendance))
        .route("/ess/punch", post(handlers::ess::ess_punch))
        .route("/ess/time-entries", get(handlers::ess::get_my_time_entries))
        .route(
            "/ess/leave/balances",
            get(handlers::ess::get_my_leave_balances),
        )
        .route(
            "/ess/leave/requests",
            get(handlers::ess::ess_get_my_leave_requests),
        )
        // ESS Shift Swap routes
        .route(
            "/ess/shift-swaps",
            get(handlers::ess::ess_get_my_shift_swaps).post(handlers::ess::ess_create_shift_swap),
        )
        .route(
            "/ess/shift-swaps/:id/respond",
            put(handlers::ess::ess_respond_to_swap),
        )
        .route(
            "/ess/shift-swaps/:id",
            delete(handlers::ess::ess_cancel_swap),
        )
}
