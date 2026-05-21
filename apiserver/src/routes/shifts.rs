use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/workspaces/:ws_id/shifts",
            get(handlers::get_shifts).post(handlers::create_shift),
        )
        .route(
            "/workspaces/:ws_id/shifts/:shift_id",
            put(handlers::update_shift),
        )
        .route(
            "/workspaces/:ws_id/shifts/assign",
            post(handlers::assign_shift),
        )
        // Working patterns
        .route(
            "/workspaces/:ws_id/working-patterns",
            get(handlers::working_patterns::list_patterns)
                .post(handlers::working_patterns::create_pattern),
        )
        .route(
            "/workspaces/:ws_id/working-patterns/bulk-assign",
            post(handlers::working_patterns::bulk_assign_pattern),
        )
        .route(
            "/workspaces/:ws_id/working-patterns/:pattern_id",
            get(handlers::working_patterns::get_pattern_details)
                .put(handlers::working_patterns::update_pattern)
                .delete(handlers::working_patterns::delete_pattern),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/working-patterns",
            get(handlers::working_patterns::get_employee_pattern_history)
                .post(handlers::working_patterns::assign_pattern),
        )
        // Assignment rules
        .route(
            "/workspaces/:ws_id/assignment-rules",
            get(handlers::assignment_rules::get_assignment_rules)
                .post(handlers::assignment_rules::create_assignment_rule),
        )
        .route(
            "/workspaces/:ws_id/assignment-rules/:rule_id",
            patch(handlers::assignment_rules::update_assignment_rule)
                .delete(handlers::assignment_rules::delete_assignment_rule),
        )
        // Shift swaps
        .route(
            "/workspaces/:ws_id/shift-swaps",
            get(handlers::shift_swaps::list_shift_swaps)
                .post(handlers::shift_swaps::create_shift_swap),
        )
        .route(
            "/workspaces/:ws_id/shift-swaps/stats",
            get(handlers::shift_swaps::get_shift_swap_stats),
        )
        .route(
            "/workspaces/:ws_id/shift-swaps/:id/approve",
            put(handlers::shift_swaps::approve_shift_swap),
        )
        // Rosters (schedules)
        .route(
            "/workspaces/:ws_id/schedules",
            get(handlers::get_workspace_schedules),
        )
        .route(
            "/workspaces/:ws_id/assignment-history",
            get(handlers::get_assignment_history),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rosters",
            get(handlers::get_employee_rosters).post(handlers::upsert_roster),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rosters/bulk",
            post(handlers::bulk_upsert_roster),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rosters/:date",
            delete(handlers::delete_roster),
        )
}
