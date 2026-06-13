#![allow(clippy::needless_for_each)]
use crate::models::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::get_devices,
        crate::handlers::get_agents,
        crate::handlers::attendance::time_tracking::punch_in,
        crate::handlers::attendance::time_tracking::punch_out,
        crate::handlers::attendance::time_tracking::get_time_history,
        crate::handlers::attendance::logs::get_attendance_logs,
        crate::handlers::attendance::logs::get_raw_attendance_logs,
        crate::handlers::workspace_config::list_configs,
        crate::handlers::workspace_config::get_config,
        crate::handlers::workspace_config::upsert_config,
        crate::handlers::workspace_config::delete_config,

        // Auth Paths
        crate::handlers::register,
        crate::handlers::login,
        crate::handlers::refresh_token,
        crate::handlers::change_password,
        crate::handlers::logout,
        crate::handlers::get_me,
        crate::handlers::get_my_permissions,
        crate::handlers::update_me,
        crate::handlers::get_users,

        // ESS Paths
        crate::handlers::get_my_profile,
        crate::handlers::update_my_profile,
        crate::handlers::get_my_emergency_contacts,
        crate::handlers::add_emergency_contact,
        crate::handlers::delete_emergency_contact,
        crate::handlers::get_my_documents,
        crate::handlers::get_my_payslips,
        crate::handlers::get_my_attendance,
        crate::handlers::ess_punch,
        crate::handlers::get_my_time_entries,
        crate::handlers::get_my_leave_balances,
        crate::handlers::ess_get_my_leave_requests,
        crate::handlers::ess_get_my_shift_swaps,
        crate::handlers::ess_create_shift_swap,
        crate::handlers::ess_respond_to_swap,
        crate::handlers::ess_cancel_swap,

        // Workspace Paths
        crate::handlers::get_attendance_trend,
        crate::handlers::get_department_attendance,
        crate::handlers::get_unexcused_absences,
        crate::handlers::get_overtime_trend,
        crate::handlers::get_workspace_stats,
        crate::handlers::get_peak_times,
        crate::handlers::get_leave_types_breakdown,
        crate::handlers::get_lateness_by_day,
        crate::handlers::get_workspaces,
        crate::handlers::get_workspace_by_id,
        crate::handlers::create_workspace,
        crate::handlers::update_workspace,

        // Locations
        crate::handlers::locations::list_locations,
        crate::handlers::locations::get_location,
        crate::handlers::locations::create_location,
        crate::handlers::locations::update_location,
        crate::handlers::locations::delete_location,

        // Employers
        crate::handlers::workspaces::list_employers,
        crate::handlers::workspaces::get_employer,
        crate::handlers::workspaces::create_employer,
        crate::handlers::workspaces::update_employer,
        crate::handlers::workspaces::delete_employer,

        // Phase 3: Staff Management & Rosters
        crate::handlers::get_working_conditions,
        crate::handlers::create_working_condition,
        crate::handlers::delete_working_condition,
        crate::handlers::get_employees,
        crate::handlers::get_employee,
        crate::handlers::create_employee,
        crate::handlers::update_employee,
        crate::handlers::delete_employee,
        crate::handlers::rehire_employee,
        crate::handlers::get_employee_department_history,
        crate::handlers::transfer_employee,
        crate::handlers::get_compensation_history,

        crate::handlers::employee_contracts::list_employee_contracts,
        crate::handlers::employee_contracts::get_contract,
        crate::handlers::employee_contracts::create_contract,
        crate::handlers::employee_contracts::update_contract,
        crate::handlers::employee_contracts::delete_contract,

        crate::handlers::employments::list_employee_employments,
        crate::handlers::employments::get_employment,
        crate::handlers::employments::create_employment,
        crate::handlers::employments::update_employment,
        crate::handlers::employments::delete_employment,

        crate::handlers::list_employee_assignments,
        crate::handlers::get_employee_assignment,
        crate::handlers::create_employee_assignment,
        crate::handlers::update_employee_assignment,
        crate::handlers::delete_employee_assignment,

        crate::handlers::get_shifts,
        crate::handlers::create_shift,
        crate::handlers::update_shift,
        crate::handlers::assign_shift,
        crate::handlers::get_workspace_schedules,
        crate::handlers::get_assignment_history,
        crate::handlers::get_employee_rosters,
        crate::handlers::upsert_roster,
        crate::handlers::bulk_upsert_roster,
        crate::handlers::delete_roster,

        crate::handlers::working_patterns::create_pattern,
        crate::handlers::working_patterns::list_patterns,
        crate::handlers::working_patterns::get_pattern_details,
        crate::handlers::working_patterns::assign_pattern,
        crate::handlers::working_patterns::bulk_assign_pattern,
        crate::handlers::working_patterns::get_employment_pattern_history,
        crate::handlers::working_patterns::delete_pattern,
        crate::handlers::working_patterns::update_pattern,

        crate::handlers::shift_swaps::create_shift_swap,
        crate::handlers::shift_swaps::list_shift_swaps,
        crate::handlers::shift_swaps::approve_shift_swap,
        crate::handlers::shift_swaps::get_shift_swap_stats,

        // Phase 4: Leave Requests & Overtime Claims
        crate::handlers::leaves::submit_leave_request,
        crate::handlers::leaves::approve_leave_request,
        crate::handlers::leaves::get_my_leave_requests,
        crate::handlers::leaves::get_workspace_leave_balances,
        crate::handlers::leaves::get_employee_leave_balances,
        crate::handlers::leaves::get_workspace_leave_requests,
        crate::handlers::leaves::get_employee_leave_history,
        crate::handlers::leaves::adjust_leave_balance,
        crate::handlers::leaves::create_leave_policy,
        crate::handlers::leaves::update_leave_policy,
        crate::handlers::leaves::delete_leave_policy,
        crate::handlers::leaves::get_leave_policies,
        crate::handlers::leaves::assign_leave_policy,
        crate::handlers::leaves::get_leave_policy_assignments,
        crate::handlers::leaves::trigger_accrual_run,
        crate::handlers::leaves::get_job_status,
        crate::handlers::leaves::upload_leave_excel,
        crate::handlers::leaves::get_latest_leave_import,
        crate::handlers::leaves::list_leave_behalf_assignments,
        crate::handlers::leaves::create_leave_behalf_assignment,
        crate::handlers::leaves::delete_leave_behalf_assignment,
        crate::handlers::leaves::get_my_leave_behalf_targets,
        crate::handlers::leaves::download_leave_import_errors,
        crate::handlers::leaves::download_leave_import_template,
        crate::handlers::leave_types::list_leave_types,
        crate::handlers::leave_types::create_leave_type,
        crate::handlers::leave_types::update_leave_type_config,
        crate::handlers::leave_types::delete_leave_type,
        crate::handlers::holidays::list_holidays,
        crate::handlers::holidays::get_holiday,
        crate::handlers::holidays::create_holiday,
        crate::handlers::holidays::update_holiday,
        crate::handlers::holidays::delete_holiday,
        crate::handlers::holidays::process_holiday_leave,
        crate::handlers::overtime_request::create_ot_request,
        crate::handlers::overtime_request::list_ot_requests,
        crate::handlers::overtime_request::get_ot_request,
        crate::handlers::overtime_request::submit_for_approval,
        crate::handlers::overtime_request::employee_consent,
        crate::handlers::overtime_request::list_request_behalf_assignments,
        crate::handlers::overtime_request::create_request_behalf_assignment,
        crate::handlers::overtime_request::delete_request_behalf_assignment,
        crate::handlers::overtime_request::get_my_request_behalf_targets,
        crate::handlers::overtime::list_policies,
        crate::handlers::overtime::get_policy,
        crate::handlers::overtime::create_policy,
        crate::handlers::overtime::add_rule,
        crate::handlers::overtime::assign_policy,

        // Phase 5: Compensation, Workflows & Webhooks
        crate::handlers::payroll::payslips::calculate_batch_payroll,
        crate::handlers::payroll::payslips::get_workspace_payslips,
        crate::handlers::payroll::payslips::calculate_payroll,
        crate::handlers::payroll::payslips::get_employee_payslips,
        crate::handlers::payroll::settings::get_payroll_settings,
        crate::handlers::payroll::settings::get_payroll_settings_history,
        crate::handlers::payroll::settings::update_payroll_settings,
        crate::handlers::payroll::salary_components::get_salary_components,
        crate::handlers::payroll::salary_components::create_salary_component,
        crate::handlers::payroll::salary_components::get_employment_salary_components,
        crate::handlers::payroll::salary_components::assign_employment_salary_component,
        crate::handlers::payroll::incentives::get_incentive_policies,
        crate::handlers::payroll::incentives::create_incentive_policy,
        crate::handlers::payroll::incentives::delete_incentive_policy,
        crate::handlers::payroll::incentives::clone_incentive_policy,
        crate::handlers::payroll::configs::set_department_payroll_config,
        crate::handlers::payroll::configs::set_employee_payroll_config,
        crate::handlers::payroll::configs::get_department_payroll_configs,
        crate::handlers::payroll::configs::get_employee_payroll_configs,
        crate::handlers::salary_bases::get_salary_bases,
        crate::handlers::salary_bases::create_salary_base,
        crate::handlers::salary_bases::assign_component_to_base,
        crate::handlers::salary_bases::unassign_component_from_base,
        crate::handlers::approval_workflow::list_workflows,
        crate::handlers::approval_workflow::create_workflow,
        crate::handlers::approval_workflow::get_workflow,
        crate::handlers::approval_workflow::update_workflow,
        crate::handlers::approval_workflow::delete_workflow,
        crate::handlers::approval_workflow::process_approval,
        crate::handlers::approval_workflow::get_approval_history,
        crate::handlers::approval_workflow::get_pending_tasks,
        crate::handlers::report_export::create_export_job,
        crate::handlers::report_export::get_job_status,
        crate::handlers::report_export::download_report,
        crate::handlers::report_export::cancel_job,

        // Payroll Rules Paths
        crate::handlers::payroll_rules::create_payroll_rule_definition,
        crate::handlers::payroll_rules::get_payroll_rule_definitions,
        crate::handlers::payroll_rules::create_payroll_rule_tier,
        crate::handlers::payroll_rules::get_payroll_rule_tiers,
        crate::handlers::payroll_rules::assign_payroll_rule,
        crate::handlers::payroll_rules::get_payroll_rule_assignments,
        crate::handlers::payroll_rules::delete_payroll_rule_assignment,
        crate::handlers::payroll_rules::delete_payroll_rule_tier,

        // Phase 6: Platform Admin & Infrastructure
        crate::handlers::system_admin::list_system_roles,
        crate::handlers::system_admin::get_user_system_roles,
        crate::handlers::system_admin::assign_system_role,
        crate::handlers::system_admin::revoke_system_role,
        crate::handlers::system_admin::system_health,
        crate::handlers::system_db_clusters::list_db_clusters,
        crate::handlers::system_db_clusters::create_db_cluster,
        crate::handlers::system_db_clusters::delete_db_cluster,
        crate::handlers::system_db_clusters::update_db_cluster_status,
        crate::handlers::worker_monitor::get_monitor_overview,
        crate::handlers::worker_monitor::get_monitor_jobs,
        crate::handlers::workspaces::audit::get_audit_logs,

        // Agent Paths
        crate::handlers::agents::send_command,
        crate::handlers::agents::create_device,
        crate::handlers::agents::update_device,
        crate::handlers::agents::delete_device,
        crate::handlers::agents::get_authorized_agents,
        crate::handlers::agents::create_authorized_agent,
        crate::handlers::agents::update_agent_trust,
        crate::handlers::agents::get_authorized_agent_by_id,
        crate::handlers::agents::update_authorized_agent,
        crate::handlers::agents::get_device_info_on_demand,
        crate::handlers::agents::delete_authorized_agent,

        // Notifications Paths
        crate::handlers::notifications::get_notifications,
        crate::handlers::notifications::get_unread_count,
        crate::handlers::notifications::mark_as_read,
        crate::handlers::notifications::mark_all_as_read,

        // Workspace Members Paths
        crate::handlers::workspaces::members::get_workspace_members,
        crate::handlers::workspaces::members::add_workspace_member,
        crate::handlers::workspaces::members::update_workspace_member_role,
        crate::handlers::workspaces::members::remove_workspace_member,

        // Workspace Permissions Paths
        crate::handlers::workspaces::permissions::get_member_extra_permissions,
        crate::handlers::workspaces::permissions::assign_extra_permission,
        crate::handlers::workspaces::permissions::revoke_extra_permission,
        crate::handlers::workspaces::permissions::get_available_permissions,

        // Workspace Roles Paths
        crate::handlers::workspaces::roles::get_member_roles,
        crate::handlers::workspaces::roles::assign_extra_role,
        crate::handlers::workspaces::roles::revoke_extra_role,
        crate::handlers::workspaces::roles::get_workspace_roles,
        crate::handlers::workspaces::roles::create_workspace_role,
        crate::handlers::workspaces::roles::update_workspace_role,
        crate::handlers::workspaces::roles::delete_workspace_role,
        crate::handlers::workspaces::roles::get_role_permissions,
        crate::handlers::workspaces::roles::update_role_permissions,

        // Departments Paths
        crate::handlers::workspaces::departments::get_departments,
        crate::handlers::workspaces::departments::create_department,
        crate::handlers::workspaces::departments::update_department,
        crate::handlers::workspaces::departments::delete_department,

        // Feature Flags Paths
        crate::handlers::feature_flags::list_feature_flags,
        crate::handlers::feature_flags::create_feature_flag,
        crate::handlers::feature_flags::get_feature_flag,
        crate::handlers::feature_flags::check_feature_flag,
        crate::handlers::feature_flags::update_feature_flag,
        crate::handlers::feature_flags::upsert_feature_flag,
        crate::handlers::feature_flags::delete_feature_flag,

        // Dashboard Preferences Paths
        crate::handlers::dashboard_preferences::get_dashboard_preferences,
        crate::handlers::dashboard_preferences::update_dashboard_preferences,

        // Employment Phases Paths
        crate::handlers::employment_phases::create_employment_phase_definition,
        crate::handlers::employment_phases::get_employment_phase_definitions,
        crate::handlers::employment_phases::assign_employee_phase,
        crate::handlers::employment_phases::update_employee_phase_status,
        crate::handlers::employment_phases::get_employee_phases,
        crate::handlers::employment_phases::assign_employment_phase,
        crate::handlers::employment_phases::update_employment_phase_status,
        crate::handlers::employment_phases::get_employment_phases,

        // Recruitment Sources Paths
        crate::handlers::recruitment_sources::get_recruitment_sources,
        crate::handlers::recruitment_sources::create_recruitment_source,
        crate::handlers::recruitment_sources::update_recruitment_source,
        crate::handlers::recruitment_sources::delete_recruitment_source,
        crate::handlers::recruitment_sources::get_recruitment_source_breakdown,
        crate::handlers::recruitment_sources::get_turnover_analytics,

        // Assignment Rules Paths
        crate::handlers::assignment_rules::get_assignment_rules,
        crate::handlers::assignment_rules::create_assignment_rule,
        crate::handlers::assignment_rules::update_assignment_rule,
        crate::handlers::assignment_rules::delete_assignment_rule,

        // Attendance Adjustments Paths
        crate::handlers::attendance::adjustments::submit_attendance_adjustment,
        crate::handlers::attendance::adjustments::approve_attendance_adjustment,
        crate::handlers::attendance::adjustments::get_my_attendance_adjustments,
        crate::handlers::attendance::adjustments::get_workspace_attendance_adjustments,
        crate::handlers::attendance::adjustments::create_adjustment_behalf_assignment,
        crate::handlers::attendance::adjustments::delete_adjustment_behalf_assignment,
        crate::handlers::attendance::adjustments::list_adjustment_behalf_assignments,
        crate::handlers::attendance::adjustments::get_my_adjustment_behalf_targets,

        // Attendance Overtime Paths
        crate::handlers::attendance::overtime::claim_ot,
        crate::handlers::attendance::overtime::approve_ot,
        crate::handlers::attendance::overtime::get_attendance_summaries,
        crate::handlers::attendance::overtime::create_behalf_assignment,
        crate::handlers::attendance::overtime::delete_behalf_assignment,
        crate::handlers::attendance::overtime::list_behalf_assignments,

        // Attendance Logs Paths
        crate::handlers::attendance::logs::force_reprocess_attendance,
        crate::handlers::attendance::logs::force_reprocess_all_attendance,
        crate::handlers::attendance::staging::get_staging_logs,
        crate::handlers::attendance::staging::resolve_staging_log,
        crate::handlers::attendance::staging::dismiss_staging_log,
        crate::handlers::attendance::paper_forms::enter_paper_ot,
        crate::handlers::attendance::paper_forms::bulk_upload_paper_ot,
        crate::handlers::attendance::time_tracking::update_time_entry,
        crate::handlers::attendance::time_tracking::delete_time_entry,
        crate::handlers::attendance::reports::get_attendance_report,

        // Attendance Policies Paths
        crate::handlers::attendance::policies::get_default_attendance_rules,
        crate::handlers::attendance::policies::list_default_attendance_policy_versions,
        crate::handlers::attendance::policies::update_default_attendance_rules,

        // Attendance Policy Assignments Paths
        crate::handlers::attendance::policy_assignments::list_policy_assignments,
        crate::handlers::attendance::policy_assignments::upsert_policy_assignment,
        crate::handlers::attendance::policy_assignments::delete_policy_assignment,

        // Attendance Imports Paths
        crate::handlers::attendance::import::upload_attendance_excel,
        crate::handlers::attendance::import::upload_attendance_logs,
        crate::handlers::attendance::import::upload_employee_excel,
        crate::handlers::attendance::import::get_latest_import_job,
        crate::handlers::attendance::import::get_import_job_status,
        crate::handlers::attendance::import::download_import_error_file,

        // Jobs Paths
        crate::handlers::jobs::get_generic_job_status,

        // Custom Fields Paths
        crate::handlers::custom_fields::create_definition,
        crate::handlers::custom_fields::get_definition,
        crate::handlers::custom_fields::list_definitions,
        crate::handlers::custom_fields::update_definition,
        crate::handlers::custom_fields::delete_definition,
        crate::handlers::custom_fields::get_values,
        crate::handlers::custom_fields::set_values,

        // Lifecycle Checklists
        crate::handlers::lifecycle_checklists::get_my_tasks,
        crate::handlers::lifecycle_checklists::list_templates,
        crate::handlers::lifecycle_checklists::create_template,
        crate::handlers::lifecycle_checklists::get_template,
        crate::handlers::lifecycle_checklists::update_template,
        crate::handlers::lifecycle_checklists::delete_template,
        crate::handlers::lifecycle_checklists::list_template_items,
        crate::handlers::lifecycle_checklists::create_template_item,
        crate::handlers::lifecycle_checklists::get_template_item,
        crate::handlers::lifecycle_checklists::update_template_item,
        crate::handlers::lifecycle_checklists::delete_template_item,
        crate::handlers::lifecycle_checklists::list_checklists,
        crate::handlers::lifecycle_checklists::create_checklist,
        crate::handlers::lifecycle_checklists::get_checklist,
        crate::handlers::lifecycle_checklists::update_item_status,
    ),
    components(
        schemas(
            Device,
            CreateDeviceRequest,
            UpdateDeviceRequest,
            AuthorizedAgent,
            TimeEntry,
            PunchInRequest,
            PunchOutRequest,
            AgentStatus,
            DeviceStatus,
            GenericResponse,
            WorkspaceStats,
            EarlyArrivalStrategy,
            LeaveStatus,
            AdjustmentStatus,
            LogSource,
            ImportJobStatus,
            LeaveTransactionType,
            WorkspaceConfigEntry,
            ResolvedConfig,
            ConfigValue,
            crate::handlers::workspace_config::UpsertConfigRequest,

            // Notifications Schemas
            Notification,
            NotificationCursorRequest,
            NotificationResponse,
            UnreadCountResponse,

            // Workspace Members Schemas
            crate::handlers::workspaces::members::UpdateMemberRoleRequest,
            AddMemberRequest,
            WorkspaceMemberDetail,
            WorkspaceMemberDetailResponse,

            // Workspace Permissions Schemas
            MemberExtraPermission,
            AssignPermissionRequest,

            // Workspace Roles Schemas
            crate::handlers::workspaces::roles::CreateWorkspaceRoleRequest,
            crate::handlers::workspaces::roles::UpdateWorkspaceRoleRequest,
            crate::handlers::workspaces::roles::UpdateRolePermissionsRequest,
            crate::handlers::workspaces::roles::PaginatedWorkspaceRoles,
            WorkspaceRole,
            AssignRoleRequest,
            MemberExtraRole,

            // Departments Schemas
            Department,
            CreateDepartmentRequest,
            UpdateDepartmentRequest,
            crate::handlers::workspaces::departments::PaginatedDepartments,

            // Feature Flags Schemas
            crate::models::feature_flag::FeatureFlag,
            crate::models::feature_flag::CreateFeatureFlagRequest,
            crate::models::feature_flag::UpdateFeatureFlagRequest,
            crate::models::feature_flag::FeatureFlagCheckResponse,

            // Dashboard Preferences Schemas
            crate::handlers::dashboard_preferences::DashboardPreferences,
            crate::handlers::dashboard_preferences::UpdateDashboardPreferences,

            // Employment Phases Schemas
            crate::models::employment_phase::EmploymentPhaseDefinition,
            crate::models::employment_phase::EmployeePhase,
            crate::handlers::employment_phases::CreateEmploymentPhaseDefinitionRequest,
            crate::handlers::employment_phases::AssignEmployeePhaseRequest,
            crate::handlers::employment_phases::UpdateEmployeePhaseStatusRequest,

            // Recruitment Sources Schemas
            crate::models::RecruitmentSource,
            crate::handlers::recruitment_sources::PaginatedRecruitmentSources,
            crate::models::CreateRecruitmentSourceRequest,
            crate::models::UpdateRecruitmentSourceRequest,
            crate::models::RecruitmentSourceBreakdown,
            crate::models::TurnoverTenureBucket,
            crate::models::TurnoverBySource,
            crate::models::TurnoverByDepartment,
            crate::models::TurnoverAnalytics,

            // Assignment Rules Schemas
            crate::models::AssignmentRule,
            crate::models::CreateAssignmentRuleRequest,
            crate::models::UpdateAssignmentRuleRequest,
            crate::models::AssignmentRuleConditions,

            // Attendance Schemas
            crate::models::AttendanceAdjustmentRequest,
            crate::models::CreateAttendanceAdjustmentRequest,
            crate::models::ApproveAttendanceAdjustmentRequest,
            crate::handlers::attendance::overtime::ClaimOtRequest,
            crate::handlers::attendance::overtime::ApproveOtRequest,
            crate::models::AttendanceDailySummary,
            crate::models::AttendanceRules,
            crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
            crate::handlers::attendance::policies::AttendancePolicyVersionResponse,
            crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse,
            crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
            crate::models::AttendanceImportJob,
            crate::models::enums::ImportTargetType,
            crate::models::enums::ImportJobStatus,
            crate::handlers::attendance::paper_forms::EnterPaperOtRequest,
            crate::models::BulkOtUploadResult,
            crate::models::BulkOtEntryError,
            crate::handlers::attendance::staging::StagingLogsResponse,
            crate::handlers::attendance::staging::StagingLogResponse,
            crate::handlers::attendance::staging::ResolveStagingLogRequest,
            crate::models::UpdateTimeEntryRequest,

            // Auth Schemas
            RegisterRequest,
            LoginRequest,
            RefreshRequest,
            ChangePasswordRequest,
            UpdateUserRequest,
            UserResponse,
            AuthResponse,

            // ESS Schemas
            MyProfileResponse,
            UpdateEmployeeProfileRequest,
            EmployeeProfile,
            EmployeeEmergencyContact,
            CreateEmergencyContactRequest,
            EmployeeDocument,
            UploadDocumentRequest,
            crate::handlers::ess::EssPunchRequest,
            crate::handlers::ess::EssLeaveBalance,

            // Workspace / Dashboard Schemas
            AttendanceTrendPoint,
            DepartmentAttendancePoint,
            UnexcusedAbsencesPoint,
            OvertimeTrendPoint,
            PeakTimeSlot,
            LeaveTypeBreakdown,
            LatenessByDay,
            Workspace,
            CreateWorkspaceRequest,
            UpdateWorkspaceRequest,
            WorkLocation,
            CreateWorkLocationRequest,
            UpdateWorkLocationRequest,
            crate::handlers::workspaces::locations::PaginatedLocations,
            Employer,
            CreateEmployerRequest,
            UpdateEmployerRequest,
            crate::handlers::workspaces::employers::PaginatedEmployers,

            // Phase 3: Staff Management & Rosters
            WorkingCondition,
            crate::handlers::employees::PaginatedWorkingConditions,
            CreateWorkingConditionRequest,
            crate::handlers::employees::PaginatedEmployees,
            Employee,
            CreateEmployeeRequest,
            UpdateEmployeeRequest,
            RehireEmployeeRequest,
            EmployeeDepartmentHistory,
            TransferEmployeeRequest,
            Department,
            CreateDepartmentRequest,
            UpdateDepartmentRequest,
            EmployeeContract,
            CreateContractRequest,
            UpdateContractRequest,
            Employment,
            CreateEmploymentRequest,
            UpdateEmploymentRequest,
            EmployeeAssignment,
            CreateEmployeeAssignmentRequest,
            UpdateEmployeeAssignmentRequest,
            Shift,
            CreateShiftRequest,
            UpdateShiftRequest,
            AssignShiftRequest,
            EmployeeSchedule,
            EffectiveSchedule,
            AssignmentHistory,
            ShiftRoster,
            CreateRosterRequest,
            BulkRosterRequest,
            RosterItem,
            WorkingPattern,
            WorkingPatternDetail,
            EmployeeWorkingPattern,
            WorkingPatternDetailRequest,
            CreateWorkingPatternRequest,
            AssignPatternRequest,
            BulkAssignPatternRequest,
            BulkAssignResponse,
            crate::models::shift_swap::ShiftSwapRequest,
            crate::models::shift_swap::ShiftSwapRequestWithNames,
            crate::models::shift_swap::CreateShiftSwapRequest,
            crate::models::shift_swap::ApproveShiftSwapRequest,
            crate::models::shift_swap::ShiftSwapStatus,
            crate::repositories::shift_swap_repository::SwapStats,
            crate::models::attendance::ScheduleTrace,

            // Phase 4: Leave Requests & Overtime Claims Schemas
            LeaveRequest,
            SubmitLeaveRequest,
            ApproveLeaveRequest,
            EmployeeLeaveBalance,
            WorkspaceLeaveBalance,
            LeaveHistoryItem,
            AdjustBalanceRequest,
            CreateLeavePolicyRequest,
            UpdateLeavePolicyRequest,
            LeavePolicyDefinition,
            AssignLeavePolicyRequest,
            LeavePolicyAssignment,
            crate::handlers::leaves::PaginatedWorkspaceLeaveBalances,
            crate::handlers::leaves::TriggerAccrualRequest,
            crate::handlers::leaves::AccrualJobResponse,
            crate::handlers::leaves::JobStatusResponse,
            LeavePolicyAssignmentDetail,
            LeaveType,
            LeaveTypeConfig,
            crate::handlers::leave_types::CreateLeaveTypeRequest,
            crate::handlers::leave_types::UpdateLeaveTypeConfigRequest,
            Holiday,
            CreateHolidayRequest,
            UpdateHolidayRequest,
            crate::repositories::holiday_repository::HolidayLeaveResult,
            OvertimeRequest,
            OvertimeRequestEmployee,
            crate::handlers::overtime_request::CreateOvertimeRequestPayload,
            crate::handlers::overtime_request::OvertimeRequestDetail,
            crate::handlers::overtime_request::EmployeeConsentRequest,
            OvertimePolicy,
            OvertimeRule,
            OvertimePolicyAssignment,
            crate::handlers::overtime::CreatePolicyRequest,
            crate::handlers::overtime::CreateRuleRequest,
            crate::handlers::overtime::AssignPolicyRequest,
            BehalfAssignment,
            CreateBehalfAssignmentRequest,

            // Phase 5: Compensation, Workflows & Webhooks Schemas
            Payslip,
            PayslipSegment,
            BatchCalculatePayrollRequest,
            BatchCalculatePayrollResponse,
            BatchCalculateResult,
            crate::handlers::payroll::payslips::PayslipSegmentResponse,
            crate::handlers::payroll::payslips::FullPayslipResponse,
            PayrollSettings,
            UpdatePayrollSettingsRequest,
            SalaryComponentDefinition,
            CreateSalaryComponentRequest,
            AssignSalaryComponentRequest,
            EmployeeSalaryComponent,
            PayrollIncentivePolicyResponse,
            PayrollIncentivePolicy,
            IncentiveTier,
            IncentiveEvaluationResult,
            PayslipRuleEvaluation,
            CreateIncentiveTierRequest,
            CreatePayrollIncentivePolicyRequest,
            SetDepartmentPayrollConfigRequest,
            SetEmployeePayrollConfigRequest,
            DepartmentPayrollConfig,
            EmployeePayrollConfig,
            SalaryComponentWithDefinition,
            CompensationEmploymentContext,
            CompensationChangeDetails,
            CompensationTimelineEvent,
            CompensationHistoryResponse,
            crate::handlers::salary_bases::SalaryBasesResponse,
            crate::handlers::salary_bases::CreateBaseRequest,
            crate::handlers::salary_bases::AssignMemberRequest,
            ApprovalWorkflow,
            crate::handlers::approval_workflow::PaginatedApprovalWorkflows,
            CreateWorkflowRequest,
            crate::handlers::approval_workflow::ProcessApprovalRequest,
            ApprovalActionLog,
            ApprovalWorkflowTask,
            crate::handlers::report_export::ExportReportRequest,
            crate::handlers::report_export::ExportJobResponse,
            crate::handlers::report_export::JobStatusResponse,
            crate::handlers::report_export::JobProgress,

            // Payroll Rules Schemas
            PayrollRuleDefinition,
            PayrollRuleTier,
            PayrollRuleAssignment,
            crate::handlers::payroll_rules::CreatePayrollRuleDefinitionRequest,
            crate::handlers::payroll_rules::CreatePayrollRuleTierRequest,
            crate::handlers::payroll_rules::AssignPayrollRuleRequest,

            // Phase 6: Platform Admin & Infrastructure Schemas
            SystemDbCluster,
            crate::handlers::system_db_clusters::CreateDbClusterRequest,
            crate::handlers::system_db_clusters::UpdateDbClusterStatusRequest,
            crate::handlers::system_admin::SystemRoleAssignment,
            crate::handlers::system_admin::SystemRoleInfo,
            crate::handlers::system_admin::AssignSystemRoleRequest,
            crate::handlers::worker_monitor::WorkerStatus,
            crate::handlers::worker_monitor::WorkerHeartbeat,
            crate::handlers::worker_monitor::MonitorOverviewResponse,
            crate::handlers::worker_monitor::MonitorJobsResponse,
            crate::handlers::worker_monitor::JobTypeStats,
            crate::queue::JobMetadata,
            crate::queue::JobStatus,
            crate::queue::ReportType,
            crate::queue::DateRange,
            crate::handlers::workspaces::audit::PaginatedAuditLogs,
            crate::domain::audit::AuditLog,

            // Agent Request Schemas
            CreateAuthorizedAgentRequest,
            TrustAgentRequest,
            UpdateAuthorizedAgentRequest,

            // Custom Fields Schemas
            CustomFieldDefinition,
            CreateCustomFieldDefinitionRequest,
            UpdateCustomFieldDefinitionRequest,
            SetCustomFieldValueRequest,
            BulkSetCustomFieldValuesRequest,
            CustomFieldValueResponse,

            // Lifecycle Checklists
            CreateChecklistRequest,
            LifecycleChecklistResponse,
            PendingTaskResponse,
            LifecycleChecklistTemplate,
            LifecycleChecklistTemplateItem,
            LifecycleChecklistType,
            ChecklistCategory,
            ChecklistAssigneeType,
            ChecklistStatus,
            ChecklistItemStatus,
            CreateLifecycleChecklistTemplateRequest,
            UpdateLifecycleChecklistTemplateRequest,
            CreateLifecycleChecklistTemplateItemRequest,
            UpdateLifecycleChecklistTemplateItemRequest,
            UpdateChecklistItemStatusRequest,
            LifecycleChecklistItem,
            LifecycleChecklist,
        )
    ),
    tags(
        (name = "leeattend", description = "LeeAttend API")
    )
)]
pub struct ApiDoc;

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use utoipa::OpenApi;

    #[test]
    fn test_openapi_generation() {
        let openapi = ApiDoc::openapi();
        let json = openapi
            .to_json()
            .expect("Failed to serialize OpenAPI to JSON");

        // Basic sanity checks
        assert!(json.contains("\"openapi\":") || json.contains("\"openapi\" :"));
        assert!(json.contains("3.1.0"));
        assert!(json.contains("\"title\":") && json.contains("apiserver"));

        // Verify paths are present
        assert!(json.contains("/agents"));
        assert!(json.contains("/devices"));

        // Verify core schemas are present
        assert!(json.contains("Device"));
        assert!(json.contains("AgentStatus"));
        assert!(json.contains("AuthorizedAgent"));
    }

    #[test]
    fn test_all_handlers_documented() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

        let openapi_file = format!("{manifest_dir}/src/openapi.rs");
        let openapi_content =
            std::fs::read_to_string(&openapi_file).expect("Failed to read openapi.rs");

        let mut registered_handlers = std::collections::HashSet::new();
        for line in openapi_content.lines() {
            if let Some(start_idx) = line.find("crate::handlers::") {
                let start = start_idx + "crate::handlers::".len();
                let mut end = start;
                while end < line.len()
                    && (line.as_bytes()[end].is_ascii_alphanumeric()
                        || line.as_bytes()[end] == b'_'
                        || line.as_bytes()[end] == b':')
                {
                    end += 1;
                }
                registered_handlers.insert(line[start..end].to_string());
            }
        }

        let routes_dir = format!("{manifest_dir}/src/routes");
        let mut used_handlers = std::collections::HashSet::new();

        fn visit_dirs(dir: &std::path::Path, used: &mut std::collections::HashSet<String>) {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dirs(&path, used);
                    } else if path.extension().is_some_and(|ext| ext == "rs") {
                        let content = std::fs::read_to_string(&path).unwrap();
                        let methods = ["get(", "post(", "put(", "delete(", "patch("];
                        for method in methods {
                            let mut start_idx = 0;
                            while let Some(idx) = content[start_idx..].find(method) {
                                let absolute_idx = start_idx + idx;
                                let start = absolute_idx + method.len();
                                let mut end = start;
                                while end < content.len()
                                    && (content.as_bytes()[end].is_ascii_alphanumeric()
                                        || content.as_bytes()[end] == b'_'
                                        || content.as_bytes()[end] == b':')
                                {
                                    end += 1;
                                }
                                let handler = &content[start..end];
                                if !handler.is_empty() && !handler.contains("axum") {
                                    let clean_handler = if handler.starts_with("crate::handlers::")
                                    {
                                        &handler["crate::handlers::".len()..]
                                    } else {
                                        handler
                                    };
                                    used.insert(clean_handler.to_string());
                                }
                                start_idx = end;
                            }
                        }
                    }
                }
            }
        }

        if std::path::Path::new(&routes_dir).exists() {
            visit_dirs(std::path::Path::new(&routes_dir), &mut used_handlers);
        }

        let mut missing_handlers = Vec::new();
        let whitelist = [
            // Internal/WS handlers that do not need openapi
            "ws_handler",
            "report_stream_ws",
            "monitor_stream_sse",
            "health_check",
            "readiness_check",
        ];

        for h in used_handlers {
            if h == "get"
                || h == "post"
                || h == "put"
                || h == "delete"
                || h == "patch"
                || h == "any"
            {
                continue;
            }
            if whitelist.iter().any(|&w| h.ends_with(w)) {
                continue;
            }

            let h_last = h.split("::").last().unwrap_or(&h);
            let is_registered = registered_handlers.iter().any(|reg| {
                let reg_last = reg.split("::").last().unwrap_or(reg);
                reg_last == h_last
            });

            if !is_registered {
                missing_handlers.push(h);
            }
        }

        if !missing_handlers.is_empty() {
            missing_handlers.sort();
            panic!(
                "The following handlers are used in routes but missing from openapi.rs paths(...):\n{missing_handlers:#?}\n\n\
                 Please add them to `apiserver/src/openapi.rs` to ensure the API contract is complete."
            );
        }
    }

    #[test]
    fn test_openapi_paths_match_axum_routes() {
        let openapi = ApiDoc::openapi();
        let json_str = openapi
            .to_json()
            .expect("Failed to serialize OpenAPI to JSON");
        let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let mut openapi_normalized_paths = std::collections::HashSet::new();
        if let Some(paths) = json_val["paths"].as_object() {
            for path in paths.keys() {
                let mut normalized = String::new();
                let mut in_param = false;
                for c in path.chars() {
                    if c == '{' {
                        in_param = true;
                        normalized.push('{');
                    } else if c == '}' {
                        in_param = false;
                        normalized.push('}');
                    } else if !in_param {
                        normalized.push(c);
                    }
                }
                openapi_normalized_paths.insert(normalized);
            }
        }

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let routes_dir = format!("{manifest_dir}/src/routes");

        let mut axum_normalized_paths = std::collections::HashSet::new();

        fn visit_dirs_for_routes(
            dir: &std::path::Path,
            used: &mut std::collections::HashSet<String>,
        ) {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dirs_for_routes(&path, used);
                    } else if path.extension().is_some_and(|ext| ext == "rs") {
                        let content = std::fs::read_to_string(&path).unwrap();
                        let mut start_idx = 0;
                        while let Some(idx) = content[start_idx..].find(".route(\"") {
                            let absolute_idx = start_idx + idx;
                            let start = absolute_idx + ".route(\"".len();
                            if let Some(end_offset) = content[start..].find("\",") {
                                let end = start + end_offset;
                                let axum_route = &content[start..end];

                                let parts: Vec<&str> = axum_route.split('/').collect();
                                let mut normalized = String::new();
                                for (i, part) in parts.iter().enumerate() {
                                    if i > 0 {
                                        normalized.push('/');
                                    }
                                    if part.starts_with(':') || part.starts_with('*') {
                                        normalized.push_str("{}");
                                    } else {
                                        normalized.push_str(part);
                                    }
                                }

                                used.insert(normalized);
                            }
                            start_idx = start;
                        }
                    }
                }
            }
        }

        if std::path::Path::new(&routes_dir).exists() {
            visit_dirs_for_routes(
                std::path::Path::new(&routes_dir),
                &mut axum_normalized_paths,
            );
        }

        let whitelist = [
            "/{}/ws",
            "/ws",
            "/debug/users",
            "/debug/workspaces/{}/clear",
            "/dev/workspaces/{}/clear",
            "/health",
            "/health/ready",
        ];

        let mut missing_in_openapi = Vec::new();
        for axum_path in axum_normalized_paths {
            if whitelist.iter().any(|&w| axum_path.ends_with(w)) {
                continue;
            }

            // Axum routes are typically nested under /api/v1 (some under /dev, /debug)
            // Utoipa paths usually start with the sub-path e.g. /workspaces/{}
            // We check if the axum_path matches an openapi_path directly, or if the openapi_path is a prefix/suffix.
            // A simple reliable way: check if ANY openapi path ends with the axum path,
            // or if the axum path ends with the openapi path.
            let has_match = openapi_normalized_paths
                .iter()
                .any(|op| op == &axum_path || op.ends_with(&axum_path) || axum_path.ends_with(op));

            if !has_match {
                missing_in_openapi.push(axum_path);
            }
        }

        if !missing_in_openapi.is_empty() {
            missing_in_openapi.sort();
            panic!(
                "The following Axum routes are defined but missing (or path structure mismatched) in OpenAPI:\n{missing_in_openapi:#?}"
            );
        }
    }
}
