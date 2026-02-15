# Scout Report: Critical Source Files

**Date:** 2026-02-15  
**Codebase:** leeattend  
**Scope:** Comprehensive catalog of critical source files across apiserver, agent-main, and webdashboard

---

## Overview

Complete inventory of key source files organized by module and layer:
- **apiserver/src/** - Rust API server (handlers, services, repositories, domain, models)
- **agent-main/src/** - Rust agent (worker, memory store, database, config)
- **webdashboard/src/** - TypeScript/React web dashboard (hooks, context, lib, components)

---

## 1. APISERVER (Rust)

### 1.1 Handlers (Request/Response Layer)

**Top-level handlers:**
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/mod.rs` - Handler registry
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/auth.rs` - Authentication endpoints
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/common.rs` - Common utilities
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/debug.rs` - Debug endpoints
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/agents.rs` - Agent management
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/employees.rs` - Employee endpoints
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/shifts.rs` - Shift endpoints
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/leaves.rs` - Leave request endpoints
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/overtime.rs` - Overtime endpoints
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/ess.rs` - Employee Self-Service
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/assignment_rules.rs` - Assignment rules
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/working_patterns.rs` - Working pattern config
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/leave_types.rs` - Leave type management
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/report_websocket.rs` - WebSocket reports
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/report_export.rs` - Report export
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/worker_monitor.rs` - Worker monitoring

**Attendance handlers (module):**
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/mod.rs` - Attendance registry
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/time_tracking.rs` - Time tracking
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/import.rs` - Attendance import
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/logs.rs` - Attendance logs
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/adjustments.rs` - Adjustments
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/reports.rs` - Report generation
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/staging.rs` - Staging data
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/overtime.rs` - Overtime processing
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/attendance/paper_forms.rs` - Paper forms

**Workspace handlers (module):**
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/workspaces/mod.rs` - Workspace registry
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/workspaces/crud.rs` - CRUD operations
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/workspaces/members.rs` - Member management
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/workspaces/roles.rs` - Role management
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/workspaces/departments.rs` - Department config
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/workspaces/data_management.rs` - Data management

**Payroll handlers (module):**
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/payroll/mod.rs` - Payroll registry
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/payroll/payslips.rs` - Payslip generation
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/payroll/salary_components.rs` - Component config
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/payroll/incentives.rs` - Incentives
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/payroll/settings.rs` - Settings
- `/home/elt1541/lee/leeattend/apiserver/src/handlers/payroll/configs.rs` - Configuration

### 1.2 Services (Business Logic Layer)

**Core services:**
- `/home/elt1541/lee/leeattend/apiserver/src/services/mod.rs` - Service registry
- `/home/elt1541/lee/leeattend/apiserver/src/services/auth_service.rs` - Authentication logic
- `/home/elt1541/lee/leeattend/apiserver/src/services/employee_service.rs` - Employee operations
- `/home/elt1541/lee/leeattend/apiserver/src/services/workspace_service.rs` - Workspace operations

**Attendance services:**
- `/home/elt1541/lee/leeattend/apiserver/src/services/attendance_calculator.rs` - Calculation engine
- `/home/elt1541/lee/leeattend/apiserver/src/services/attendance_log_service.rs` - Log management
- `/home/elt1541/lee/leeattend/apiserver/src/services/attendance_summary_service.rs` - Summaries
- `/home/elt1541/lee/leeattend/apiserver/src/services/attendance_import_service.rs` - Import logic
- `/home/elt1541/lee/leeattend/apiserver/src/services/attendance_adjustment_service.rs` - Adjustments

**Shift & pattern services:**
- `/home/elt1541/lee/leeattend/apiserver/src/services/shift_service.rs` - Shift management
- `/home/elt1541/lee/leeattend/apiserver/src/services/working_pattern_service.rs` - Pattern logic
- `/home/elt1541/lee/leeattend/apiserver/src/services/pairing_service.rs` - Pairing algorithm

**Leave services:**
- `/home/elt1541/lee/leeattend/apiserver/src/services/leave_service.rs` - Leave requests
- `/home/elt1541/lee/leeattend/apiserver/src/services/leave_accrual_service.rs` - Accrual engine

**Payroll services:**
- `/home/elt1541/lee/leeattend/apiserver/src/services/payroll_service.rs` - Payroll orchestration
- `/home/elt1541/lee/leeattend/apiserver/src/services/payroll_evaluator.rs` - Evaluation logic

**Reporting services:**
- `/home/elt1541/lee/leeattend/apiserver/src/services/report_service.rs` - Report generation
- `/home/elt1541/lee/leeattend/apiserver/src/services/report_coordinator.rs` - Coordination

**Utility services:**
- `/home/elt1541/lee/leeattend/apiserver/src/services/agent_service.rs` - Agent operations
- `/home/elt1541/lee/leeattend/apiserver/src/services/agent_log_buffer.rs` - Log buffering
- `/home/elt1541/lee/leeattend/apiserver/src/services/time_tracking_service.rs` - Time tracking
- `/home/elt1541/lee/leeattend/apiserver/src/services/excel_writer.rs` - Excel export
- `/home/elt1541/lee/leeattend/apiserver/src/services/rule_evaluator.rs` - Rule evaluation
- `/home/elt1541/lee/leeattend/apiserver/src/services/pairing_algo.rs` - Pairing algorithms

### 1.3 Repositories (Data Access Layer)

**User & workspace repos:**
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/user_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/workspace_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/employee_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/department_repository.rs`

**Attendance repos:**
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/attendance_log_error_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/raw_attendance_log_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/staging_log_repository.rs`

**Shift & pattern repos:**
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/shift_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/roster_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/working_pattern_repository.rs`

**Leave repos:**
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/leave_request_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/leave_balance_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/leave_policy_repository.rs`

**Payroll & policy repos:**
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/payroll_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/overtime_policy_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/assignment_rule_repository.rs`

**Utility repos:**
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/mod.rs` - Repository registry
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/agent_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/import_job_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/time_entry_repository.rs`
- `/home/elt1541/lee/leeattend/apiserver/src/repositories/attendance_adjustment_repository.rs`

### 1.4 Domain Models

- `/home/elt1541/lee/leeattend/apiserver/src/domain/mod.rs` - Domain registry
- `/home/elt1541/lee/leeattend/apiserver/src/domain/policy/mod.rs` - Policy models
- `/home/elt1541/lee/leeattend/apiserver/src/domain/policy/resolver.rs` - Policy resolution
- `/home/elt1541/lee/leeattend/apiserver/src/domain/policy/overtime.rs` - Overtime policies
- `/home/elt1541/lee/leeattend/apiserver/src/models.rs` - Core data models

### 1.5 Caching & Queue

**Cache layer:**
- `/home/elt1541/lee/leeattend/apiserver/src/cache/mod.rs` - Cache registry
- `/home/elt1541/lee/leeattend/apiserver/src/cache/manager.rs` - Cache management
- `/home/elt1541/lee/leeattend/apiserver/src/cache/redis_cache.rs` - Redis implementation
- `/home/elt1541/lee/leeattend/apiserver/src/cache/workspace_cache.rs` - Workspace cache
- `/home/elt1541/lee/leeattend/apiserver/src/cache/shift_cache.rs` - Shift cache
- `/home/elt1541/lee/leeattend/apiserver/src/cache/roster_cache.rs` - Roster cache

**Queue layer:**
- `/home/elt1541/lee/leeattend/apiserver/src/queue/mod.rs` - Queue registry
- `/home/elt1541/lee/leeattend/apiserver/src/queue/job.rs` - Job definitions
- `/home/elt1541/lee/leeattend/apiserver/src/queue/redis_client.rs` - Redis client
- `/home/elt1541/lee/leeattend/apiserver/src/queue/pubsub.rs` - Pub/Sub
- `/home/elt1541/lee/leeattend/apiserver/src/queue/report_queue.rs` - Report queue

### 1.6 Worker & Background Tasks

- `/home/elt1541/lee/leeattend/apiserver/src/worker/mod.rs` - Worker registry
- `/home/elt1541/lee/leeattend/apiserver/src/worker/runner.rs` - Task runner
- `/home/elt1541/lee/leeattend/apiserver/src/worker/handlers/mod.rs` - Handler registry
- `/home/elt1541/lee/leeattend/apiserver/src/worker/handlers/attendance.rs` - Attendance tasks
- `/home/elt1541/lee/leeattend/apiserver/src/worker/handlers/merge.rs` - Merge operations

### 1.7 Utilities & Core

- `/home/elt1541/lee/leeattend/apiserver/src/main.rs` - Application entry point
- `/home/elt1541/lee/leeattend/apiserver/src/openapi.rs` - OpenAPI documentation
- `/home/elt1541/lee/leeattend/apiserver/src/agent_manager.rs` - Agent management
- `/home/elt1541/lee/leeattend/apiserver/src/dev_initialization.rs` - Development setup
- `/home/elt1541/lee/leeattend/apiserver/src/gen_hash.rs` - Hash generation
- `/home/elt1541/lee/leeattend/apiserver/src/utils/mod.rs` - Utility registry
- `/home/elt1541/lee/leeattend/apiserver/src/utils/datetime.rs` - DateTime utilities
- `/home/elt1541/lee/leeattend/apiserver/src/utils/serde_helpers.rs` - Serde helpers
- `/home/elt1541/lee/leeattend/apiserver/src/utils/debug.rs` - Debug utilities

---

## 2. AGENT-MAIN (Rust)

### 2.1 Core Components

- `/home/elt1541/lee/leeattend/agent-main/src/main.rs` - Application entry point
- `/home/elt1541/lee/leeattend/agent-main/src/lib.rs` - Library exports
- `/home/elt1541/lee/leeattend/agent-main/src/config.rs` - Configuration management
- `/home/elt1541/lee/leeattend/agent-main/src/cli.rs` - CLI argument parsing

### 2.2 Client Communication

**Central client:**
- `/home/elt1541/lee/leeattend/agent-main/src/central_client/mod.rs` - Client registry
- `/home/elt1541/lee/leeattend/agent-main/src/central_client/connection.rs` - Connection handling
- `/home/elt1541/lee/leeattend/agent-main/src/central_client/message_handler.rs` - Message processing
- `/home/elt1541/lee/leeattend/agent-main/src/central_client/retry_manager.rs` - Retry logic

**Worker client:**
- `/home/elt1541/lee/leeattend/agent-main/src/worker_client.rs` - Worker communication
- `/home/elt1541/lee/leeattend/agent-main/src/worker_processor.rs` - Task processing

**ZK client:**
- `/home/elt1541/lee/leeattend/agent-main/src/zk_client.rs` - ZooKeeper integration

### 2.3 Database Layer

**Database operations:**
- `/home/elt1541/lee/leeattend/agent-main/src/database/mod.rs` - Database registry
- `/home/elt1541/lee/leeattend/agent-main/src/database/read_ops.rs` - Read operations
- `/home/elt1541/lee/leeattend/agent-main/src/database/write_ops.rs` - Write operations
- `/home/elt1541/lee/leeattend/agent-main/src/database/sync_ops.rs` - Synchronization
- `/home/elt1541/lee/leeattend/agent-main/src/database/maintenance.rs` - Maintenance tasks
- `/home/elt1541/lee/leeattend/agent-main/src/database/device_management.rs` - Device management

**Hybrid database:**
- `/home/elt1541/lee/leeattend/agent-main/src/hybrid_database.rs` - Dual storage abstraction

### 2.4 Memory Store

- `/home/elt1541/lee/leeattend/agent-main/src/memory_store/mod.rs` - In-memory registry
- `/home/elt1541/lee/leeattend/agent-main/src/memory_store/store.rs` - Store implementation
- `/home/elt1541/lee/leeattend/agent-main/src/memory_store/types.rs` - Type definitions
- `/home/elt1541/lee/leeattend/agent-main/src/memory_store/backoff.rs` - Backoff logic

### 2.5 Utilities

- `/home/elt1541/lee/leeattend/agent-main/src/models.rs` - Data models
- `/home/elt1541/lee/leeattend/agent-main/src/utils.rs` - Utility functions
- `/home/elt1541/lee/leeattend/agent-main/src/retry.rs` - Retry mechanisms
- `/home/elt1541/lee/leeattend/agent-main/src/crash_recovery.rs` - Crash recovery
- `/home/elt1541/lee/leeattend/agent-main/src/adaptive_polling.rs` - Polling strategy

---

## 3. WEBDASHBOARD (TypeScript/React)

### 3.1 Hooks

**Custom hooks:**
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-workspace.ts` - Workspace context
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-dashboard.ts` - Dashboard state
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-profile.ts` - User profile
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-attendance.ts` - Attendance data
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-ess.ts` - ESS operations
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-agents.ts` - Agent management
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-devices.ts` - Device tracking

**Utility hooks:**
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-language.ts` - i18n language
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-i18n-sync.ts` - i18n synchronization
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-theme.ts` - Theme management
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-dark-mode.ts` - Dark mode toggle
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-debounce.ts` - Debouncing
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-websocket.ts` - WebSocket connection
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-report-stream.ts` - Report streaming
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/use-worker-monitor.ts` - Worker monitoring

**Test files:**
- `/home/elt1541/lee/leeattend/webdashboard/src/hooks/__tests__/` - Comprehensive hook tests

### 3.2 Context Providers

- `/home/elt1541/lee/leeattend/webdashboard/src/context/DarkModeContext.tsx` - Dark mode provider
- `/home/elt1541/lee/leeattend/webdashboard/src/context/DebugContext.tsx` - Debug context
- `/home/elt1541/lee/leeattend/webdashboard/src/context/use-dark-mode-context.ts` - Dark mode hook
- `/home/elt1541/lee/leeattend/webdashboard/src/context/dark-mode-context-definition.ts` - Definitions

### 3.3 API Integration

**API modules:**
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/index.ts` - API registry
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/auth.ts` - Authentication API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/employees.ts` - Employee API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/attendance.ts` - Attendance API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/shifts.ts` - Shift API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/workspaces.ts` - Workspace API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/departments.ts` - Department API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/leaves.ts` - Leave API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/ess.ts` - ESS API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/payroll.ts` - Payroll API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/rules.ts` - Rules API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/agents.ts` - Agent API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/config.ts` - Configuration API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/working-patterns.ts` - Pattern API
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/debug.ts` - Debug API

**API utilities:**
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/types.ts` - Type definitions
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/api/helpers.ts` - Helper functions

### 3.4 Utilities & Helpers

**Utility modules:**
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/utils.ts` - General utilities
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/datetime.ts` - DateTime utilities
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/excel-builder.ts` - Excel generation
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/utils/attendance.ts` - Attendance helpers

**Test utilities:**
- `/home/elt1541/lee/leeattend/webdashboard/src/lib/__tests__/` - Library tests

### 3.5 Components (Key Categories)

**Layout components:**
- `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/MainLayout.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/Sidebar.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/InfraLayout.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/layout/WorkspaceSwitcher.tsx`

**Common components:**
- `/home/elt1541/lee/leeattend/webdashboard/src/components/common/AuthGuard.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/common/WorkspaceGuard.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/common/EmployeeSearch.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/common/MonthYearPicker.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/common/PaginationControls.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/common/TemporalConfigTable.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/common/HierarchyLegend.tsx`

**Feature modules:**

*Attendance:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/attendance/AttendanceImportDialog.tsx`

*Reports:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/reports/DailyReportTable.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/reports/SummaryReportTable.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/reports/ReportFilters.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/reports/ReportStats.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/reports/ReportExportDialog.tsx`

*Agents:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/agents/AgentEditModal.tsx`

*Connectivity:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/connectivity/ConnectivityTopology.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/connectivity/AgentAccordionCard.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/connectivity/DeviceListItem.tsx`

*Workspaces:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/workspaces/CreateWorkspaceDialog.tsx`

*Settings:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/features/settings/RoleManagementModal.tsx`

**Domain-specific components:**

*Time tracking:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/time-tracking/PunchCard.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/time-tracking/TimeEntryRow.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/time-tracking/AdjustmentRequestModal.tsx`

*Leave management:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/leave/AssignPolicyModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/leave/LeavePolicyManager.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/leave/LeaveTypeManager.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/leave/LeaveHistoryModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/leave/LeaveBalanceDetailModal.tsx`

*Shift management:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/ShiftList.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/CreateShiftModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/EditShiftModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/ShiftPickerModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/WeeklyRosterGrid.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/PatternAssignmentForm.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/AssignmentHistory.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/ScheduleTracePopover.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/shifts/PriorityLegend.tsx`

*Payroll:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/PayrollTable.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/PayrollSummaryTable.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/PayrollSettingsSection.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/PayrollDetailDrawer.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/PayrollPriorityLegend.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/PayslipModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/PayslipDocument.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/SalaryComponentManager.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/EmployeeSalaryConfig.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/OvertimePolicyManager.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/AssignOvertimePolicyModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/OvertimeRuleEditor.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/BonusPolicyManager.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/BatchCalculationModal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/AdjustmentDrawer.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/payroll/SegmentVisualizer.tsx`

*Employee management:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/employees/WorkingConditionConfig.tsx`

*Members:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/members/MemberTable.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/members/InviteMemberDialog.tsx`

*ESS (Employee Self-Service):*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ess/ClaimOtDialog.tsx`

*Approvals:*
- `/home/elt1541/lee/leeattend/webdashboard/src/components/approvals/EnterPaperOtDialog.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/approvals/BulkUploadOtDialog.tsx`

**UI Components:**
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/Modal.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/Sheet.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/Tabs.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/Combobox.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/ComboboxAutoFocus.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/dropdown-menu.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/LanguageSwitcher.tsx`
- `/home/elt1541/lee/leeattend/webdashboard/src/components/ui/Toast.tsx`

**Debug components:**
- `/home/elt1541/lee/leeattend/webdashboard/src/components/debug/DebugToolbar.tsx`

---

## Summary Statistics

| Category | Count |
|----------|-------|
| **APISERVER Handlers** | 39 files |
| **APISERVER Services** | 20+ files |
| **APISERVER Repositories** | 18 files |
| **APISERVER Domain/Utils/Cache/Queue** | 15+ files |
| **APISERVER Total** | ~100 files |
| **AGENT-MAIN** | 32 files |
| **WEBDASHBOARD Hooks** | 26 files |
| **WEBDASHBOARD Context** | 4 files |
| **WEBDASHBOARD API** | 18 files |
| **WEBDASHBOARD Utilities** | 10+ files |
| **WEBDASHBOARD Components** | 150+ files (partial list) |
| **WEBDASHBOARD Total** | 200+ files |
| **GRAND TOTAL** | 330+ files |

---

## Key Architectural Observations

1. **APISERVER** - Layered architecture (Handlers → Services → Repositories) with domain models and caching
2. **AGENT-MAIN** - Client/server communication with hybrid database and memory store
3. **WEBDASHBOARD** - React hooks pattern with centralized API layer and context providers

All paths are absolute file paths suitable for IDE navigation and code analysis.

