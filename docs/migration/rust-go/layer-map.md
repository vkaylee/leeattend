# Architecture Layer Map & Dependency Specification: Rust to Go Migration

**Document Status:** Ground-Truth Evidence Base  
**Date:** 2026-10-19  
**Source Baseline:** `apiserver/src/` (Rust 2021 / Axum / Diesel-Async)  
**Target Architecture:** Clean Architecture in Enterprise Go (`net/http.ServeMux` + `oapi-codegen` + `pgx/v5` / `sqlc`)  

---

## 1. Architectural Layers & Edge Invariants

### 1.1 Layer Definition Table

| Layer Name | Architectural Role & Scope | Rust Subsystems / Directories | Permitted Outgoing Edges | Prohibited Outgoing Edges |
|---|---|---|---|---|
| **L1: Presentation / Transport** | HTTP routing, request parsing, parameter extraction, response serialization, authentication extraction (`MemberContext`). | `apiserver/src/handlers/`<br>`apiserver/src/mw/`<br>`apiserver/src/routes/`<br>`apiserver/src/openapi.rs` | • L2 (Application Services)<br>• L6 (Domain / API DTOs) | • **L3 (Repositories)** *(Forbidden bypass)*<br>• **L4 (Persistence / Diesel)**<br>• **L5 (Pool Management)**<br>• **L7 (Direct SQL execution)** |
| **L2: Application / Services** | Use-case orchestration, business workflow coordination, transaction boundary demarcation, cross-domain coordination. | `apiserver/src/services/`<br>`apiserver/src/state.rs` | • L3 (Repository Traits)<br>• L6 (Domain Models & Rules)<br>• L2 (Inter-service orchestration)<br>• L5 (Transaction coordinator) | • **L4 (Concrete Diesel Repos)**<br>• **L7 (Raw SQL / Schema)**<br>• **L1 (HTTP Transport / Handlers)** |
| **L3: Repository Abstractions** | Abstract persistence contracts defining queries and commands without storage engine coupling. | `apiserver/src/repositories/*.rs` (Traits) | • L6 (Domain Entities / Value Objects)<br>• L5 (`TenantAuthToken`) | • **L1 (HTTP Handlers)**<br>• **L2 (Services)**<br>• **L4 (Concrete Engine Impls)** |
| **L4: Persistence Implementation** | Concrete SQL queries, Diesel schema mappings, database record hydration, connection execution. | `apiserver/src/repositories/diesel_*`<br>`apiserver/src/schema.rs`<br>`apiserver/src/models/diesel/` | • L3 (Repository Traits implemented)<br>• L5 (Connection Pool & Tx wrapper)<br>• L6 (Entity mapping) | • **L1 (HTTP Handlers)**<br>• **L2 (Services)**<br>• Direct business rule calculation |
| **L5: Connection & Transaction Infra** | Multi-tenant database connection management, dynamic pool routing, shard isolation, and transaction scoping. | `apiserver/src/diesel_pool/`<br>`apiserver/src/workspace_id_guard/` | • External database drivers (`deadpool`, `diesel-async`, `tokio-postgres`) | • **L1 (HTTP Handlers)**<br>• **L6 (Pure Domain calculations)** |
| **L6: Pure Domain Logic & Rules** | Pure domain calculations (attendance pairing, matcher, payroll formulas, OT rules, permissions, audit). Zero I/O. | `apiserver/src/domain/`<br>`apiserver/src/services/pairing/` (pure rules)<br>`apiserver/src/services/attendance_calculator.rs`<br>`apiserver/src/services/financial_calculator.rs`<br>`apiserver/src/services/payroll_evaluator.rs`<br>`apiserver/src/services/payroll_rule_evaluator.rs`<br>`apiserver/src/services/rule_evaluator.rs`<br>`apiserver/src/models/` | • Standard library types<br>• Decimal numeric library (`rust_decimal` / `shopspring/decimal`) | • **L1 (HTTP)**<br>• **L3 (Repositories)**<br>• **L4 (Diesel / SQL)**<br>• **L5 (DB Pools / Network)**<br>• Wall-clock system time |
| **L7: Operational Diagnostics** | Read-only aggregate analytics, blast-radius queries, administrative audit tools. | `apiserver/src/diagnostics/` | • L5 (`DieselPoolsTenantTxExt`, read-only tx)<br>• L6 (Domain parameters) | • Mutation operations<br>• L1 (Direct HTTP exposure without auth) |

### 1.2 Edge Invariant Rules

1. **Strict Dependency Rule (Downward Only):** Handlers (L1) must depend exclusively on Services (L2). Services (L2) coordinate workflows and call Repository interfaces (L3) and pure Domain components (L6). Repositories (L4) implement interfaces (L3) using connection infrastructure (L5).
2. **Prohibition of Service Bypass:** Handlers MUST NOT bypass the Service layer to execute repository calls or database queries directly.
3. **Purity of Domain Layer:** Domain calculations (pairing, overtime evaluation, payroll gross-to-net) MUST remain 100% pure in-process functions with zero I/O, zero database connection references, and zero floating-point arithmetic.
4. **Transaction Ownership Invariant:** Handlers NEVER acquire connections, begin transactions, commit, or rollback. Transactions are owned by the Service layer (orchestration) or Repository layer (atomic single-aggregate updates).

---

## 2. Concrete Service Types Inventory

The table below documents every concrete service and functional orchestration type declared in `apiserver/src/services/` (total: 60 types). Data-only structs (DTOs, argument structs, return containers) are excluded.

| Service / Component Name | Source File & Line | Wiring Location (`AppState`) | Architectural Role & Responsibilities |
|---|---|---|---|
| `AgentLogBuffer` | `apiserver/src/services/agent_log_buffer.rs:87` | Background worker queue | In-memory MPSC punch log batching and background flushing buffer |
| `AgentService` | `apiserver/src/services/agent_service.rs:7` | `state.agents().service()` | Biometric device agent lifecycle, heartbeat tracking, and IP updates |
| `ApprovalWorkflowService` | `apiserver/src/services/approval_workflow/mod.rs:29` | `state.core().approval_workflow_service()` | Multi-step approval state machine, step transitions, and task resolution |
| `AttendanceAdjustmentService` | `apiserver/src/services/attendance_adjustment_service.rs:19` | `state.attendance().adjustment_service()` | Time punch adjustment requests, approval callbacks, and punch re-pairing |
| `AttendanceCalculator` | `apiserver/src/services/attendance_calculator.rs:113` | Domain pure calculation | Fixed minute-bucket calculation (standard, OT1, OT2, night) from punches |
| `EmployeeImportStrategy` | `apiserver/src/services/attendance_import_service.rs:61` | `AttendanceImportService` helper | Pipeline strategy for parsing and importing employee attendance batches |
| `AttendanceImportService` | `apiserver/src/services/attendance_import_service.rs:984` | `state.attendance().import_service()` | Bulk punch import job coordination, validation, and staging resolution |
| `AttendanceLogService` | `apiserver/src/services/attendance_log_service.rs:23` | `state.attendance().log_service()` | Raw punch ingestion, employment resolution, and pairing notification |
| `AttendanceMatcher` | `apiserver/src/services/attendance_matcher/mod.rs:46` | `PayrollService` helper | Matches punches and daily logs against shift rules and payroll segments |
| `AttendanceService` | `apiserver/src/services/attendance_service.rs:272` | `state.attendance().service()` | Attendance daily unit calculation and schedule resolution |
| `AttendanceSummaryService` | `apiserver/src/services/attendance_summary_service.rs:248` | `state.attendance().summary_service()` | Daily attendance aggregation, paper OT entry, and workflow callbacks |
| `AuditHelper` | `apiserver/src/services/audit_service.rs:20` | `state.core().audit_repo()` helper | Centralized audit log generation and persistence helper |
| `AuthService` | `apiserver/src/services/auth_service.rs:25` | `state.core().auth_service()` | User password authentication, Argon2id verify/rehash, and JWT issuance |
| `ComplianceValidationService` | `apiserver/src/services/compliance_validation_service.rs:26` | `state.shifts().compliance()` | Roster compliance validation (rest intervals, max weekly OT limits) |
| `ConditionTypeService` | `apiserver/src/services/condition_type_service.rs:15` | `state.core().condition_type_service()` | Working condition catalog management and workspace overrides |
| `CustomFieldService` | `apiserver/src/services/custom_field_service.rs:19` | `state.core().custom_field_service()` | Custom field definition and entity value management with audit logging |
| `EContractDispatcher` | `apiserver/src/services/econtract/dispatcher.rs:9` | Contract signing subsystem | Dispatches digital employment contracts to configured external signers |
| `EmailService` | `apiserver/src/services/email_service.rs:181` | Background notification | Transactional email dispatch, rate limiting, and SMTP transport |
| `EmployeeService` | `apiserver/src/services/employee_service/mod.rs:13` | `state.employees().service()` | Employee lifecycle (create, update, rehire, profile, compensation) |
| `EmployerService` | `apiserver/src/services/employer_service.rs:14` | `state.core().employer_service()` | Legal employer entity management and workspace assignment |
| `ExcelWriter` | `apiserver/src/services/excel_writer.rs:23` | Export service helper | Streaming XLSX binary generation for attendance and payroll reports |
| `FinancialCalculator` | `apiserver/src/services/financial_calculator.rs:75` | Domain pure calculation | Arbitrary-precision gross-to-net payroll, incentive, and deduction math |
| `HolidayLeaveService` | `apiserver/src/services/holiday_leave_service.rs:23` | `state.leaves().holiday_leave_service()` | Holiday calendar coordination and automatic holiday leave balance credits |
| `ImportPipeline` | `apiserver/src/services/import_pipeline.rs:68` | Worker import subsystem | Multi-step tabular data validation and progressive import orchestrator |
| `InfraJobService` | `apiserver/src/services/infra_job_service.rs:12` | Worker subsystem | Long-running asynchronous system job tracking and telemetry |
| `LeaveAccrualService` | `apiserver/src/services/leave_accrual_service/mod.rs:20` | `state.leaves().accrual_service()` | Monthly leave accrual calculation based on qualifying worked days |
| `LeaveImportService` | `apiserver/src/services/leave_import_service.rs:34` | Worker import subsystem | Batch leave allocation and historical balance import orchestrator |
| `LeaveImportStrategy` | `apiserver/src/services/leave_import_service.rs:38` | `LeaveImportService` helper | Row-level validation and ingestion strategy for leave data |
| `LeaveService` | `apiserver/src/services/leave_service/mod.rs:29` | `state.leaves().service()` | Leave request submission, balance verification, approval, and cancellations |
| `LifecycleService` | `apiserver/src/services/lifecycle_service.rs:21` | `state.core().lifecycle_service()` | Onboarding and offboarding checklist workflows and step assignments |
| `LocationService` | `apiserver/src/services/location_service.rs:13` | `state.core().location_service()` | Work location geofencing, IP whitelisting, and workspace assignment |
| `MonitorAlertService` | `apiserver/src/services/monitor_alert_service.rs:10` | Worker alert subsystem | System operational monitoring alerts and threshold breach dispatches |
| `ThresholdEvaluator` | `apiserver/src/services/monitor_threshold_evaluator.rs:317` | Worker alert subsystem | In-memory evaluation of sliding-window monitor threshold rules |
| `OutboxJobService` | `apiserver/src/services/outbox_job_service.rs:9` | Reliable messaging subsystem | Outbox table polling, payload dispatch, and retry backoff manager |
| `OvertimeRequestService` | `apiserver/src/services/overtime_request_service.rs:8` | `state.payroll().overtime_request_service()` | Pre-shift overtime pre-approvals, quota checks, and approval engine sync |
| `PairingService` | `apiserver/src/services/pairing/service.rs:19` | `state.attendance().pairing_service()` | Attendance punch pairing (single-day and date-range propagation) |
| `ShiftAwareStrategy` | `apiserver/src/services/pairing/strategy.rs:10` | `PairingService` helper | Roster-aware punch boundary alignment and overnight shift pairing |
| `PayrollEvaluator` | `apiserver/src/services/payroll_evaluator.rs:5` | Domain pure calculation | Segment-level incentive calculations and condition evaluation |
| `PayrollRuleEvaluator` | `apiserver/src/services/payroll_rule_evaluator.rs:7` | Domain pure calculation | Custom rule AST evaluator for salary components and allowances |
| `DirectRosterDataProvider` | `apiserver/src/services/payroll_service/context.rs:94` | `PayrollService` helper | Direct in-memory adapter supplying roster snapshot data to payroll runs |
| `PayrollService` | `apiserver/src/services/payroll_service/mod.rs:56` | `state.payroll().service()` | Workspace payroll calculation, batch chunking, and payslip generation |
| `PolicyResolverService` | `apiserver/src/services/policy_resolver_service.rs:9` | Domain rule resolution | Hierarchical policy resolution (Employee -> Department -> Workspace) |
| `RecruitmentService` | `apiserver/src/services/recruitment_service.rs:20` | `state.employees().recruitment_service()` | Candidate attribution and hiring source retention analytics |
| `ReportCoordinator` | `apiserver/src/services/report_coordinator.rs:102` | `state.reports().coordinator()` | Distributed asynchronous export report job lifecycle and status tracking |
| `ReportService` | `apiserver/src/services/report_service/mod.rs:334` | `state.reports().service()` | Interactive and paginated attendance reports (Daily, Summary, No-Show) |
| `RuleEvaluator` | `apiserver/src/services/rule_evaluator.rs:14` | Domain pure calculation | General condition matching rule evaluator for assignments |
| `ShiftService` | `apiserver/src/services/shift_service.rs:11` | `state.shifts().service()` | Shift definitions, intervals, and working pattern assignments |
| `ShiftSwapService` | `apiserver/src/services/shift_swap_service.rs:20` | `state.shifts().swap_service()` | Peer-to-peer shift swap requests, approvals, and roster updates |
| `SignatureService` | `apiserver/src/services/signature_service.rs:13` | Document signing subsystem | Digital signature capture, cryptographic verification, and storage |
| `FileService` | `apiserver/src/services/storage/file_service.rs:19` | `state.file_service()` | Encrypted file upload, quarantine, clamav virus scan, and S3 resolution |
| `StorageResolver` | `apiserver/src/services/storage/resolver.rs:11` | Storage subsystem | Resolves workspace-custom S3 vs system-shared object storage backend |
| `S3Storage` | `apiserver/src/services/storage/s3_storage.rs:20` | Storage subsystem | AWS S3 / MinIO multipart storage client |
| `ClamAvScanner` | `apiserver/src/services/storage/scanner/clamav.rs:6` | Storage subsystem | ClamAV TCP streaming daemon client for real-time malware scanning |
| `ScanProvider` | `apiserver/src/services/storage/scanner/mod.rs:20` | Storage subsystem | Antivirus scanning provider abstraction and mock fallback |
| `TemplateService` | `apiserver/src/services/template_service.rs:10` | Document template subsystem | Dynamic document template variable substitution and formatting |
| `TenantMigrator` | `apiserver/src/services/tenant_provisioner.rs:36` | Tenant provisioning | DDL migration runner executing embedded migrations on tenant shards |
| `TerminationService` | `apiserver/src/services/termination_service.rs:43` | Not exposed by `AppState` accessor in `state.rs`; wiring outside requested directories | Employee termination, final leave payout computation, and offboarding |
| `TimeTrackingService` | `apiserver/src/services/time_tracking_service.rs:18` | `state.attendance().time_tracking_service()` | Real-time web/mobile clock-in/out, GPS/geofence validation, and audit |
| `WorkspaceConfigService` | `apiserver/src/services/workspace_config_service.rs:9` | `state.core().workspace_config_service()` | Tenant feature flags, module toggle configuration, and parameters |
| `WorkspaceService` | `apiserver/src/services/workspace_service.rs:12` | `state.core().workspace_service()` | Workspace onboarding, tenant DB provisioning, membership, and settings |

---

## 3. Repository Traits and Diesel Implementations

The table below documents every repository trait (total: 65 traits), its declaration location, its primary Diesel implementation (total: 54 Diesel structs), and implementation status.

| Repository Trait Name | Trait Declaration | Concrete Implementation Struct | Implementation File & Line | Implementation Status & Notes |
|---|---|---|---|---|
| `AgentRepository` | `repositories/agent_repository.rs:42` | `DieselAgentRepository` | `repositories/diesel_agent_repository.rs:22` | Implemented (line 33). System + Tenant queries. |
| `ApprovalWorkflowRepository` | `repositories/approval_workflow_repository.rs:11` | `DieselApprovalWorkflowRepository` | `repositories/diesel_approval_workflow_repository.rs:18` | Implemented (line 29). Multi-step approval persistence. |
| `AssignmentRuleRepository` | `repositories/assignment_rule_repository.rs:12` | `DieselAssignmentRuleRepository` | `repositories/diesel_assignment_rule_repository.rs:16` | Implemented (line 27). Shift assignment rules. |
| `AttendanceAdjustmentRepository` | `repositories/attendance_adjustment_repository.rs:11` | `DieselAttendanceAdjustmentRepository` | `repositories/diesel_attendance_adjustment_repository.rs:16` | Implemented (line 27). Punch correction requests. |
| `AttendanceDailySummaryRepository` | `repositories/attendance_daily_summary_repository.rs:78` | `DieselAttendanceDailySummaryRepository` | `repositories/diesel_attendance_daily_summary_repository.rs:14` | Implemented (line 36). Aggregated attendance days. |
| `AttendanceLogErrorRepository` | `repositories/attendance_log_error_repository.rs:9` | `DieselAttendanceLogErrorRepository` | `repositories/diesel_attendance_log_error_repository.rs:10` | Implemented (line 21). Unmatched/errored punch logs. |
| `AttendancePolicyRepository` | `repositories/attendance_policy_repository.rs:9` | `DieselAttendancePolicyRepository` | `repositories/diesel_attendance_policy_repository.rs:74` | Implemented (line 86). Grace period, rounding rules. |
| `AuditLogRepository` | `repositories/audit_log_repository.rs:9` | `DieselAuditLogRepository` | `repositories/diesel_audit_log_repository.rs:16` | Implemented (line 27). Append-only tenant audit events. |
| `BehalfAssignmentRepository` | `repositories/behalf_assignment_repository.rs:7` | `DieselBehalfAssignmentRepository` | `repositories/diesel_behalf_assignment_repository.rs:15` | Implemented (line 26). Delegation permissions. |
| `CacheRepository` | `repositories/cache_repository.rs:11` | `RedisCache` | `apiserver/src/cache/redis_cache.rs:72` | Implemented. Non-Diesel Redis caching provider. |
| `CacheRepositoryExt` | `repositories/cache_repository.rs:42` | Blanket `impl<T: CacheRepository>` | `repositories/cache_repository.rs:48` | Implemented. Serde JSON convenience methods. |
| `ComplianceRepository` | `repositories/compliance_repository.rs:11` | `DieselComplianceRepository` | `repositories/diesel_compliance_repository.rs:13` | Implemented (line 24). Roster compliance constraints. |
| `ConditionTypeRepository` | `repositories/condition_type_repository.rs:8` | `DieselConditionTypeRepository` | `repositories/diesel_condition_type_repository.rs:13` | Implemented (line 99). Employment working conditions. |
| `CustomFieldRepository` | `repositories/custom_field_repository.rs:13` | `DieselCustomFieldRepository` | `repositories/diesel_custom_field_repository.rs:19` | Implemented (line 30). Dynamic custom attributes. |
| `DebugRepository` | `repositories/debug_repository.rs:15` | `DieselDebugRepository` | `repositories/diesel_debug_repository.rs:9` | Implemented (line 21). Development test data reset. |
| `DepartmentRepository` | `repositories/department_repository.rs:15` | `DieselDepartmentRepository` | `repositories/diesel_department_repository.rs:94` | Implemented (line 105). Hierarchy, managers, transfers. |
| `EmployeeAssignmentRepository` | `repositories/diesel_employee_repository/assignments.rs:15` | `DieselEmployeeRepository` | `repositories/diesel_employee_repository/assignments.rs:65` | Implemented. Sub-trait of `EmployeeRepository`. |
| `EmployeeContractRepository` | `repositories/employee_contract_repository.rs:12` | `DieselEmployeeContractRepository` | `repositories/diesel_employee_contract_repository.rs:16` | Implemented (line 27). Labor contract tracking. |
| `EmployeeCoreRepository` | `repositories/employee/mod.rs:29` | `DieselEmployeeRepository` | `repositories/diesel_employee_repository/mod.rs:51` | Implemented. Core CRUD; sub-trait of `EmployeeRepository`. |
| `EmployeePayrollConfigRepository` | `repositories/employee/mod.rs:242` | `DieselEmployeeRepository` | `repositories/diesel_employee_repository/payroll.rs:15` | Implemented. Sub-trait of `EmployeeRepository`. |
| `EmployeeRecruitmentRepository` | `repositories/employee/mod.rs:277` | `DieselEmployeeRepository` | `repositories/diesel_employee_repository/recruitment.rs:32` | Implemented. Sub-trait of `EmployeeRepository`. |
| `EmployeeRepository` | `repositories/employee/mod.rs:394` | `DieselEmployeeRepository` | `repositories/diesel_employee_repository/mod.rs:21` | Implemented via blanket `impl<T>` over 6 sub-traits. |
| `EmployeeSelfServiceRepository` | `repositories/employee/mod.rs:336` | `DieselEmployeeRepository` | `repositories/diesel_employee_repository/ess.rs:24` | Implemented. Sub-trait of `EmployeeRepository`. |
| `EmployeeWorkingConditionRepository` | `repositories/employee/mod.rs:199` | `DieselEmployeeRepository` | `repositories/diesel_employee_repository/working_condition.rs:16` | Implemented. Sub-trait of `EmployeeRepository`. |
| `EmployerRepository` | `repositories/employer_repository.rs:9` | `DieselEmployerRepository` | `repositories/diesel_employer_repository.rs:14` | Implemented (line 25). Legal employer entities. |
| `EmploymentPhaseRepository` | `repositories/employment_phase_repository.rs:10` | `DieselEmploymentPhaseRepository` | `repositories/diesel_employment_phase_repository.rs:14` | Implemented (line 25). Probation/official phase tracking. |
| `EmploymentRepository` | `repositories/employment_repository.rs:11` | `DieselEmploymentRepository` | `repositories/diesel_employment_repository.rs:15` | Implemented (line 26). Job titles, departments, dates. |
| `FeatureFlagRepository` | `repositories/feature_flag_repository.rs:12` | `DieselFeatureFlagRepository` | `repositories/diesel_feature_flag_repository.rs:14` | Implemented (line 25). Tenant feature toggles. |
| `FileRepository` | `repositories/file_repository.rs:9` | `DieselFileRepository` | `repositories/diesel_file_repository.rs:12` | Implemented (line 75). Encrypted file metadata. |
| `HolidayRepository` | `repositories/holiday_repository.rs:30` | `DieselHolidayRepository` | `repositories/diesel_holiday_repository.rs:17` | Implemented (line 29). Public and company holiday dates. |
| `ImportJobRepository` | `repositories/import_job_repository.rs:11` | `DieselImportJobRepository` | `repositories/diesel_import_job_repository.rs:15` | Implemented (line 26). Batch import progress & errors. |
| `InfraJobRepository` | `repositories/infra_job_repository.rs:69` | `DieselInfraJobRepository` | `repositories/diesel_infra_job_repository.rs:18` | Implemented (line 35). Asynchronous task records. |
| `LeaveAccrualLogRepository` | `repositories/leave_accrual_log_repository.rs:36` | `DieselLeaveAccrualLogRepository` | `repositories/diesel_leave_accrual_log_repository.rs:16` | Implemented (line 28). Monthly leave accrual audits. |
| `LeaveBalanceRepository` | `repositories/leave_balance_repository.rs:11` | `DieselLeaveBalanceRepository` | `repositories/diesel_leave_balance_repository.rs:16` | Implemented (line 28). Current employee leave quotas. |
| `LeavePolicyRepository` | `repositories/leave_policy_repository.rs:12` | `DieselLeavePolicyRepository` | `repositories/diesel_leave_policy_repository.rs:19` | Implemented (line 49). Annual leave entitlement rules. |
| `LeaveRequestRepository` | `repositories/leave_request_repository.rs:12` | `DieselLeaveRequestRepository` | `repositories/diesel_leave_request_repository/mod.rs:18` | Implemented (line 34). Leave requests & statuses. |
| `LifecycleChecklistRepository` | `repositories/lifecycle_checklist_repository.rs:15` | `DieselLifecycleChecklistRepository` | `repositories/diesel_lifecycle_checklist_repository.rs:28` | Implemented (line 39). Onboarding task checklists. |
| `LocationRepository` | `repositories/location_repository.rs:7` | `DieselLocationRepository` | `repositories/diesel_location_repository.rs:13` | Implemented (line 24). Geographic work locations. |
| `MonitorAlertRepository` | `repositories/monitor_alert_repository.rs:35` | `DieselMonitorAlertRepository` | `repositories/diesel_monitor_alert_repository.rs:15` | Implemented (line 39). System telemetry alert events. |
| `OutboxJobRepository` | `repositories/outbox_job_repository.rs:9` | `DieselOutboxJobRepository` | `repositories/diesel_outbox_job_repository.rs:15` | Implemented (line 26). Transactional outbox events. |
| `OvertimePolicyRepositoryTrait` | `repositories/overtime_policy_repository.rs:11` | `DieselOvertimePolicyRepository` | `repositories/diesel_overtime_policy_repository.rs:16` | Implemented (line 27). OT rates, multiplier tiers. |
| `OvertimeRequestRepository` | `repositories/overtime_request_repository.rs:7` | `DieselOvertimeRequestRepository` | `repositories/diesel_overtime_request_repository.rs:11` | Implemented (line 22). Pre-approved overtime requests. |
| `PayrollRepository` | `repositories/payroll_repository.rs:17` | `DieselPayrollRepository` | `repositories/diesel_payroll_repository/mod.rs:24` | Implemented (line 44). Payslips, components, payroll runs. |
| `PayrollRuleRepository` | `repositories/payroll_rule_repository.rs:13` | `DieselPayrollRuleRepository` | `repositories/diesel_payroll_rule_repository.rs:19` | Implemented (line 30). User-defined payroll formulas. |
| `PublisherRepository` | `repositories/publisher_repository.rs:7` | `RedisPublisherRepository` | `repositories/publisher_repository.rs:35` | Implemented (line 48). Non-Diesel Redis pub/sub. |
| `QueueRepository` | `repositories/queue_repository.rs:11` | `RedisQueueRepository` | `repositories/queue_repository.rs:75` | Implemented (line 90). Non-Diesel Redis task queue. |
| `RawAttendanceLogRepository` | `repositories/raw_attendance_log_repository.rs:12` | `DieselRawAttendanceLogRepository` | `repositories/diesel_raw_attendance_log_repository.rs:16` | Implemented (line 27). Immutable raw punch records. |
| `RosterRepository` | `repositories/roster/mod.rs:17` | `DieselRosterRepository` | `repositories/diesel_roster_repository/mod.rs:20` | Implemented (line 73). Shift schedules and rosters. |
| `SalaryBaseRepository` | `repositories/salary_base_repository.rs:7` | `DieselSalaryBaseRepository` | `repositories/diesel_salary_base_repository.rs:70` | Implemented (line 81). Statutory salary base tiers. |
| `SessionRepository` | `repositories/session_repository.rs:8` | `DieselSessionRepository` | `repositories/diesel_session_repository.rs:12` | Implemented (line 23). Global user JWT refresh sessions. |
| `ShiftIntervalRepository` | `repositories/shift_interval_repository.rs:8` | **None** *(Orphan Trait)* | **None** | **Unimplemented in codebase.** Replaced by `DieselRosterRepository` / `roster_data.shift_intervals`. |
| `ShiftRepository` | `repositories/shift_repository.rs:16` | `DieselShiftRepository` | `repositories/diesel_shift_repository.rs:19` | Implemented (line 48). Shift templates & times. |
| `ShiftSwapRepository` | `repositories/shift_swap_repository.rs:11` | `DieselShiftSwapRepository` | `repositories/diesel_shift_swap_repository.rs:118` | Implemented (line 129). Shift exchange requests. |
| `SignatureRepository` | `repositories/signature_repository.rs:9` | `DieselSignatureRepository` | `repositories/diesel_signature_repository.rs:14` | Implemented (line 25). Signed document audit trails. |
| `StagingLogRepository` | `repositories/staging_log_repository.rs:46` | `DieselStagingLogRepository` | `repositories/diesel_staging_log_repository.rs:16` | Implemented (line 28). Unmatched biometric punches. |
| `SystemDbClusterRepository` | `repositories/system_db_cluster_repository.rs:7` | `DieselSystemDbClusterRepository` | `repositories/diesel_system_db_cluster_repository.rs:10` | Implemented (line 21). Multi-tenant database cluster nodes. |
| `SystemRepository` | `repositories/system_repository.rs:6` | `DieselSystemRepository` | `repositories/diesel_system_repository.rs:8` | Implemented (line 19). System-level schema health. |
| `SystemSettingsRepository` | `repositories/system_settings_repository.rs:6` | `DieselSystemSettingsRepository` | `repositories/diesel_system_settings_repository.rs:10` | Implemented (line 21). Global instance settings. |
| `TimeEntryRepository` | `repositories/time_entry_repository.rs:14` | `DieselTimeEntryRepository` | `repositories/diesel_time_entry_repository.rs:40` | Implemented (line 51). Paired work intervals & breaks. |
| `UserRepository` | `repositories/user_repository.rs:12` | `DieselUserRepository` | `repositories/diesel_user_repository.rs:12` | Implemented (line 23). User accounts and auth credentials. |
| `WorkSegmentRepository` | `repositories/work_segment_repository.rs:11` | `DieselWorkSegmentRepository` | `repositories/diesel_work_segment_repository.rs:19` | Implemented (line 67). Multi-segment shifts and branches. |
| `WorkingPatternRepositoryTrait` | `repositories/working_pattern_repository.rs:21` | `DieselWorkingPatternRepository` | `repositories/diesel_working_pattern_repository.rs:17` | Implemented (line 39). Recurring shift pattern schedules. |
| `WorkspaceConfigRepository` | `repositories/workspace_config_repository.rs:9` | `DieselWorkspaceConfigRepository` | `repositories/diesel_workspace_config_repository.rs:12` | Implemented (line 23). Workspace-scoped configuration. |
| `WorkspaceRepository` | `repositories/workspace_repository.rs:25` | `DieselWorkspaceRepository` | `repositories/diesel_workspace_repository/mod.rs:15` | Implemented (line 33). Tenant workspaces, roles, members. |
| `WorkspaceStatsRepository` | `repositories/workspace_stats_repository.rs:96` | `DieselWorkspaceStatsRepository` | `repositories/workspace_stats_repository.rs:142` | Implemented (line 218). Executive dashboard aggregations. |

---

## 4. Handlers Making Direct Diesel / Repository Calls (Layer Violations)

An audit of all 77 handler files in `apiserver/src/handlers/` reveals widespread violation of the clean architecture boundary.

### 4.1 Handlers with Direct Persistence / Diesel / Pool Access (High-Severity Violations)

These handlers bypass the intended abstraction, executing raw SQL/Diesel, acquiring a pool, importing Diesel entities, or instantiating a concrete repository:

1. **`apiserver/src/handlers/common.rs`**
   - Line 525: `use diesel_async::SimpleAsyncConnection;`
   - Line 532: Directly acquires API connection: `state.infra().diesel().api.get().await`
   - Line 534: Executes raw SQL: `conn.batch_execute("SELECT 1").await`
2. **`apiserver/src/handlers/storage_config.rs`**
   - Lines 5-6: `use diesel::prelude::*;`, `use diesel_async::RunQueryDsl;`
   - Lines 97, 132, 264: Directly acquires API connection: `state.infra().diesel().api.get().await`
   - Line 174: Directly executes Diesel query: `diesel::insert_into(storage_configs::table)`
   - Line 268: Directly executes Diesel query: `diesel::delete(storage_configs::table.filter(...))`
3. **`apiserver/src/handlers/system_admin.rs`**
   - Lines 72, 181, 237, 318: Directly acquires connection: `state.infra().diesel().api.get().await`
   - Lines 76-88: Declares `RoleRow` and executes raw SQL: `diesel::sql_query("SELECT id, name, description FROM system_roles ...")`
   - Lines 99-122: Declares `AssigneeRow` and executes raw SQL: `diesel::sql_query("SELECT role_id, user_id ... FROM user_system_roles ...")`
   - Lines 185-205: Executes raw SQL: `diesel::sql_query("SELECT sr.name FROM user_system_roles usr ...")`
   - Lines 241-273: Executes raw SQL: `diesel::sql_query("SELECT id FROM system_roles ...")` and `diesel::sql_query("INSERT INTO user_system_roles ...")`
   - Lines 333-364: Executes raw SQL count & delete: `diesel::sql_query("DELETE FROM user_system_roles ...")`
4. **`apiserver/src/handlers/debug.rs`**
   - Lines 33, 67: Directly instantiates concrete Diesel repository: `DieselDebugRepository::new(state.infra().diesel().clone())`
5. **`apiserver/src/handlers/shifts.rs`**
   - Line 194: Passes raw Diesel pool to cache helper: `.get_roster_data(workspace_id, state.infra().diesel())`
   - Lines 199, 203: Imports and maps internal Diesel entity: `models::diesel::roster::ShiftDefaultIntervalEntity`
6. **`apiserver/src/handlers/worker_monitor.rs`**
   - Line 1265: Directly inspects connection pool metrics: `state.infra().diesel().api.status()`
   - Lines 424, 450: Imports raw internal Diesel entities: `models::diesel::infra_job::*`
7. **`apiserver/src/handlers/monitor_alerts.rs`**
   - Line 11: Imports raw internal Diesel entity: `models::diesel::monitor_alert::*`

### 4.2 Handlers Bypassing Service Layer to Call Repositories Directly

Out of 77 handler files, **55 files** bypass the Service layer to execute calls directly on repositories (28 repository-only, 27 hybrid).

#### Category A: Complete Service Bypass (28 files)
These handlers contain **zero service calls**, interacting solely with repositories:

| Handler File | Total Repo Calls | Exact Citation of Representative Repository Calls |
|---|---|---|
| `handlers/assignment_rules.rs` | 4 | lines 37, 66, 103, 138 (`.shifts().assignment_rule_repo().*`) |
| `handlers/attendance/logs.rs` | 11 | lines 117, 188, 293, 359, 451 (`.employees().repo()`), 132, 147 (`.time_entry_repo()`), 203, 216 (`.raw_log_repo()`), 311, 344 (`.employment_repo()`) |
| `handlers/attendance/policies.rs` | 3 | lines 61, 96, 147 (`.attendance().policy_repo().*`) |
| `handlers/attendance/policy_assignments.rs` | 3 | lines 57, 89, 121 (`.attendance().policy_repo().*`) |
| `handlers/common.rs` | 17 | lines 271 (`.user_repo()`), 279, 287, 302, 316, 334, 768 (`.workspace_repo()`), 346, 403, 776, 792 (`.employees().repo()`), 595 (`.department_repo()`), 641, 685, 708, 732, 752 (`.behalf_assignment_repo()`) |
| `handlers/dashboard_overtime_breakdown.rs` | 3 | lines 73 (`.workspace_repo()`), 137 (`.stats_repo()`), 161 (`.audit_repo()`) |
| `handlers/dashboard_preferences.rs` | 2 | lines 238, 281 (`.core().user_repo().*`) |
| `handlers/employee_assignments.rs` | 11 | lines 39, 85, 131, 220, 274 (`.employees().repo()`), 43, 89, 148, 224, 278 (`.employment_repo()`), 140 (`.workspace_repo()`) |
| `handlers/employee_contracts.rs` | 10 | lines 72, 156, 224 (`.employment_repo()`), 82, 118, 150, 218, 262, 293 (`.contract_repo()`), 172 (`.employment_phase_repo()`) |
| `handlers/employment_phases.rs` | 11 | lines 65, 98, 140, 191, 238, 271, 312, 348 (`.employment_phase_repo()`), 130, 181, 228 (`.employees().repo()`) |
| `handlers/feature_flags.rs` | 7 | lines 37, 76, 118, 156, 194, 242, 284 (`.feature_flags().feature_flag_repo().*`) |
| `handlers/leave_types.rs` | 5 | lines 50, 77, 107, 135, 167 (`.leaves().policy_repo().*`) |
| `handlers/notifications.rs` | 5 | lines 36, 49, 77, 104, 127 (`.core().user_repo().*`) |
| `handlers/overtime.rs` | 5 | lines 70, 102, 128, 169, 202 (`.payroll().overtime_policy_repo().*`) |
| `handlers/payroll/configs.rs` | 6 | lines 39, 120 (`.department_repo()`), 75, 87, 148, 160 (`.payroll().repo()`) |
| `handlers/payroll/incentives.rs` | 5 | lines 36, 44, 71, 97, 123 (`.payroll().repo().*`) |
| `handlers/payroll/salary_components.rs` | 5 | lines 36, 64, 108, 155 (`.payroll().repo()`), 98 (`.workspace_repo()`) |
| `handlers/payroll/settings.rs` | 5 | lines 29, 56, 96, 132 (`.payroll().repo()`), 84 (`.workspace_repo()`) |
| `handlers/payroll_rules.rs` | 8 | lines 71, 108, 136, 173, 201, 238, 265, 292 (`.payroll().rule_repo().*`) |
| `handlers/salary_bases.rs` | 5 | lines 37, 42, 75, 114, 141 (`.payroll().salary_base_repo().*`) |
| `handlers/system_auth.rs` | 1 | line 88 (`.core().user_repo().*`) |
| `handlers/system_db_clusters.rs` | 4 | lines 55, 94, 123, 154 (`.infra().system_db_cluster_repo().*`) |
| `handlers/work_segments.rs` | 7 | lines 85, 117, 149, 185, 219, 226, 262 (`.shifts().work_segment_repo().*`) |
| `handlers/working_patterns.rs` | 14 | lines 42, 72, 97, 150, 210, 241, 268, 302, 338, 355 (`.working_pattern_repo()`), 43, 303 (`.shift_repo()`), 44, 304 (`.roster_repo()`) |
| `handlers/workspaces/audit.rs` | 1 | line 61 (`.core().audit_repo().*`) |
| `handlers/workspaces/data_management.rs` | 1 | line 24 (`.core().workspace_repo().*`) |
| `handlers/workspaces/members.rs` | 10 | lines 58, 64, 125, 184, 191, 256 (`.workspace_repo()`), 116 (`.user_repo()`), 130, 196, 261 (`.audit_repo()`) |
| `handlers/workspaces/permissions.rs` | 5 | lines 43, 88, 157 (`.workspace_repo()`), 99, 163 (`.audit_repo()`) |

#### Category B: Hybrid Handlers (Repository + Service Calls) (27 files)
These handlers invoke both Services and direct Repository queries in the same endpoint workflows:

| Handler File | Repo Calls | Service Calls | Representative Direct Repository Citations |
|---|---|---|---|
| `handlers/agents.rs` | 19 | 7 | lines 106, 524 (`.workspace_repo()`), 362, 381, 439, 490, 533, 578, 600, 643, 649, 695, 724, 764, 798, 842, 875, 919, 954 (`.agents().repo()`) |
| `handlers/attendance/adjustments.rs` | 3 | 7 | line 49 (`.employment_repo()`), 59, 195 (`.employees().repo()`) |
| `handlers/attendance/import.rs` | 6 | 3 | lines 160, 255, 262, 299, 306, 343 (`.attendance().import_repo()`) |
| `handlers/attendance/overtime.rs` | 6 | 3 | lines 52, 147, 216, 232 (`.employees().repo()`), 83, 267 (`.attendance_daily_summary_repo()`) |
| `handlers/attendance/paper_forms.rs` | 4 | 2 | lines 61, 158 (`.employees().repo()`), 174 (`.workspace_repo()`), 241 (`.attendance_daily_summary_repo()`) |
| `handlers/attendance/reports.rs` | 5 | 3 | lines 116 (`.behalf_assignment_repo()`), 139, 153 (`.employees().repo()`), 145 (`.workspace_repo()`), 272 (`.feature_flag_repo()`) |
| `handlers/attendance/staging.rs` | 4 | 1 | lines 88, 94, 147, 191 (`.attendance().staging_repo().*`) |
| `handlers/attendance/time_tracking.rs` | 3 | 7 | lines 84, 133, 144 (`.employees().repo().*`) |
| `handlers/auth.rs` | 12 | 18 | lines 48, 80, 166, 240, 348, 377, 491, 499, 560, 598, 644, 714 (`.core().user_repo().*`) |
| `handlers/dashboard_metrics.rs` | 3 | 2 | lines 73 (`.workspace_repo()`), 74 (`.stats_repo()`), 93 (`.audit_repo()`) |
| `handlers/document_templates.rs` | 4 | 1 | lines 49 (`.workspace_repo()`), 50 (`.employees().repo()`), 51 (`.employment_repo()`), 52 (`.contract_repo()`) |
| `handlers/employees.rs` | 4 | 17 | lines 621 (`.employees().repo()`), 622, 653 (`.employment_repo()`), 623 (`.balance_repo()`) |
| `handlers/employments.rs` | 3 | 6 | lines 39, 82, 205 (`.employees().employment_repo().*`) |
| `handlers/ess.rs` | 16 | 7 | lines 36, 44, 51, 79, 102, 125, 151, 178, 201, 256, 295, 346 (`.employees().repo()`), 236 (`.workspace_repo()`), 266 (`.time_entry_repo()`), 392 (`.balance_repo()`), 425 (`.request_repo()`) |
| `handlers/holidays.rs` | 6 | 1 | lines 55, 88, 116, 150, 183, 219 (`.leaves().holiday_repo().*`) |
| `handlers/leaves.rs` | 15 | 9 | lines 133, 173, 276, 996 (`.request_repo()`), 382, 419, 455, 483, 522, 705, 733 (`.policy_repo()`), 567 (`.employees().repo()`), 799 (`.import_repo()`) |
| `handlers/overtime_request.rs` | 4 | 8 | lines 77, 323 (`.payroll().overtime_request_repo()`), 119, 274 (`.employment_repo()`) |
| `handlers/payroll/payslips.rs` | 10 | 2 | lines 103, 144, 149, 154, 161, 220, 228, 233, 238, 245 (`.payroll().repo().*`) |
| `handlers/recruitment_sources.rs` | 2 | 8 | lines 203, 332 (`.core().workspace_repo().*`) |
| `handlers/report_websocket.rs` | 1 | 1 | line 109 (`.core().workspace_repo().*`) |
| `handlers/shift_swaps.rs` | 1 | 3 | line 150 (`.shifts().swap_repo().*`) |
| `handlers/shifts.rs` | 13 | 6 | lines 118, 419 (`.workspace_repo()`), 440 (`.employment_repo()`), 503, 547, 608, 689, 778, 850 (`.roster_repo()`), 595, 652, 734, 840 (`.shifts().repo()`) |
| `handlers/system_admin.rs` | 3 | 1 | lines 426, 454, 515 (`.infra().system_settings_repo().*`) |
| `handlers/system_tenants.rs` | 4 | 4 | lines 137, 217, 246, 306 (`.core().workspace_repo().*`) |
| `handlers/workspaces/crud.rs` | 13 | 4 | lines 93, 153, 242, 412, 453, 477 (`.workspace_repo()`), 115, 196, 301, 335, 359, 383, 434 (`.stats_repo()`) |
| `handlers/workspaces/departments.rs` | 4 | 3 | lines 50, 100, 139, 176 (`.core().department_repo().*`) |
| `handlers/workspaces/roles.rs` | 12 | 1 | lines 64, 107, 166, 237, 274, 313, 352, 392, 425, 435 (`.workspace_repo()`), 112, 172 (`.audit_repo()`) |

### 4.3 Handlers Without Direct Repository Calls (22 files)

Of the 22 handlers that do not call repositories directly:
- **11 Strictly Clean Handlers:** Invoke only application services (L1 -> L2 compliance):
  1. `handlers/approval_workflow.rs`: 10 service calls to `ApprovalWorkflowService`
  2. `handlers/attendance/no_show.rs`: 1 service call to `ReportService::get_no_show_report`
  3. `handlers/condition_types.rs`: 5 service calls to `ConditionTypeService`
  4. `handlers/custom_fields.rs`: 7 service calls to `CustomFieldService`
  5. `handlers/files.rs`: 10 service calls to `FileService`
  6. `handlers/jobs.rs`: 1 service call to `InfraJobService`
  7. `handlers/lifecycle_checklists.rs`: 16 service calls to `LifecycleService`
  8. `handlers/signatures.rs`: 3 service calls to `SignatureService`
  9. `handlers/workspace_config.rs`: 4 service calls to `WorkspaceConfigService`
  10. `handlers/workspaces/employers.rs`: 5 service calls to `EmployerService`
  11. `handlers/workspaces/locations.rs`: 5 service calls to `LocationService`
- **3 Service-Calling Handlers with Direct Diesel / Pool Violations:**
  12. `handlers/monitor_alerts.rs`: 3 service calls, but imports `models::diesel::monitor_alert::*` (line 11)
  13. `handlers/storage_config.rs`: executes direct `diesel::insert_into` and `diesel::delete` queries (lines 174, 268)
  14. `handlers/worker_monitor.rs`: inspects pool status `state.infra().diesel().api.status()` (line 1265) and imports `models::diesel::infra_job::*` (lines 424, 450)
- **8 Non-Calling / Routing / Helper Files:**
  15. `handlers/mod.rs`: top-level handler router module declarations
  16. `handlers/attendance/mod.rs`: attendance router module declarations
  17. `handlers/payroll/mod.rs`: payroll router module declarations
  18. `handlers/workspaces/mod.rs`: workspace router module declarations
  19. `handlers/document_acknowledgements.rs`: mock HTTP handler stub returning JSON
  20. `handlers/document_folders.rs`: mock HTTP handler stub returning JSON
  21. `handlers/report_export.rs`: delegates asynchronously via `ReportCoordinator` helper
  22. `handlers/debug.rs`: dev-only endpoint directly instantiating `DieselDebugRepository::new` (lines 33, 67)

---

## 5. Transaction Ownership & Connection Acquisition Evidence

### 5.1 Connection Acquisition Architecture

Database connections in `apiserver` are resolved and acquired through two layers:

1. **Deadpool Pools (`apiserver/src/diesel_pool/core.rs:48-53`):**
   - `api: DieselPool` (System database connection pool, global users, clusters).
   - `worker: DieselPool` (Background workers).
   - `tenants_tx: Arc<dyn TenantConnectionProvider>` (Supavisor transactional pool provider).
   - `tenants_sess: Arc<dyn TenantConnectionProvider>` (Supavisor session pool provider).
2. **Tenant Pool Dynamic Manager (`apiserver/src/diesel_pool/tenant_manager.rs:236`):**
   - Dynamic per-tenant connection pooling with moka cache (capacity: 50 pools, 30m idle TTL).
   - `get_connection(&workspace_id)`: acquires an async connection from deadpool.

### 5.2 Transaction Demarcation Primitives

Transactions are created via three patterns:

1. **`DieselPools::with_tenant_tx` (`apiserver/src/diesel_pool/core.rs:72-93`):**
   ```rust
   let mut conn = self.tenants_tx.get_connection(&token.workspace_id()).await?;
   (&mut *conn).transaction::<T, AppError, _>(f).await
   ```
   This executes the user closure inside `BEGIN ... COMMIT` and rolls back on `Err`. Used in **299 call sites** across 50 files.
2. **`DieselPools::tenant_transaction_conn` (`apiserver/src/diesel_pool/core.rs:98-107`):**
   Acquires a `TransactionConn(conn)` without opening a transaction immediately, allowing caller-controlled transaction blocks. Used in **205 call sites** across 36 files.
3. **`conn.transaction(...)` (`diesel_async::AsyncConnection::transaction`):**
   Used across **45 distinct call sites** across 26 files (36 repository scopes, 8 service scopes, 1 diesel_pool core).

### 5.3 Evidence: Repository-Owned Transactions (Atomic Single-Entity Scopes)

Repositories own transactions for operations requiring multiple related inserts/updates within a single aggregate:
- `DieselEmployeeRepository::create_employee_tx` (`crud.rs:35, 153, 246, 344`): executes insert + audit log inside single `conn.transaction`.
- `DieselAttendancePolicyRepository::upsert_default_policy` (`diesel_attendance_policy_repository.rs:707, 949`): executes policy insert + rule binding inside `conn.transaction`.
- `DieselHolidayRepository::bulk_create_holidays` (`diesel_holiday_repository.rs:316`): bulk employment lookup and holiday log insertion inside `conn.transaction`.
- `DieselShiftRepository::create_shift` (`diesel_shift_repository.rs:64, 334, 503`): creates shift header and interval children inside `conn.transaction`.
- `DieselPayrollRepository::create_payroll_run` (`incentives.rs:151, 234`, `payslip.rs:297`): inserts payroll runs, batches, and payslip line items in a transaction.
- `DieselCustomFieldRepository` (`diesel_custom_field_repository.rs:205, 320`): field definition updates with cascade values in `conn.transaction`.
- `DieselApprovalWorkflowRepository` (`diesel_approval_workflow_repository.rs:101, 149, 793`): atomic workflow state transitions and task completions.

### 5.4 Evidence: Service-Owned Transactions (Cross-Repository Orchestration)

Three services directly own multi-repository transactions:

1. **`LeaveService` (`apiserver/src/services/leave_service/`):**
   - File: `approval.rs:53, 58`
     ```rust
     let mut conn = pools.tenant_transaction_conn(auth_token).await?;
     conn.transaction::<_, AppError, _>(|pg_conn| {
         Box::pin(async move {
             repo.update_leave_request_status_tx(pg_conn, auth, request_id, status, ...).await
         })
     }).await?;
     ```
   - File: `submission.rs:151, 163`
     ```rust
     let mut conn = pools.tenant_transaction_conn(auth_token).await?;
     conn.transaction::<_, AppError, _>(|pg_conn| {
         Box::pin(async move {
             repo.create_leave_request_tx(pg_conn, ...).await?;
             balance_repo.deduct_leave_balance_tx(pg_conn, ...).await
         })
     }).await?;
     ```
   *Architectural observation:* `LeaveService` injects `diesel_pools: Option<DieselPools>` solely to coordinate the atomic update of `leave_requests` and `leave_balances`. If `diesel_pools` is `None`, it falls back to non-transactional execution (`approval.rs:74`).
2. **`PairingService` (`apiserver/src/services/pairing/`):**
   - File: `service_range.rs:61-63, 231-233`:
     ```rust
     let mut tx_conn = pools.tenant_transaction_conn(&auth_token).await?;
     tx_conn.transaction(|conn| Box::pin(async move {
         save_range_entries_tx(service, &auth_token, &plan, conn).await
     })).await?;
     ```
   - File: `service_single.rs:183-185, 280-282`:
     ```rust
     let mut tx_conn = pools.tenant_transaction_conn(auth_token).await?;
     tx_conn.transaction(|conn| Box::pin(async move {
         save_paired_entries_and_update_summary_tx(service, auth_token, &plan, conn).await
     })).await?;
     ```
   - File: `service_helpers.rs:191, 274`:
     Transaction closures coordinating deletion of old auto entries followed by insertion of newly paired time entries and updating `attendance_daily_summaries`.
3. **`TenantProvisioner` (`apiserver/src/services/tenant_provisioner.rs:37, 86, 104, 131, 150`):**
   Directly manages database connections via `diesel_pools.api.get().await` and synchronous `diesel::PgConnection::establish(&db_url)` to execute DDL (`CREATE DATABASE`, `DROP DATABASE`, migration harnesses, and initial workspace seeding).

### 5.5 Evidence: Diagnostics Read-Only Transactions

- **`QualifyingDaysBlastRadius` (`apiserver/src/diagnostics/qualifying_days_blast_radius.rs:229`):**
  Uses `pools.with_tenant_tx(&token, |conn| Box::pin(async move { ... }))` to run:
  - `diesel::sql_query("SET TRANSACTION READ ONLY")` (line 231)
  - `diesel::sql_query("SET LOCAL statement_timeout = '120s'")` (line 237)
  - Raw analytical CTE report query `REPORT_QUERY` (line 244)

---

## 6. Go Package Mapping & Dependency-Direction Rules

### 6.1 Package Mapping Table

To eliminate the architectural decay found in Rust, the Go backend will map existing subsystems into strictly partitioned packages:

| Rust Source Directory | Target Go Package | Primary Package Responsibilities | Allowed Imports | Forbidden Imports |
|---|---|---|---|---|
| `apiserver/src/handlers/` | `internal/api/` | Implements `oapi-codegen` generated `StrictServerInterface`. HTTP serialization/status codes. | • `internal/service/`<br>• `internal/api/generated`<br>• `context`, `net/http` | • `internal/repository/`<br>• `internal/db/`<br>• `pgx/*`<br>• SQL strings |
| `apiserver/src/services/` | `internal/service/` | Business use-cases, workflow coordination, atomic transaction management (`RunInTx`). | • `internal/domain/...`<br>• `internal/repository/`<br>• `internal/tenant/`<br>• `shopspring/decimal` | • `internal/api/`<br>• `net/http`<br>• Raw SQL statements |
| `apiserver/src/domain/` | `internal/domain/` | Pure domain models, permissions, role hierarchy, audit metadata. | • `shopspring/decimal`<br>• Go standard library (`time`, `errors`) | • `database/sql`<br>• `pgx/*`<br>• `internal/repository/`<br>• `internal/api/` |
| `apiserver/src/services/pairing/`<br>`apiserver/src/services/attendance_calculator.rs` | `internal/domain/pairing/`<br>`internal/domain/matcher/` | In-process punch pairing, shift alignment, minute buckets. 100% pure functions. | • `shopspring/decimal`<br>• Standard library | • `database/sql`<br>• `pgx/*`<br>• Network / I/O<br>• `time.Now()` (must be arg) |
| `apiserver/src/services/financial_calculator.rs`<br>`apiserver/src/services/payroll_evaluator.rs` | `internal/domain/payroll/` | Gross-to-net calculation, OT multiplier formulas, tax tiers, deductions. | • `shopspring/decimal`<br>• Standard library | • `float32`, `float64`<br>• `database/sql`<br>• `pgx/*` |
| `apiserver/src/repositories/` (Traits) | `internal/repository/` | Interface definitions for storage operations (CRUD, paginated queries). | • `internal/domain/`<br>• `context` | • `internal/service/`<br>• `internal/api/`<br>• Driver implementations |
| `apiserver/src/repositories/diesel_*`<br>`apiserver/src/models/diesel/` | `internal/db/` | `sqlc` compiled queries, PostgreSQL parameter mappings, generated row structs. | • `github.com/jackc/pgx/v5`<br>• `internal/repository/` | • `internal/api/`<br>• `internal/service/` |
| `apiserver/src/diesel_pool/` | `internal/db/pool/` | `pgxpool.Pool` multi-tenant pool manager, Supavisor shard routing. | • `github.com/jackc/pgx/v5/pgxpool`<br>• `internal/tenant/` | • `internal/api/`<br>• `internal/domain/` |
| `apiserver/src/workspace_id_guard/` | `internal/tenant/` | Tenant context propagation (`context.Context`), tenant token validator, workspace guard. | • `context`, `uuid` | • `internal/domain/`<br>• `internal/repository/` |
| `apiserver/src/diagnostics/` | `internal/diagnostics/` | Blast-radius diagnostics, read-only analytics, maintenance CLI tools. | • `internal/db/pool/`<br>• `internal/tenant/` | • Mutation endpoints<br>• `internal/api/` |

### 6.2 Strict Dependency-Direction Invariants (Go Migration Contract)

```
[ HTTP Request ]
       │
       ▼
┌──────────────────────────────────────────────┐
│  internal/api/ (oapi-codegen Strict Server)  │
└──────────────────────────────────────────────┘
       │
       ▼  (Calls use-cases only; NEVER repositories or pgx)
┌──────────────────────────────────────────────┐
│  internal/service/ (Use-case Orchestration)  │
│  - Owns transaction boundary: RunInTx(ctx)   │
└──────────────────────────────────────────────┘
       │                             │
       ▼ (Pure logic)                ▼ (Storage interface)
┌───────────────────────────┐  ┌───────────────────────────────┐
│ internal/domain/          │  │ internal/repository/ (Traits) │
│ - pairing/ matcher/       │  └───────────────────────────────┘
│ - payroll/ policy/        │                ▲
│ - ZERO I/O, ZERO float64  │                │ (Implements)
└───────────────────────────┘  ┌───────────────────────────────┐
                               │ internal/db/ (sqlc + pgx/v5)  │
                               │ - DBTX interface (Pool or Tx) │
                               └───────────────────────────────┘
```

1. **Rule 1 (Compiler-Enforced Strict Server):** All HTTP routes are registered through `oapi-codegen` v2 `StrictServerInterface`. Handler methods must take `(ctx context.Context, request RequestObject)` and return `(ResponseObject, error)`. No ad-hoc HTTP parameter parsing.
2. **Rule 2 (No Handler Repository Bypass):** No package under `internal/api/` may import `internal/repository/`, `internal/db/`, or `pgx`. Handlers only communicate with `internal/service/`.
3. **Rule 3 (Transaction Interface & Ambient Tx):**
   - Repositories accept `DBTX` interface:
     ```go
     type DBTX interface {
         Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
         Query(context.Context, string, ...any) (pgx.Rows, error)
         QueryRow(context.Context, string, ...any) pgx.Row
     }
     ```
   - Services manage transactions via a standard runner:
     ```go
     func (s *LeaveService) RunInTx(ctx context.Context, fn func(txRepo repository.Registry) error) error
     ```
   - Connection pools are never passed to handlers or leaked outside `internal/db/pool`.
4. **Rule 4 (Prohibition of Float in Calculations):**
   Every monetary, overtime hour, and leave accrual value in `internal/domain/...` must use `shopspring/decimal.Decimal`. Use of `float32` or `float64` will be flagged as a build error by static analysis.

---

## 7. Metrics Summary: Counts, Violations & Unknowns

### 7.1 Exact Metrics Census

| Metric Category | Exact Count | Verification Ground Truth |
|---|---|---|
| **Concrete Service Types** | **60** | Complete census across `apiserver/src/services/` (Section 2) |
| **Repository Traits** | **65** | Declarations in `apiserver/src/repositories/` (Section 3) |
| **Diesel Repository Implementations** | **54** | Structs named `Diesel*Repository` in `repositories/` |
| **Non-Diesel Repository Implementations** | **3** | `RedisCache` (`CacheRepository`), `RedisPublisherRepository`, `RedisQueueRepository` |
| **Total Handler Files** | **77** | Total `.rs` files in `apiserver/src/handlers/` |
| **Handlers Calling Repositories Directly** | **55** | 28 exclusive repository bypass + 27 hybrid (Section 4) |
| **Handlers Calling Services Without Repositories** | **14** | 11 strictly clean + 3 with internal Diesel/pool references |
| **Handlers Strictly Clean (L1 -> L2 only)** | **11** | Section 4.3 clean list |
| **Routing / Submodule / Stub Handler Files** | **8** | 4 `mod.rs` files + 2 stubs + 1 coordinator delegator + 1 debug |
| **Handlers with Direct SQL / Diesel DML** | **3** | `common.rs:534`, `storage_config.rs:174,268`, `system_admin.rs:88,112,193,252,264,339,356` |
| **Handlers with Direct DB Pool Access / Status** | **4** | `common.rs:532`, `storage_config.rs:97,132,264`, `system_admin.rs:72,180,236,317`, `worker_monitor.rs:1265` |
| **Handlers Instantiating Concrete Repository** | **1** | `debug.rs:33,67` |
| **Handlers Importing Internal Diesel Entities** | **3** | `monitor_alerts.rs:11`, `shifts.rs:199,203`, `worker_monitor.rs:424,450` |
| **Internal Repository Transaction Scopes** | **36** | `conn.transaction(...)` scopes across 20 repository files |
| **Services Owning Cross-Repo Transactions** | **3** | `LeaveService`, `PairingService`, `TenantProvisioner` (8 scopes) |
| **with_tenant_tx Call Sites** | **299** | Across 50 files in repositories & diagnostics |
| **tenant_transaction_conn Call Sites** | **205** | Across 36 files in repositories & services |
| **Unimplemented Repository Traits** | **1** | `ShiftIntervalRepository` (orphan trait) |

### 7.2 Architectural Violations in Rust Codebase

1. **55 of 77 Handlers Bypass Service Layer:** 71% of handler files directly access repository methods via `state.<domain>().repo()`, bypassing application business logic, validation, and domain invariants.
2. **Raw SQL in System Admin Handler (`system_admin.rs`):** `diesel::sql_query` is invoked directly inside HTTP handlers for managing system roles and user assignments instead of using `SystemRepository` or `UserRepository`.
3. **Storage Config Handler Executes Direct DML (`storage_config.rs`):** Direct calls to `diesel::insert_into` and `diesel::delete` inside handlers, bypassing `StorageConfigRepository`.
4. **Concrete Pool Dependency in Services:** `LeaveService` and `PairingService` take `diesel_pools: Option<DieselPools>` directly to manage transactions, leaking persistence driver types into the service layer.
5. **Fallback Non-Transactional Code Paths:** `LeaveService::approve_leave_request` executes non-transactionally if `diesel_pools` is not injected (`approval.rs:73-77`), introducing potential partial-write inconsistency in tests or misconfigured runtime setups.

### 7.3 Unknowns and Anomalies Identified

1. **Orphan Trait `ShiftIntervalRepository`:**
   - Declared at `apiserver/src/repositories/shift_interval_repository.rs:8`.
   - Has **0 implementations** in either production or test code.
   - *Investigation finding:* Shift intervals were subsumed into `DieselRosterRepository` and `ShiftDefaultIntervalEntity` inside `apiserver/src/models/diesel/roster.rs`. In the Go migration, this orphan trait must be discarded.
2. **Direct Handler Dependency on `SalaryBaseRepository`:**
   - Trait declared at `repositories/salary_base_repository.rs:7`, implemented at `repositories/diesel_salary_base_repository.rs:70`.
   - Has no corresponding service type in `apiserver/src/services/`.
   - Directly consumed by `handlers/salary_bases.rs`.
   - *Migration requirement:* Introduce `SalaryBaseService` in `internal/service/salary/` to encapsulate salary base tier queries.
3. **Dual Pool vs Direct Migration Connection in `TenantProvisioner`:**
   - Uses async deadpool connection `api.get()` for database status, but switches to synchronous `diesel::PgConnection::establish` for executing tenant DDL migrations (`services/tenant_provisioner.rs:150`).
   - *Migration requirement:* In Go, DDL migrations and tenant provisioning will standardize on `pgx/v5` connections with explicit timeouts.
