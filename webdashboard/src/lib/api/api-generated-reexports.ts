/**
 * Re-exports of generated OpenAPI schema types (components["schemas"]["X"]).
 *
 * This file is the home for every 1:1 alias that previously lived in
 * `types.ts`. Keeping them here means `types.ts` itself only carries genuine
 * overrides (Omit/&), hand-written interfaces, and schema gaps (Group B/C) —
 * see `types.ts` for those.
 *
 * Consumer code imports from `@/lib/api/types` (which `export * from`s this
 * file), so there is NO migration needed on the consumer side.
 */

import type { components } from "@/types/api-generated";

export type DeviceStatus = components["schemas"]["DeviceStatus"];
export type AgentStatus = components["schemas"]["AgentStatus"];
export type Device = components["schemas"]["Device"];
export type AuthorizedAgent = components["schemas"]["AuthorizedAgent"];
export type Employer = components["schemas"]["Employer"];
export type CreateEmployerRequest = components["schemas"]["CreateEmployerRequest"];
export type UpdateEmployerRequest = components["schemas"]["UpdateEmployerRequest"];
export type UpdateUserRequest = components["schemas"]["UpdateUserRequest"];
export type ChangePasswordRequest = components["schemas"]["ChangePasswordRequest"];
export type RegisterRequest = components["schemas"]["RegisterRequest"];
export type AuthResponse = components["schemas"]["AuthResponse"];
export type LeaveRequest = components["schemas"]["LeaveRequest"];
export type WorkspaceLeaveBalance = components["schemas"]["WorkspaceLeaveBalance"];
export type Holiday = components["schemas"]["Holiday"];
export type CreateShiftRequest = components["schemas"]["CreateShiftRequest"];
export type UpdateShiftRequest = components["schemas"]["UpdateShiftRequest"];
export type PayrollSettings = components["schemas"]["PayrollSettings"];
export type UpdatePayrollSettingsRequest = components["schemas"]["UpdatePayrollSettingsRequest"];
export type SalaryComponentDefinition = components["schemas"]["SalaryComponentDefinition"];
export type CreateSalaryComponentRequest = components["schemas"]["CreateSalaryComponentRequest"];
export type CreatePayrollRuleDefinitionRequest = components["schemas"]["CreatePayrollRuleDefinitionRequest"];
export type SalaryBaseGroup = components["schemas"]["SalaryBaseGroup"];
export type SalaryBaseGroupMember = components["schemas"]["SalaryBaseGroupMember"];
export type CompensationHistoryResponse = components["schemas"]["CompensationHistoryResponse"];
export type CompensationTimelineEvent = components["schemas"]["CompensationTimelineEvent"];
export type CompensationChangeDetails = components["schemas"]["CompensationChangeDetails"];
export type CompensationEmploymentContext = components["schemas"]["CompensationEmploymentContext"];
export type IncentiveTier = components["schemas"]["IncentiveTier"];
export type PayrollIncentivePolicy = components["schemas"]["PayrollIncentivePolicy"];
export type PayrollIncentivePolicyResponse = components["schemas"]["PayrollIncentivePolicyResponse"];
export type AssigneeType = components["schemas"]["AssigneeType"];
export type BatchCalculatePayrollRequest = components["schemas"]["BatchCalculatePayrollRequest"];
export type BatchCalculateResult = components["schemas"]["BatchCalculateResult"];
export type BatchCalculatePayrollResponse = components["schemas"]["BatchCalculatePayrollResponse"];
export type EmployeeDepartmentHistory = components["schemas"]["EmployeeDepartmentHistory"];
export type ShiftRoster = components["schemas"]["ShiftRoster"];
export type AttendanceImportJob = components["schemas"]["AttendanceImportJob"];
export type MemberExtraPermission = components["schemas"]["MemberExtraPermission"];
export type LeaveTypeConfig = components["schemas"]["LeaveTypeConfig"];
export type LeaveType = components["schemas"]["LeaveType"];
export type ScheduleTrace = components["schemas"]["ScheduleTrace"];
export type EffectiveSchedule = components["schemas"]["EffectiveSchedule"];
export type AssignmentHistory = components["schemas"]["AssignmentHistory"];
export type LeaveHistoryItem = components["schemas"]["LeaveHistoryItem"];
export type CreateEmployeeAssignmentRequest = components["schemas"]["CreateEmployeeAssignmentRequest"];
export type EmployeeEmergencyContact = components["schemas"]["EmployeeEmergencyContact"];
export type EmployeeDocument = components["schemas"]["EmployeeDocument"];
export type CreateEmergencyContactRequest = components["schemas"]["CreateEmergencyContactRequest"];
export type WorkingPatternDetail = components["schemas"]["WorkingPatternDetail"];
export type EmployeeWorkingPattern = components["schemas"]["EmployeeWorkingPattern"];
export type CreateWorkingPatternRequest = components["schemas"]["CreateWorkingPatternRequest"];
export type AssignPatternRequest = components["schemas"]["AssignPatternRequest"];
export type BulkAssignResponse = components["schemas"]["BulkAssignResponse"];
export type BulkAssignPatternRequest = components["schemas"]["BulkAssignPatternRequest"];
export type CreateWorkingConditionRequest = components["schemas"]["CreateWorkingConditionRequest"];
export type OvertimePolicy = components["schemas"]["OvertimePolicy"];
export type OvertimeRule = components["schemas"]["OvertimeRule"];
export type PayrollRuleDefinition = components["schemas"]["PayrollRuleDefinition"];
export type PayrollRuleTier = components["schemas"]["PayrollRuleTier"];
export type CreateEmploymentPhaseDefinitionRequest =
  components["schemas"]["CreateEmploymentPhaseDefinitionRequest"];
export type AssignEmployeePhaseRequest = components["schemas"]["AssignEmployeePhaseRequest"];
export type UpdateEmployeePhaseStatusRequest = components["schemas"]["UpdateEmployeePhaseStatusRequest"];
export type WorkspaceConfigEntry = components["schemas"]["WorkspaceConfigEntry"];
export type CreateWorkflowRequest = components["schemas"]["CreateWorkflowRequest"];
export type ApprovalActionLog = components["schemas"]["ApprovalActionLog"];
export type ProcessApprovalRequest = components["schemas"]["ProcessApprovalRequest"];
export type OvertimeRequestEmployee = components["schemas"]["OvertimeRequestEmployee"];
export type OvertimeRequest = components["schemas"]["OvertimeRequest"];
export type OvertimeRequestDetail = components["schemas"]["OvertimeRequestDetail"];
export type CreateOvertimeRequestPayload = components["schemas"]["CreateOvertimeRequestPayload"];
export type BehalfAssignment = components["schemas"]["BehalfAssignment"];
export type CreateBehalfAssignmentRequest = components["schemas"]["CreateBehalfAssignmentRequest"];
export type CustomFieldDefinition = components["schemas"]["CustomFieldDefinition"];
export type CreateCustomFieldDefinitionRequest = components["schemas"]["CreateCustomFieldDefinitionRequest"];
export type UpdateCustomFieldDefinitionRequest = components["schemas"]["UpdateCustomFieldDefinitionRequest"];
export type SetCustomFieldValueRequest = components["schemas"]["SetCustomFieldValueRequest"];
export type BulkSetCustomFieldValuesRequest = components["schemas"]["BulkSetCustomFieldValuesRequest"];
export type CustomFieldValueResponse = components["schemas"]["CustomFieldValueResponse"];
export type LifecycleChecklistTemplate = components["schemas"]["LifecycleChecklistTemplate"];
export type CreateLifecycleChecklistTemplateRequest =
  components["schemas"]["CreateLifecycleChecklistTemplateRequest"];
export type UpdateLifecycleChecklistTemplateRequest =
  components["schemas"]["UpdateLifecycleChecklistTemplateRequest"];
export type LifecycleChecklistTemplateItem = components["schemas"]["LifecycleChecklistTemplateItem"];
export type CreateLifecycleChecklistTemplateItemRequest =
  components["schemas"]["CreateLifecycleChecklistTemplateItemRequest"];
export type UpdateLifecycleChecklistTemplateItemRequest =
  components["schemas"]["UpdateLifecycleChecklistTemplateItemRequest"];
export type LifecycleChecklistResponse = components["schemas"]["LifecycleChecklistResponse"];
export type LifecycleChecklistItem = components["schemas"]["LifecycleChecklistItem"];
export type ChecklistItemStatus = components["schemas"]["ChecklistItemStatus"];
export type CreateChecklistRequest = components["schemas"]["CreateChecklistRequest"];
export type PendingTaskResponse = components["schemas"]["PendingTaskResponse"];
export type UpdateChecklistItemStatusRequest = components["schemas"]["UpdateChecklistItemStatusRequest"];
export type ChecklistCategory = components["schemas"]["ChecklistCategory"];
export type ChecklistAssigneeType = components["schemas"]["ChecklistAssigneeType"];
export type ChecklistStatus = components["schemas"]["ChecklistStatus"];
export type LifecycleChecklistType = components["schemas"]["LifecycleChecklistType"];
export type ValidTerminationRow = components["schemas"]["ValidTerminationRow"];
export type InvalidTerminationRow = components["schemas"]["InvalidTerminationRow"];
export type ProratedLeavePreview = components["schemas"]["ProratedLeavePreview"];
