// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "adjustment_status_type"))]
    pub struct AdjustmentStatusType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "assignment_status"))]
    pub struct AssignmentStatus;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "contract_status_enum"))]
    pub struct ContractStatusEnum;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "contract_type_enum"))]
    pub struct ContractTypeEnum;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "import_job_status_type"))]
    pub struct ImportJobStatusType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "leave_status_type"))]
    pub struct LeaveStatusType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "log_source_type"))]
    pub struct LogSourceType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ot_request_status_type"))]
    pub struct OtRequestStatusType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ot_status_type"))]
    pub struct OtStatusType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "recruitment_source_type"))]
    pub struct RecruitmentSourceType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "segment_type"))]
    pub struct SegmentType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "gap_compensation_type"))]
    pub struct GapCompensationType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "segment_status_type"))]
    pub struct SegmentStatusType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ot_aggregation_mode_type"))]
    pub struct OtAggregationModeType;
}

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
    approval_workflow_instances (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        workflow_id -> Nullable<Uuid>,
        #[max_length = 50]
        entity_type -> Varchar,
        entity_id -> Uuid,
        #[max_length = 20]
        status -> Varchar,
        current_step_order -> Int4,
        steps_snapshot -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
    approval_workflow_tasks (id) {
        id -> Uuid,
        instance_id -> Uuid,
        step_order -> Int4,
        #[max_length = 20]
        status -> Varchar,
        #[max_length = 30]
        assignee_type -> Varchar,
        assignee_target_id -> Nullable<Uuid>,
        #[max_length = 100]
        assignee_target_name -> Nullable<Varchar>,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        completed_by -> Nullable<Uuid>,
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
    assignment_rules (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        pattern_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        priority -> Int4,
        conditions -> Jsonb,
        is_active -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        start_date -> Nullable<Date>,
        end_date -> Nullable<Date>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AdjustmentStatusType;

    attendance_adjustment_requests (id) {
        id -> Uuid,
        time_entry_id -> Nullable<Uuid>,
        requested_punch_time -> Timestamptz,
        reason -> Nullable<Text>,
        status -> AdjustmentStatusType,
        requested_by_user_id -> Nullable<Uuid>,
        approved_by_user_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        employment_id -> Uuid,
        attachment_file_ids -> Nullable<Array<Nullable<Uuid>>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OtStatusType;

    attendance_daily_summaries (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        date -> Date,
        actual_work_minutes -> Int4,
        system_detected_ot_minutes -> Int4,
        claimed_ot_minutes -> Int4,
        approved_ot_minutes -> Int4,
        ot_status -> OtStatusType,
        ot_reason -> Nullable<Text>,
        manager_note -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        claim_entered_by_user_id -> Nullable<Uuid>,
        claim_entered_at -> Nullable<Timestamptz>,
        paper_form_photo_url -> Nullable<Text>,
        entered_via -> Nullable<Text>,
        is_holiday -> Bool,
        #[max_length = 255]
        holiday_name -> Nullable<Varchar>,
        employment_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ImportJobStatusType;

    attendance_import_jobs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        status -> ImportJobStatusType,
        total_rows -> Int4,
        processed_rows -> Int4,
        error_log -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        file_path -> Nullable<Text>,
        #[max_length = 50]
        target_type -> Varchar,
    }
}

diesel::table! {
    attendance_log_errors (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        agent_id -> Nullable<Text>,
        raw_content -> Nullable<Jsonb>,
        error_message -> Text,
        source -> Text,
        created_at -> Nullable<Timestamptz>,
        fingerprint -> Nullable<Text>,
    }
}

diesel::table! {
    attendance_policies (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        is_default -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_policy_assignments (id) {
        id -> Uuid,
        policy_id -> Uuid,
        #[max_length = 50]
        target_type -> Varchar,
        target_id -> Uuid,
        valid_from -> Timestamptz,
        valid_to -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_policy_versions (id) {
        id -> Uuid,
        policy_id -> Uuid,
        rules -> Jsonb,
        valid_from -> Timestamptz,
        valid_to -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_default (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m01 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m02 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m03 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m04 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m05 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m06 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m07 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m08 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m09 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m10 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m11 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    attendance_staging_logs_y2026m12 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        device_id -> Nullable<Text>,
        raw_user_id -> Text,
        punch_time -> Timestamptz,
        raw_data -> Nullable<Jsonb>,
        fingerprint -> Text,
        status -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_default (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m01 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m02 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m03 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m04 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m05 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m06 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m07 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m08 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m09 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m10 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m11 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    audit_logs_y2026m12 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        #[max_length = 50]
        actor_type -> Nullable<Varchar>,
        #[max_length = 255]
        actor_name -> Nullable<Varchar>,
        #[max_length = 255]
        actor_email -> Nullable<Varchar>,
    }
}

diesel::table! {
    authorized_agents (id) {
        #[max_length = 50]
        id -> Varchar,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 255]
        token -> Varchar,
        is_trusted -> Nullable<Bool>,
        #[max_length = 45]
        last_ip -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        is_active -> Bool,
        workspace_id -> Uuid,
        last_seen_at -> Nullable<Timestamptz>,
        max_concurrent_workers -> Int4,
        success_cooldown_secs -> Int4,
        retention_days -> Int4,
    }
}

diesel::table! {
    behalf_assignments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        target_employee_id -> Nullable<Uuid>,
        target_department_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 50]
        assignment_type -> Varchar,
        assignee_user_id -> Uuid,
        employer_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    custom_field_definitions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 50]
        entity_type -> Varchar,
        #[max_length = 100]
        field_key -> Varchar,
        field_label -> Text,
        #[max_length = 20]
        field_type -> Varchar,
        options -> Nullable<Jsonb>,
        default_value -> Nullable<Text>,
        is_required -> Bool,
        is_visible_to_employee -> Bool,
        is_filterable -> Bool,
        is_exportable -> Bool,
        display_order -> Int4,
        #[max_length = 100]
        section_name -> Nullable<Varchar>,
        validation_rules -> Nullable<Jsonb>,
        deleted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    custom_field_values (id) {
        id -> Uuid,
        definition_id -> Uuid,
        entity_id -> Uuid,
        value_text -> Nullable<Text>,
        value_number -> Nullable<Numeric>,
        value_date -> Nullable<Date>,
        value_bool -> Nullable<Bool>,
        value_json -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    condition_type_definitions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 50]
        code -> Varchar,
        name_i18n -> Jsonb,
        #[max_length = 50]
        category -> Varchar,
        default_minute_allowance -> Int4,
        default_is_paid -> Bool,
        affects_rules -> Jsonb,
        icon -> Nullable<Varchar>,
        #[max_length = 20]
        color -> Nullable<Varchar>,
        is_system -> Bool,
        sort_order -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    dashboard_preferences (id) {
        id -> Uuid,
        user_id -> Uuid,
        workspace_id -> Uuid,
        active_view -> Text,
        visible_widgets -> Jsonb,
        widget_order -> Jsonb,
        collapsed_sections -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    departments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        description -> Nullable<Text>,
        manager_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        std_working_days -> Nullable<Numeric>,
        deleted_at -> Nullable<Timestamptz>,
        default_working_pattern_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    devices (id) {
        id -> Uuid,
        #[max_length = 50]
        agent_id -> Varchar,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 45]
        ip -> Varchar,
        port -> Nullable<Int4>,
        is_enabled -> Nullable<Bool>,
        #[max_length = 20]
        last_status -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        workspace_id -> Uuid,
        brand -> Text,
        #[max_length = 50]
        password -> Nullable<Varchar>,
        #[max_length = 50]
        timezone -> Nullable<Varchar>,
        location_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    device_enrollments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
        device_id -> Uuid,
        device_uid -> Nullable<Int4>,
        #[max_length = 30]
        sync_status -> Varchar,
        last_synced_at -> Nullable<Timestamptz>,
        last_error -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    employee_biometric_templates (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
        finger_id -> Int2,
        template_encrypted -> Text,
        #[max_length = 50]
        encryption_key_version -> Varchar,
        source_device_id -> Nullable<Uuid>,
        quality -> Nullable<Int2>,
        valid -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AssignmentStatus;

    employee_assignments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        home_location_id -> Uuid,
        host_location_id -> Uuid,
        start_date -> Date,
        end_date -> Nullable<Date>,
        reason -> Nullable<Text>,
        status -> AssignmentStatus,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        employment_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ContractTypeEnum;
    use super::sql_types::ContractStatusEnum;

    employee_contracts (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 100]
        contract_number -> Nullable<Varchar>,
        contract_type -> ContractTypeEnum,
        status -> ContractStatusEnum,
        start_date -> Date,
        end_date -> Nullable<Date>,
        document_id -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        employment_id -> Uuid,
    }
}

diesel::table! {
    employee_documents (id) {
        id -> Uuid,
        employee_id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 50]
        document_type -> Varchar,
        #[max_length = 255]
        file_name -> Varchar,
        #[max_length = 500]
        file_path -> Varchar,
        file_size -> Int4,
        #[max_length = 100]
        mime_type -> Varchar,
        expires_at -> Nullable<Date>,
        is_active -> Bool,
        uploaded_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    employee_emergency_contacts (id) {
        id -> Uuid,
        employee_id -> Uuid,
        #[max_length = 100]
        contact_name -> Varchar,
        #[max_length = 50]
        relationship -> Varchar,
        #[max_length = 20]
        phone -> Varchar,
        is_primary -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    employee_phases (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        phase_definition_id -> Uuid,
        start_date -> Date,
        end_date -> Date,
        status -> Text,
        override_config -> Nullable<Jsonb>,
        evaluation_notes -> Nullable<Text>,
        evaluated_by -> Nullable<Uuid>,
        completed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        employment_id -> Uuid,
    }
}

diesel::table! {
    employee_profiles (id) {
        id -> Uuid,
        employee_id -> Uuid,
        #[max_length = 20]
        phone -> Nullable<Varchar>,
        address -> Nullable<Text>,
        date_of_birth -> Nullable<Date>,
        #[max_length = 10]
        gender -> Nullable<Varchar>,
        national_id -> Nullable<Text>,
        #[max_length = 100]
        bank_name -> Nullable<Varchar>,
        bank_account_number -> Nullable<Text>,
        #[max_length = 100]
        bank_branch -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        avatar_file_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    employee_salary_components (id) {
        id -> Uuid,
        component_definition_id -> Uuid,
        amount -> Numeric,
        effective_date -> Date,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        employment_id -> Uuid,
    }
}

diesel::table! {
    employee_shifts (id) {
        id -> Uuid,
        template_id -> Uuid,
        start_date -> Date,
        end_date -> Nullable<Date>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        employment_id -> Uuid,
    }
}

diesel::table! {
    employee_working_patterns (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        pattern_id -> Uuid,
        start_date -> Date,
        end_date -> Nullable<Date>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        anchor_date -> Nullable<Date>,
        employment_id -> Uuid,
    }
}

diesel::table! {
    employees (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 50]
        employee_id_number -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        tags -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 255]
        full_name -> Varchar,
        recruitment_source_id -> Nullable<Uuid>,
        deleted_at -> Nullable<Timestamptz>,
        hire_date -> Date,
        original_hire_date -> Date,
        termination_date -> Nullable<Date>,
        is_active -> Bool,
    }
}

diesel::table! {
    employers (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 50]
        tax_code -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        legal_address -> Nullable<Varchar>,
        deleted_at -> Nullable<Timestamptz>,
        parent_id -> Nullable<Uuid>,
        #[max_length = 100]
        relationship_type -> Nullable<Varchar>,
        #[max_length = 50]
        business_type -> Nullable<Varchar>,
        #[max_length = 100]
        registration_number -> Nullable<Varchar>,
        #[max_length = 20]
        phone -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        #[max_length = 255]
        website -> Nullable<Varchar>,
        incorporation_date -> Nullable<Date>,
        #[max_length = 3]
        country_code -> Nullable<Varchar>,
        #[max_length = 3]
        currency_code -> Nullable<Varchar>,
    }
}

diesel::table! {
    employment_phase_definitions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        phase_type -> Text,
        rule_config -> Jsonb,
        is_active -> Bool,
        display_order -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    employments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
        employer_id -> Uuid,
        primary_location_id -> Nullable<Uuid>,
        department_id -> Nullable<Uuid>,
        #[max_length = 50]
        status -> Varchar,
        start_date -> Date,
        end_date -> Nullable<Date>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 100]
        job_title -> Nullable<Varchar>,
        #[max_length = 50]
        employment_type -> Varchar,
        manager_employment_id -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        #[max_length = 50]
        termination_reason -> Nullable<Varchar>,
    }
}

diesel::table! {
    feature_flags (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 128]
        flag_key -> Varchar,
        enabled -> Bool,
        description -> Nullable<Text>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    holidays (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        date -> Date,
        #[max_length = 50]
        holiday_type -> Varchar,
        is_paid -> Bool,
        ot_multiplier -> Nullable<Numeric>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        leave_type_id -> Nullable<Uuid>,
        work_threshold_percent -> Nullable<Int4>,
        is_credit_on_work -> Bool,
        deleted_at -> Nullable<Timestamptz>,
        location_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    leave_accrual_logs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        policy_assignment_id -> Nullable<Uuid>,
        accrual_date -> Date,
        amount -> Numeric,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        leave_type_id -> Uuid,
        employment_id -> Uuid,
    }
}

diesel::table! {
    leave_balances (id) {
        id -> Uuid,
        year -> Int4,
        entitlement -> Int4,
        carried_over -> Int4,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        accrued_amount -> Numeric,
        leave_type_id -> Uuid,
        employment_id -> Uuid,
    }
}

diesel::table! {
    leave_policy_assignments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        assignee_type -> Text,
        assignee_id -> Nullable<Uuid>,
        policy_id -> Uuid,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        created_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    leave_policy_definitions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        rule_config -> Jsonb,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        leave_type_id -> Uuid,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LeaveStatusType;

    leave_requests (id) {
        id -> Uuid,
        start_date -> Date,
        end_date -> Date,
        reason -> Nullable<Text>,
        status -> LeaveStatusType,
        requested_by_user_id -> Nullable<Uuid>,
        approved_by_user_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        leave_type_id -> Uuid,
        #[max_length = 20]
        duration_type -> Varchar,
        start_time -> Nullable<Time>,
        end_time -> Nullable<Time>,
        employment_id -> Uuid,
        attachment_file_ids -> Nullable<Array<Nullable<Uuid>>>,
    }
}

diesel::table! {
    leave_types (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 50]
        code -> Varchar,
        name -> Text,
        is_paid -> Bool,
        created_at -> Timestamptz,
        policy_config -> Jsonb,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    lifecycle_checklist_items (id) {
        id -> Uuid,
        checklist_id -> Uuid,
        template_item_id -> Nullable<Uuid>,
        title -> Text,
        description -> Nullable<Text>,
        #[max_length = 50]
        category -> Varchar,
        assignee_user_id -> Nullable<Uuid>,
        is_required -> Bool,
        due_date -> Nullable<Date>,
        #[max_length = 20]
        status -> Varchar,
        completed_at -> Nullable<Timestamptz>,
        completed_by -> Nullable<Uuid>,
        notes -> Nullable<Text>,
        display_order -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    lifecycle_checklist_template_items (id) {
        id -> Uuid,
        template_id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        #[max_length = 50]
        category -> Varchar,
        #[max_length = 20]
        assignee_type -> Varchar,
        assignee_user_id -> Nullable<Uuid>,
        is_required -> Bool,
        due_days_offset -> Int4,
        display_order -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    lifecycle_checklist_templates (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        #[max_length = 20]
        checklist_type -> Varchar,
        is_default -> Bool,
        is_active -> Bool,
        deleted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    lifecycle_checklists (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employment_id -> Uuid,
        template_id -> Nullable<Uuid>,
        #[max_length = 20]
        checklist_type -> Varchar,
        #[max_length = 20]
        status -> Varchar,
        progress_percent -> Int4,
        started_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    member_extra_roles (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        assigned_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        role_id -> Uuid,
    }
}

diesel::table! {
    notifications (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[sql_name = "type"]
        #[max_length = 50]
        type_ -> Varchar,
        #[max_length = 255]
        title -> Varchar,
        content -> Text,
        is_read -> Bool,
        #[max_length = 255]
        link -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    outbox_jobs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 50]
        job_type -> Varchar,
        payload -> Jsonb,
        #[max_length = 20]
        status -> Varchar,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    overtime_policy_assignments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        assignee_type -> Text,
        assignee_id -> Nullable<Uuid>,
        policy_id -> Uuid,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    overtime_policy_definitions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    overtime_request_employees (id) {
        id -> Uuid,
        overtime_request_id -> Uuid,
        #[max_length = 20]
        consent_status -> Varchar,
        consent_at -> Nullable<Timestamptz>,
        decline_reason -> Nullable<Text>,
        actual_ot_minutes -> Nullable<Int4>,
        matched_summary_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        employment_id -> Uuid,
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
        attachment_file_ids -> Nullable<Array<Nullable<Uuid>>>,
    }
}

diesel::table! {
    overtime_rules (id) {
        id -> Uuid,
        policy_id -> Uuid,
        priority -> Int4,
        multiplier -> Numeric,
        time_start -> Nullable<Time>,
        time_end -> Nullable<Time>,
        days_of_week -> Int4,
        is_holiday -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    payroll_incentive_assignments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        policy_id -> Uuid,
        assignee_type -> Text,
        assignee_id -> Nullable<Uuid>,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    payroll_incentive_policies (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        base_amount -> Float8,
        payout_type -> Text,
        is_active -> Bool,
        created_at -> Timestamptz,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    payroll_incentive_tiers (id) {
        id -> Uuid,
        policy_id -> Uuid,
        priority -> Int4,
        payout_amount -> Float8,
        rule_config -> Jsonb,
    }
}

diesel::table! {
    payroll_rule_assignments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        rule_definition_id -> Uuid,
        assignee_type -> Text,
        assignee_id -> Nullable<Uuid>,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        override_config -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    payroll_rule_definitions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        code -> Text,
        category -> Text,
        applies_to -> Text,
        calculation_method -> Text,
        rule_config -> Jsonb,
        is_mandatory -> Bool,
        is_active -> Bool,
        display_order -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    payroll_rule_tiers (id) {
        id -> Uuid,
        rule_definition_id -> Uuid,
        tier_order -> Int4,
        min_amount -> Numeric,
        max_amount -> Nullable<Numeric>,
        rate -> Numeric,
        fixed_amount -> Numeric,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    payslip_incentives (id) {
        payslip_id -> Uuid,
        policy_name -> Text,
        amount -> Float8,
        tier_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        standard_amount -> Float8,
        id -> Uuid,
    }
}

diesel::table! {
    payslip_rule_evaluations (id) {
        id -> Uuid,
        payslip_id -> Uuid,
        rule_code -> Text,
        amount -> Numeric,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    payslip_segments (id) {
        id -> Uuid,
        payslip_id -> Uuid,
        start_date -> Date,
        end_date -> Date,
        department_id -> Nullable<Uuid>,
        applied_base_salary -> Numeric,
        applied_std_days -> Numeric,
        actual_work_days -> Numeric,
        paid_work_days -> Numeric,
        amount -> Numeric,
        created_at -> Nullable<Timestamptz>,
        applied_hours_per_day -> Numeric,
        #[max_length = 50]
        std_days_source -> Nullable<Varchar>,
    }
}

diesel::table! {
    payslips (id) {
        id -> Uuid,
        period_start -> Date,
        period_end -> Date,
        base_pay_amount -> Numeric,
        overtime_pay_amount -> Numeric,
        deductions_amount -> Numeric,
        net_pay_amount -> Numeric,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        standard_units -> Int4,
        overtime_units -> Int4,
        holiday_work_premium_amount -> Numeric,
        workspace_id -> Uuid,
        employment_id -> Uuid,
    }
}

diesel::table! {
    policy_templates (id) {
        id -> Uuid,
        workspace_id -> Nullable<Uuid>,
        name -> Text,
        description -> Nullable<Text>,
        #[max_length = 3]
        country_code -> Nullable<Varchar>,
        category -> Text,
        template_data -> Jsonb,
        version -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LogSourceType;

    raw_attendance_logs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        punch_time -> Timestamptz,
        source -> LogSourceType,
        #[max_length = 50]
        device_id -> Nullable<Varchar>,
        raw_data -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        punch -> Int4,
        status -> Int4,
        employment_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RecruitmentSourceType;

    recruitment_sources (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        source_type -> RecruitmentSourceType,
        #[max_length = 255]
        name -> Varchar,
        referrer_employee_id -> Nullable<Uuid>,
        referral_bonus_amount -> Nullable<Numeric>,
        referral_bonus_paid -> Bool,
        #[max_length = 255]
        agency_contact_name -> Nullable<Varchar>,
        #[max_length = 255]
        agency_contact_email -> Nullable<Varchar>,
        agency_fee_amount -> Nullable<Numeric>,
        agency_fee_paid -> Bool,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    salary_base_groups (id) {
        id -> Uuid,
        workspace_id -> Nullable<Uuid>,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 100]
        slug -> Varchar,
        description -> Nullable<Text>,
        is_system -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    salary_component_base_members (component_definition_id, base_group_id) {
        component_definition_id -> Uuid,
        base_group_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    salary_component_definitions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        code -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        component_type -> Text,
        linked_rule_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        token -> Text,
        expires_at -> Timestamptz,
        created_at -> Nullable<Timestamptz>,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    shift_rosters (id) {
        id -> Uuid,
        template_id -> Nullable<Uuid>,
        date -> Date,
        #[max_length = 20]
        status -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        employment_id -> Uuid,
    }
}

diesel::table! {
    shift_swap_requests (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        requester_id -> Uuid,
        target_id -> Uuid,
        swap_date -> Date,
        requester_template_id -> Nullable<Uuid>,
        target_template_id -> Nullable<Uuid>,
        #[max_length = 20]
        status -> Varchar,
        reason -> Nullable<Text>,
        requested_by_user_id -> Nullable<Uuid>,
        approved_by_user_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    shifts (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        is_active -> Bool,
        parent_shift_id -> Nullable<Uuid>,
        #[max_length = 10]
        report_symbol -> Nullable<Varchar>,
        location_id -> Nullable<Uuid>,
        employer_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    shift_templates (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        current_version_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    shift_versions (id) {
        id -> Uuid,
        template_id -> Uuid,
        workspace_id -> Uuid,
        version_number -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 10]
        report_symbol -> Nullable<Varchar>,
        location_id -> Nullable<Uuid>,
        employer_id -> Nullable<Uuid>,
        effective_from -> Date,
        effective_to -> Nullable<Date>,
        #[max_length = 20]
        change_type -> Varchar,
        change_reason -> Nullable<Text>,
        superseded_by -> Nullable<Uuid>,
        created_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    shift_audit_log (id) {
        id -> Uuid,
        template_id -> Uuid,
        version_id -> Uuid,
        #[max_length = 20]
        action -> Varchar,
        #[max_length = 20]
        change_type -> Varchar,
        changed_by -> Uuid,
        change_reason -> Nullable<Text>,
        old_values -> Nullable<Jsonb>,
        new_values -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    storage_configs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 255]
        endpoint_url -> Varchar,
        #[max_length = 50]
        region -> Varchar,
        #[max_length = 100]
        bucket_name -> Varchar,
        #[max_length = 255]
        path_prefix -> Nullable<Varchar>,
        #[max_length = 500]
        access_key_enc -> Varchar,
        #[max_length = 500]
        secret_key_enc -> Varchar,
        #[max_length = 50]
        encryption_key_version -> Varchar,
        use_path_style -> Bool,
        presigned_ttl_secs -> Int4,
        max_file_size -> Int8,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    system_audit_logs (id) {
        id -> Uuid,
        actor_id -> Nullable<Uuid>,
        #[max_length = 100]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    system_db_clusters (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        admin_database_url -> Varchar,
        #[max_length = 50]
        region -> Varchar,
        max_tenants -> Int4,
        active_tenants -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    system_files (id) {
        id -> Uuid,
        #[max_length = 255]
        file_name -> Varchar,
        #[max_length = 500]
        storage_key -> Varchar,
        file_size -> Int8,
        #[max_length = 100]
        mime_type -> Varchar,
        #[max_length = 64]
        content_hash -> Nullable<Varchar>,
        #[max_length = 50]
        category -> Varchar,
        #[max_length = 50]
        owner_type -> Varchar,
        owner_id -> Uuid,
        uploaded_by -> Nullable<Uuid>,
        is_active -> Bool,
        #[max_length = 20]
        scan_status -> Varchar,
        version -> Int4,
        parent_file_id -> Nullable<Uuid>,
        is_latest -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    system_settings (key) {
        #[max_length = 100]
        key -> Varchar,
        value_json -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    system_roles (id) {
        id -> Uuid,
        name -> Text,
        description -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    time_entries (id) {
        id -> Uuid,
        punch_in_time -> Timestamptz,
        punch_out_time -> Nullable<Timestamptz>,
        duration_minutes -> Nullable<Int4>,
        #[max_length = 50]
        entry_type -> Varchar,
        note -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        is_break_overtime -> Bool,
        template_id -> Nullable<Uuid>,
        generation_metadata -> Nullable<Jsonb>,
        workspace_id -> Uuid,
        employment_id -> Uuid,
    }
}

diesel::table! {
    shift_default_intervals (id) {
        id -> Uuid,
        version_id -> Uuid,
        sort_order -> Int4,
        #[max_length = 100]
        interval_name -> Nullable<Varchar>,
        start_time -> Time,
        end_time -> Time,
        break_duration_minutes -> Int4,
        break_start -> Nullable<Time>,
        break_end -> Nullable<Time>,
        is_break_auto_deduct -> Bool,
        break_is_overtime -> Bool,
        early_punch_in_grace_period_minutes -> Int4,
        allow_early_ot -> Bool,
        max_early_ot_minutes -> Nullable<Int4>,
        #[max_length = 30]
        ot_aggregation_mode -> Varchar,
        is_night_shift -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::{SegmentType, GapCompensationType, SegmentStatusType};

    work_segments (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employment_id -> Uuid,
        date -> Date,
        template_id -> Uuid,
        interval_id -> Nullable<Uuid>,
        sort_order -> Int4,
        scheduled_start -> Time,
        scheduled_end -> Time,
        segment_type -> SegmentType,
        segment_status -> SegmentStatusType,
        compensation_rate -> Numeric,
        gap_compensation_before -> GapCompensationType,
        gap_compensation_rate_before -> Numeric,
        break_config_override -> Nullable<Jsonb>,
        ot_config_override -> Nullable<Jsonb>,
        #[max_length = 30]
        source -> Varchar,
        source_detail -> Jsonb,
        notes -> Nullable<Text>,
        generation_metadata -> Jsonb,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_system_roles (user_id, role_id) {
        user_id -> Uuid,
        role_id -> Uuid,
        assigned_by -> Nullable<Uuid>,
        assigned_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Nullable<Timestamptz>,
        #[max_length = 100]
        full_name -> Nullable<Varchar>,
        avatar_url -> Nullable<Text>,
        bio -> Nullable<Text>,
        #[max_length = 10]
        preferred_language -> Nullable<Varchar>,
        avatar_file_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    work_locations (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        address -> Nullable<Text>,
        #[max_length = 50]
        timezone -> Varchar,
        is_headquarters -> Bool,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    working_conditions (id) {
        id -> Uuid,
        #[sql_name = "type"]
        #[max_length = 50]
        type_ -> Varchar,
        condition_type_id -> Nullable<Uuid>,
        minute_allowance -> Int4,
        is_paid -> Bool,
        start_date -> Date,
        end_date -> Nullable<Date>,
        created_at -> Nullable<Timestamptz>,
        employment_id -> Uuid,
    }
}

diesel::table! {
    working_pattern_details (id) {
        id -> Uuid,
        pattern_id -> Uuid,
        schedule_day -> Int4,
        template_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    working_patterns (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 10]
        pattern_type -> Varchar,
        cycle_length_days -> Nullable<Int4>,
        deleted_at -> Nullable<Timestamptz>,
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
    workspace_member_extra_permissions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        permission -> Text,
        assigned_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        assigned_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    workspace_members (workspace_id, user_id) {
        workspace_id -> Uuid,
        user_id -> Uuid,
        role_id -> Uuid,
    }
}

diesel::table! {
    workspace_role_permissions (role_id, permission_name) {
        role_id -> Uuid,
        permission_name -> Text,
    }
}

diesel::table! {
    workspace_roles (id) {
        id -> Uuid,
        workspace_id -> Nullable<Uuid>,
        name -> Text,
        description -> Nullable<Text>,
        color -> Nullable<Text>,
        is_system -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 16]
        scope_type -> Varchar,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        created_at -> Nullable<Timestamptz>,
        #[max_length = 20]
        date_format -> Varchar,
        #[max_length = 10]
        time_format -> Varchar,
        #[max_length = 50]
        timezone -> Varchar,
        theme_primary_color -> Nullable<Text>,
        #[max_length = 10]
        default_language -> Varchar,
        logo_url -> Nullable<Text>,
        tagline -> Nullable<Text>,
        custom_db_url_encrypted -> Nullable<Text>,
    }
}

diesel::table! {
    workspace_files (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 255]
        file_name -> Varchar,
        #[max_length = 500]
        storage_key -> Varchar,
        file_size -> Int8,
        #[max_length = 100]
        mime_type -> Varchar,
        #[max_length = 64]
        content_hash -> Nullable<Varchar>,
        #[max_length = 50]
        category -> Varchar,
        folder_id -> Nullable<Uuid>,
        #[max_length = 50]
        owner_type -> Varchar,
        owner_id -> Uuid,
        uploaded_by -> Nullable<Uuid>,
        is_active -> Bool,
        expires_at -> Nullable<Timestamptz>,
        #[max_length = 20]
        scan_status -> Varchar,
        scanned_at -> Nullable<Timestamptz>,
        version -> Int4,
        parent_file_id -> Nullable<Uuid>,
        is_latest -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    document_folders (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        parent_folder_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    signature_requests (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        document_id -> Uuid,
        signer_id -> Uuid,
        #[max_length = 50]
        status -> Varchar,
        signed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    file_acknowledgements (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        document_id -> Uuid,
        employee_id -> Uuid,
        #[max_length = 50]
        status -> Varchar,
        acknowledged_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(approval_workflow_instances -> approval_workflows (workflow_id));
diesel::joinable!(approval_workflow_steps -> approval_workflows (workflow_id));
diesel::joinable!(approval_workflow_tasks -> approval_workflow_instances (instance_id));
diesel::joinable!(assignment_rules -> working_patterns (pattern_id));
diesel::joinable!(attendance_adjustment_requests -> employments (employment_id));
diesel::joinable!(attendance_adjustment_requests -> time_entries (time_entry_id));
diesel::joinable!(attendance_daily_summaries -> employments (employment_id));
diesel::joinable!(attendance_policy_assignments -> attendance_policies (policy_id));
diesel::joinable!(attendance_policy_versions -> attendance_policies (policy_id));
diesel::joinable!(behalf_assignments -> departments (target_department_id));
diesel::joinable!(behalf_assignments -> employees (target_employee_id));
diesel::joinable!(behalf_assignments -> employers (employer_id));
diesel::joinable!(custom_field_values -> custom_field_definitions (definition_id));
diesel::joinable!(departments -> employees (manager_id));
diesel::joinable!(departments -> working_patterns (default_working_pattern_id));
diesel::joinable!(employee_assignments -> employments (employment_id));
diesel::joinable!(employee_contracts -> employee_documents (document_id));
diesel::joinable!(employee_contracts -> employments (employment_id));
diesel::joinable!(employee_documents -> employees (employee_id));
diesel::joinable!(employee_emergency_contacts -> employees (employee_id));
diesel::joinable!(employee_phases -> employment_phase_definitions (phase_definition_id));
diesel::joinable!(employee_phases -> employments (employment_id));
diesel::joinable!(employee_profiles -> employees (employee_id));
diesel::joinable!(employee_salary_components -> employments (employment_id));
diesel::joinable!(employee_salary_components -> salary_component_definitions (component_definition_id));
diesel::joinable!(employee_shifts -> employments (employment_id));
diesel::joinable!(employee_shifts -> shift_templates (template_id));
diesel::joinable!(employee_working_patterns -> employments (employment_id));
diesel::joinable!(employee_working_patterns -> working_patterns (pattern_id));
diesel::joinable!(employments -> departments (department_id));
diesel::joinable!(employments -> employees (employee_id));
diesel::joinable!(employments -> employers (employer_id));
diesel::joinable!(employments -> work_locations (primary_location_id));
diesel::joinable!(holidays -> leave_types (leave_type_id));
diesel::joinable!(holidays -> work_locations (location_id));
diesel::joinable!(leave_accrual_logs -> employments (employment_id));
diesel::joinable!(leave_accrual_logs -> leave_policy_assignments (policy_assignment_id));
diesel::joinable!(leave_accrual_logs -> leave_types (leave_type_id));
diesel::joinable!(leave_balances -> employments (employment_id));
diesel::joinable!(leave_balances -> leave_types (leave_type_id));
diesel::joinable!(leave_policy_assignments -> leave_policy_definitions (policy_id));
diesel::joinable!(leave_policy_definitions -> leave_types (leave_type_id));
diesel::joinable!(leave_requests -> employments (employment_id));
diesel::joinable!(leave_requests -> leave_types (leave_type_id));
diesel::joinable!(lifecycle_checklist_items -> lifecycle_checklist_template_items (template_item_id));
diesel::joinable!(lifecycle_checklist_items -> lifecycle_checklists (checklist_id));
diesel::joinable!(lifecycle_checklist_template_items -> lifecycle_checklist_templates (template_id));
diesel::joinable!(lifecycle_checklists -> lifecycle_checklist_templates (template_id));
diesel::joinable!(member_extra_roles -> users (user_id));
diesel::joinable!(member_extra_roles -> workspace_roles (role_id));
diesel::joinable!(member_extra_roles -> workspaces (workspace_id));
diesel::joinable!(notifications -> users (user_id));
diesel::joinable!(overtime_policy_assignments -> overtime_policy_definitions (policy_id));
diesel::joinable!(overtime_request_employees -> attendance_daily_summaries (matched_summary_id));
diesel::joinable!(overtime_request_employees -> employments (employment_id));
diesel::joinable!(overtime_requests -> approval_workflows (workflow_id));
diesel::joinable!(overtime_requests -> departments (department_id));
diesel::joinable!(overtime_rules -> overtime_policy_definitions (policy_id));
diesel::joinable!(payroll_incentive_assignments -> payroll_incentive_policies (policy_id));
diesel::joinable!(payroll_incentive_tiers -> payroll_incentive_policies (policy_id));
diesel::joinable!(payroll_rule_assignments -> payroll_rule_definitions (rule_definition_id));
diesel::joinable!(payroll_rule_tiers -> payroll_rule_definitions (rule_definition_id));
diesel::joinable!(payslip_incentives -> payslips (payslip_id));
diesel::joinable!(payslip_rule_evaluations -> payslips (payslip_id));
diesel::joinable!(payslip_segments -> departments (department_id));
diesel::joinable!(payslip_segments -> payslips (payslip_id));
diesel::joinable!(payslips -> employments (employment_id));
diesel::joinable!(raw_attendance_logs -> employments (employment_id));
diesel::joinable!(salary_component_base_members -> salary_base_groups (base_group_id));
diesel::joinable!(salary_component_base_members -> salary_component_definitions (component_definition_id));
diesel::joinable!(salary_component_definitions -> payroll_rule_definitions (linked_rule_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(shift_rosters -> employments (employment_id));
diesel::joinable!(shift_rosters -> shift_templates (template_id));
diesel::joinable!(shifts -> employers (employer_id));
diesel::joinable!(shifts -> work_locations (location_id));
diesel::joinable!(shift_versions -> shift_templates (template_id));
diesel::joinable!(shift_audit_log -> shift_templates (template_id));
diesel::joinable!(shift_audit_log -> shift_versions (version_id));
diesel::joinable!(storage_configs -> workspaces (workspace_id));
diesel::joinable!(system_audit_logs -> users (actor_id));
diesel::joinable!(shift_default_intervals -> shift_versions (version_id));
diesel::joinable!(time_entries -> employments (employment_id));
diesel::joinable!(time_entries -> shift_templates (template_id));
diesel::joinable!(work_segments -> employments (employment_id));
diesel::joinable!(work_segments -> shift_default_intervals (interval_id));
diesel::joinable!(work_segments -> shift_templates (template_id));
diesel::joinable!(user_system_roles -> system_roles (role_id));
diesel::joinable!(users -> system_files (avatar_file_id));
diesel::joinable!(working_conditions -> condition_type_definitions (condition_type_id));
diesel::joinable!(working_conditions -> employments (employment_id));
diesel::joinable!(working_pattern_details -> shift_templates (template_id));
diesel::joinable!(working_pattern_details -> working_patterns (pattern_id));
diesel::joinable!(workspace_member_extra_permissions -> users (user_id));
diesel::joinable!(workspace_member_extra_permissions -> workspaces (workspace_id));
diesel::joinable!(workspace_members -> users (user_id));
diesel::joinable!(workspace_members -> workspace_roles (role_id));
diesel::joinable!(workspace_members -> workspaces (workspace_id));
diesel::joinable!(workspace_role_permissions -> workspace_roles (role_id));

diesel::joinable!(workspace_files -> workspaces (workspace_id));
diesel::joinable!(workspace_files -> document_folders (folder_id));
diesel::joinable!(employee_profiles -> workspace_files (avatar_file_id));

diesel::joinable!(document_folders -> workspaces (workspace_id));
diesel::joinable!(signature_requests -> workspaces (workspace_id));
diesel::joinable!(signature_requests -> workspace_files (document_id));
diesel::joinable!(signature_requests -> employees (signer_id));
diesel::joinable!(file_acknowledgements -> workspaces (workspace_id));
diesel::joinable!(file_acknowledgements -> workspace_files (document_id));
diesel::joinable!(file_acknowledgements -> employees (employee_id));

diesel::allow_tables_to_appear_in_same_query!(
    approval_action_log,
    approval_workflow_instances,
    approval_workflow_steps,
    approval_workflow_tasks,
    approval_workflows,
    assignment_rules,
    attendance_adjustment_requests,
    attendance_daily_summaries,
    attendance_import_jobs,
    attendance_log_errors,
    attendance_policies,
    attendance_policy_assignments,
    attendance_policy_versions,
    attendance_staging_logs,
    attendance_staging_logs_default,
    attendance_staging_logs_y2026m01,
    attendance_staging_logs_y2026m02,
    attendance_staging_logs_y2026m03,
    attendance_staging_logs_y2026m04,
    attendance_staging_logs_y2026m05,
    attendance_staging_logs_y2026m06,
    attendance_staging_logs_y2026m07,
    attendance_staging_logs_y2026m08,
    attendance_staging_logs_y2026m09,
    attendance_staging_logs_y2026m10,
    attendance_staging_logs_y2026m11,
    attendance_staging_logs_y2026m12,
    audit_logs,
    audit_logs_default,
    audit_logs_y2026m01,
    audit_logs_y2026m02,
    audit_logs_y2026m03,
    audit_logs_y2026m04,
    audit_logs_y2026m05,
    audit_logs_y2026m06,
    audit_logs_y2026m07,
    audit_logs_y2026m08,
    audit_logs_y2026m09,
    audit_logs_y2026m10,
    audit_logs_y2026m11,
    audit_logs_y2026m12,
    authorized_agents,
    behalf_assignments,
    condition_type_definitions,
    custom_field_definitions,
    custom_field_values,
    dashboard_preferences,
    departments,
    device_enrollments,
    devices,
    employee_assignments,
    employee_biometric_templates,
    employee_contracts,
    employee_documents,
    employee_emergency_contacts,
    employee_phases,
    employee_profiles,
    employee_salary_components,
    employee_shifts,
    employee_working_patterns,
    employees,
    employers,
    employment_phase_definitions,
    employments,
    feature_flags,
    holidays,
    leave_accrual_logs,
    leave_balances,
    leave_policy_assignments,
    leave_policy_definitions,
    leave_requests,
    leave_types,
    lifecycle_checklist_items,
    lifecycle_checklist_template_items,
    lifecycle_checklist_templates,
    lifecycle_checklists,
    member_extra_roles,
    notifications,
    outbox_jobs,
    overtime_policy_assignments,
    overtime_policy_definitions,
    overtime_request_employees,
    overtime_requests,
    overtime_rules,
    payroll_incentive_assignments,
    payroll_incentive_policies,
    payroll_incentive_tiers,
    payroll_rule_assignments,
    payroll_rule_definitions,
    payroll_rule_tiers,
    payslip_incentives,
    payslip_rule_evaluations,
    payslip_segments,
    payslips,
    policy_templates,
    raw_attendance_logs,
    recruitment_sources,
    salary_base_groups,
    salary_component_base_members,
    salary_component_definitions,
    sessions,
    shift_audit_log,
    shift_default_intervals,
    shift_rosters,
    shift_swap_requests,
    shift_templates,
    shift_versions,
    shifts,
    storage_configs,
    system_audit_logs,
    system_db_clusters,
    system_files,
    system_settings,
    system_roles,
    time_entries,
    user_system_roles,
    users,
    work_locations,
    working_conditions,
    working_pattern_details,
    work_segments,
    working_patterns,
    workspace_config_entries,
    workspace_member_extra_permissions,
    workspace_members,
    workspace_role_permissions,
    workspace_roles,
    workspace_files,
    document_folders,
    signature_requests,
    file_acknowledgements,
    workspaces,
);

diesel::allow_columns_to_appear_in_same_group_by_clause!(
    employments::employee_id,
    leave_types::code,
);
