diesel::table! {
    approval_action_log (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 50]
        entity_type -> Varchar,
        entity_id -> Uuid,
        step_order -> Int4,
        #[max_length = 100]
        step_name -> Nullable<Varchar>,
        acted_by_user_id -> Uuid,
        #[max_length = 20]
        action -> Varchar,
        note -> Nullable<Text>,
        acted_at -> Timestamptz,
    }
}

diesel::table! {
    approval_workflow_steps (id) {
        id -> Uuid,
        workflow_id -> Uuid,
        step_order -> Int4,
        #[max_length = 100]
        step_name -> Varchar,
        #[max_length = 30]
        approver_type -> Varchar,
        approver_config -> Jsonb,
        #[max_length = 20]
        action_required -> Varchar,
        can_skip -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    approval_workflows (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 50]
        entity_type -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        total_steps -> Int4,
        is_active -> Bool,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    workspace_config_entries (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 20]
        scope_type -> Varchar,
        scope_id -> Nullable<Uuid>,
        #[max_length = 100]
        config_key -> Varchar,
        value_int -> Nullable<Int4>,
        value_decimal -> Nullable<Numeric>,
        value_text -> Nullable<Text>,
        value_bool -> Nullable<Bool>,
        value_json -> Nullable<Jsonb>,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OtRequestStatusType;

    overtime_requests (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        requested_by_user_id -> Uuid,
        department_id -> Nullable<Uuid>,
        planned_date -> Date,
        planned_start_time -> Nullable<Time>,
        planned_end_time -> Nullable<Time>,
        planned_minutes -> Int4,
        reason -> Text,
        status -> OtRequestStatusType,
        workflow_id -> Nullable<Uuid>,
        current_step -> Int4,
        quota_warning -> Nullable<Jsonb>,
        reconciled_at -> Nullable<Timestamptz>,
        reconciled_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    overtime_request_employees (id) {
        id -> Uuid,
        overtime_request_id -> Uuid,
        employee_id -> Uuid,
        #[max_length = 20]
        consent_status -> Varchar,
        consent_at -> Nullable<Timestamptz>,
        decline_reason -> Nullable<Text>,
        actual_ot_minutes -> Nullable<Int4>,
        matched_summary_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}
