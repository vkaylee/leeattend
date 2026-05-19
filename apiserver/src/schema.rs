// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "adjustment_status_type"))]
    pub struct AdjustmentStatusType;

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
    #[diesel(postgres_type(name = "ot_status_type"))]
    pub struct OtStatusType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "recruitment_source_type"))]
    pub struct RecruitmentSourceType;
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
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AdjustmentStatusType;

    attendance_adjustment_requests (id) {
        id -> Uuid,
        employee_id -> Uuid,
        time_entry_id -> Nullable<Uuid>,
        requested_punch_in_time -> Timestamptz,
        requested_punch_out_time -> Nullable<Timestamptz>,
        reason -> Nullable<Text>,
        status -> AdjustmentStatusType,
        requested_by_user_id -> Nullable<Uuid>,
        approved_by_user_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        is_break_overtime -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OtStatusType;

    attendance_daily_summaries (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
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
    department_default_shifts (department_id, shift_id) {
        department_id -> Uuid,
        shift_id -> Uuid,
    }
}

diesel::table! {
    department_payroll_configs (id) {
        id -> Uuid,
        department_id -> Uuid,
        std_working_days -> Numeric,
        effective_date -> Date,
        created_at -> Nullable<Timestamptz>,
        hours_per_day -> Numeric,
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
        default_working_pattern_id -> Nullable<Uuid>,
        std_working_days -> Nullable<Numeric>,
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
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ContractTypeEnum;
    use super::sql_types::ContractStatusEnum;

    employee_contracts (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
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
    }
}

diesel::table! {
    employee_department_history (id) {
        id -> Uuid,
        employee_id -> Uuid,
        department_id -> Uuid,
        effective_date -> Date,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
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
    employee_payroll_configs (id) {
        id -> Uuid,
        employee_id -> Uuid,
        std_working_days -> Numeric,
        effective_date -> Date,
        created_at -> Nullable<Timestamptz>,
        hours_per_day -> Numeric,
    }
}

diesel::table! {
    employee_phases (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
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
        #[max_length = 50]
        national_id -> Nullable<Varchar>,
        #[max_length = 100]
        bank_name -> Nullable<Varchar>,
        #[max_length = 50]
        bank_account_number -> Nullable<Varchar>,
        #[max_length = 100]
        bank_branch -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    employee_salary_components (id) {
        id -> Uuid,
        employee_id -> Uuid,
        component_definition_id -> Uuid,
        amount -> Numeric,
        effective_date -> Date,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    employee_shifts (id) {
        id -> Uuid,
        employee_id -> Uuid,
        shift_id -> Uuid,
        start_date -> Date,
        end_date -> Nullable<Date>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    employee_working_patterns (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
        pattern_id -> Uuid,
        start_date -> Date,
        end_date -> Nullable<Date>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    employees (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 50]
        employee_id_number -> Varchar,
        is_active -> Bool,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        tags -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 255]
        full_name -> Varchar,
        recruitment_source_id -> Nullable<Uuid>,
        hire_date -> Date,
        termination_date -> Nullable<Date>,
        original_hire_date -> Date,
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
    }
}

diesel::table! {
    leave_accrual_logs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        employee_id -> Uuid,
        policy_assignment_id -> Nullable<Uuid>,
        accrual_date -> Date,
        amount -> Numeric,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        leave_type_id -> Uuid,
    }
}

diesel::table! {
    leave_balances (id) {
        id -> Uuid,
        employee_id -> Uuid,
        year -> Int4,
        entitlement -> Int4,
        carried_over -> Int4,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        accrued_amount -> Numeric,
        leave_type_id -> Uuid,
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
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LeaveStatusType;

    leave_requests (id) {
        id -> Uuid,
        employee_id -> Uuid,
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
    payroll_settings (workspace_id, effective_date) {
        workspace_id -> Uuid,
        std_working_day_mode -> Text,
        fixed_std_days -> Numeric,
        working_days_per_week -> Int4,
        hours_per_day -> Numeric,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        effective_date -> Date,
        max_ot_hours_per_day -> Nullable<Numeric>,
        max_ot_hours_per_month -> Nullable<Numeric>,
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
        employee_id -> Uuid,
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
        employee_id -> Uuid,
        punch_time -> Timestamptz,
        source -> LogSourceType,
        #[max_length = 50]
        device_id -> Nullable<Varchar>,
        raw_data -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        punch -> Int4,
        status -> Int4,
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
        employee_id -> Uuid,
        shift_id -> Nullable<Uuid>,
        date -> Date,
        #[max_length = 20]
        status -> Nullable<Varchar>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    shift_swap_requests (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        requester_id -> Uuid,
        target_id -> Uuid,
        swap_date -> Date,
        requester_shift_id -> Nullable<Uuid>,
        target_shift_id -> Nullable<Uuid>,
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
        start_time -> Time,
        end_time -> Time,
        break_duration_minutes -> Int4,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        break_is_overtime -> Bool,
        early_punch_in_grace_period_minutes -> Int4,
        break_start -> Nullable<Time>,
        break_end -> Nullable<Time>,
        is_break_auto_deduct -> Bool,
        is_active -> Bool,
        parent_shift_id -> Nullable<Uuid>,
        allow_early_ot -> Bool,
        #[max_length = 10]
        report_symbol -> Nullable<Varchar>,
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
        employee_id -> Uuid,
        punch_in_time -> Timestamptz,
        punch_out_time -> Nullable<Timestamptz>,
        duration_minutes -> Nullable<Int4>,
        #[max_length = 50]
        entry_type -> Varchar,
        note -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        is_break_overtime -> Bool,
        shift_id -> Nullable<Uuid>,
        generation_metadata -> Nullable<Jsonb>,
        workspace_id -> Uuid,
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
    }
}

diesel::table! {
    working_conditions (id) {
        id -> Uuid,
        employee_id -> Uuid,
        #[sql_name = "type"]
        #[max_length = 50]
        type_ -> Varchar,
        minute_allowance -> Int4,
        is_paid -> Bool,
        start_date -> Date,
        end_date -> Nullable<Date>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    working_pattern_details (id) {
        id -> Uuid,
        pattern_id -> Uuid,
        day_of_week -> Int4,
        shift_id -> Nullable<Uuid>,
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
        #[max_length = 30]
        holiday_overnight_strategy -> Varchar,
        default_holiday_ot_multiplier -> Numeric,
        logo_url -> Nullable<Text>,
        tagline -> Nullable<Text>,
        custom_db_url_encrypted -> Nullable<Text>,
    }
}

diesel::joinable!(assignment_rules -> working_patterns (pattern_id));
diesel::joinable!(attendance_adjustment_requests -> employees (employee_id));
diesel::joinable!(attendance_adjustment_requests -> time_entries (time_entry_id));
diesel::joinable!(attendance_daily_summaries -> employees (employee_id));
diesel::joinable!(attendance_policy_assignments -> attendance_policies (policy_id));
diesel::joinable!(attendance_policy_versions -> attendance_policies (policy_id));
diesel::joinable!(authorized_agents -> workspaces (workspace_id));
diesel::joinable!(department_default_shifts -> departments (department_id));
diesel::joinable!(department_default_shifts -> shifts (shift_id));
diesel::joinable!(department_payroll_configs -> departments (department_id));
diesel::joinable!(departments -> employees (manager_id));
diesel::joinable!(departments -> working_patterns (default_working_pattern_id));
diesel::joinable!(employee_contracts -> employee_documents (document_id));
diesel::joinable!(employee_contracts -> employees (employee_id));
diesel::joinable!(employee_department_history -> departments (department_id));
diesel::joinable!(employee_department_history -> employees (employee_id));
diesel::joinable!(employee_documents -> employees (employee_id));
diesel::joinable!(employee_emergency_contacts -> employees (employee_id));
diesel::joinable!(employee_payroll_configs -> employees (employee_id));
diesel::joinable!(employee_phases -> employees (employee_id));
diesel::joinable!(employee_phases -> employment_phase_definitions (phase_definition_id));
diesel::joinable!(employee_profiles -> employees (employee_id));
diesel::joinable!(employee_salary_components -> employees (employee_id));
diesel::joinable!(employee_salary_components -> salary_component_definitions (component_definition_id));
diesel::joinable!(employee_shifts -> employees (employee_id));
diesel::joinable!(employee_shifts -> shifts (shift_id));
diesel::joinable!(employee_working_patterns -> employees (employee_id));
diesel::joinable!(employee_working_patterns -> working_patterns (pattern_id));
diesel::joinable!(holidays -> leave_types (leave_type_id));
diesel::joinable!(leave_accrual_logs -> employees (employee_id));
diesel::joinable!(leave_accrual_logs -> leave_policy_assignments (policy_assignment_id));
diesel::joinable!(leave_accrual_logs -> leave_types (leave_type_id));
diesel::joinable!(leave_balances -> employees (employee_id));
diesel::joinable!(leave_balances -> leave_types (leave_type_id));
diesel::joinable!(leave_policy_assignments -> leave_policy_definitions (policy_id));
diesel::joinable!(leave_policy_definitions -> leave_types (leave_type_id));
diesel::joinable!(leave_requests -> employees (employee_id));
diesel::joinable!(leave_requests -> leave_types (leave_type_id));
diesel::joinable!(member_extra_roles -> workspace_roles (role_id));
diesel::joinable!(overtime_policy_assignments -> overtime_policy_definitions (policy_id));
diesel::joinable!(overtime_rules -> overtime_policy_definitions (policy_id));
diesel::joinable!(payroll_incentive_assignments -> payroll_incentive_policies (policy_id));
diesel::joinable!(payroll_incentive_tiers -> payroll_incentive_policies (policy_id));
diesel::joinable!(payroll_rule_assignments -> payroll_rule_definitions (rule_definition_id));
diesel::joinable!(payroll_rule_tiers -> payroll_rule_definitions (rule_definition_id));
diesel::joinable!(payslip_incentives -> payslips (payslip_id));
diesel::joinable!(payslip_rule_evaluations -> payslips (payslip_id));
diesel::joinable!(payslip_segments -> departments (department_id));
diesel::joinable!(payslip_segments -> payslips (payslip_id));
diesel::joinable!(payslips -> employees (employee_id));
diesel::joinable!(raw_attendance_logs -> employees (employee_id));
diesel::joinable!(salary_component_base_members -> salary_base_groups (base_group_id));
diesel::joinable!(salary_component_base_members -> salary_component_definitions (component_definition_id));
diesel::joinable!(salary_component_definitions -> payroll_rule_definitions (linked_rule_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(shift_rosters -> employees (employee_id));
diesel::joinable!(shift_rosters -> shifts (shift_id));
diesel::joinable!(system_audit_logs -> users (actor_id));
diesel::joinable!(time_entries -> employees (employee_id));
diesel::joinable!(time_entries -> shifts (shift_id));
diesel::joinable!(user_system_roles -> system_roles (role_id));
diesel::joinable!(working_conditions -> employees (employee_id));
diesel::joinable!(working_pattern_details -> shifts (shift_id));
diesel::joinable!(working_pattern_details -> working_patterns (pattern_id));
diesel::joinable!(workspace_members -> workspace_roles (role_id));
diesel::joinable!(workspace_role_permissions -> workspace_roles (role_id));

diesel::allow_tables_to_appear_in_same_query!(
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
    dashboard_preferences,
    department_default_shifts,
    department_payroll_configs,
    departments,
    devices,
    employee_contracts,
    employee_department_history,
    employee_documents,
    employee_emergency_contacts,
    employee_payroll_configs,
    employee_phases,
    employee_profiles,
    employee_salary_components,
    employee_shifts,
    employee_working_patterns,
    employees,
    employment_phase_definitions,
    feature_flags,
    holidays,
    leave_accrual_logs,
    leave_balances,
    leave_policy_assignments,
    leave_policy_definitions,
    leave_requests,
    leave_types,
    member_extra_roles,
    notifications,
    overtime_policy_assignments,
    overtime_policy_definitions,
    overtime_rules,
    payroll_incentive_assignments,
    payroll_incentive_policies,
    payroll_incentive_tiers,
    payroll_rule_assignments,
    payroll_rule_definitions,
    payroll_rule_tiers,
    payroll_settings,
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
    shift_rosters,
    shift_swap_requests,
    shifts,
    system_audit_logs,
    system_roles,
    time_entries,
    user_system_roles,
    users,
    working_conditions,
    working_pattern_details,
    working_patterns,
    workspace_member_extra_permissions,
    workspace_members,
    workspace_role_permissions,
    workspace_roles,
    workspaces,
);
