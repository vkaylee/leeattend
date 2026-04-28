use axum::{
    extract::{Path, State},
    Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    handlers::common::MemberContext,
    models::{EmployeePhase, EmploymentPhaseDefinition},
    repositories::EmploymentPhaseRepository,
    AppState,
};

#[derive(Deserialize)]
pub struct CreateEmploymentPhaseDefinitionRequest {
    pub name: String,
    pub description: Option<String>,
    pub phase_type: String,
    pub rule_config: serde_json::Value,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct AssignEmployeePhaseRequest {
    pub phase_definition_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub override_config: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateEmployeePhaseStatusRequest {
    pub status: String,
    pub evaluation_notes: Option<String>,
}

pub async fn create_employment_phase_definition(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<CreateEmploymentPhaseDefinitionRequest>,
) -> Result<Json<EmploymentPhaseDefinition>, AppError> {
    let def = EmploymentPhaseRepository::create_definition(
        state.infra.db.api(),
        ctx.workspace_id,
        &payload.name,
        payload.description.as_deref(),
        &payload.phase_type,
        &payload.rule_config,
        payload.display_order.unwrap_or(0),
    )
    .await?;

    Ok(Json(def))
}

pub async fn get_employment_phase_definitions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<Json<Vec<EmploymentPhaseDefinition>>, AppError> {
    let defs = EmploymentPhaseRepository::get_definitions(state.infra.db.api(), ctx.workspace_id).await?;
    Ok(Json(defs))
}

pub async fn assign_employee_phase(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, emp_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignEmployeePhaseRequest>,
) -> Result<Json<EmployeePhase>, AppError> {
    let phase = EmploymentPhaseRepository::assign_phase(
        state.infra.db.api(),
        ctx.workspace_id,
        emp_id,
        payload.phase_definition_id,
        payload.start_date,
        payload.end_date,
        payload.override_config.as_ref(),
    )
    .await?;

    Ok(Json(phase))
}

pub async fn update_employee_phase_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, emp_id, phase_id)): Path<(Uuid, Uuid, Uuid)>,
    // TODO: Extract user ID from auth context for `evaluated_by`
    Json(payload): Json<UpdateEmployeePhaseStatusRequest>,
) -> Result<Json<EmployeePhase>, AppError> {
    // Dummy evaluator UUID for now, until Auth layer is injected correctly in route definition
    let dummy_evaluator = Uuid::new_v4();

    let phase = EmploymentPhaseRepository::update_phase_status(
        state.infra.db.api(),
        ctx.workspace_id,
        emp_id,
        phase_id,
        &payload.status,
        payload.evaluation_notes.as_deref(),
        dummy_evaluator,
    )
    .await?;

    Ok(Json(phase))
}
