import type { Employee } from "./types";

// --- Employee Self-Service (ESS) Types ---
export interface EmployeeEmergencyContact {
  contact_name: string;
  created_at: string;
  employee_id: string;
  id: string;
  is_primary: boolean;
  phone: string;
  relationship: string;
}

export interface EmployeeProfile {
  address?: string | null;
  avatar_file_id?: string | null;
  bank_account_number?: string | null;
  bank_branch?: string | null;
  bank_name?: string | null;
  created_at?: string;
  date_of_birth?: string | null;
  employee_id?: string;
  gender?: string | null;
  id?: string;
  national_id?: string | null;
  phone?: string | null;
  updated_at?: string;
}

export interface MyProfileResponse {
  emergency_contacts: EmployeeEmergencyContact[];
  employee: Employee;
  profile?: EmployeeProfile | null;
}

export interface UpdateProfileRequest {
  address?: string;
  avatar_file_id?: string | null;
  bank_account_number?: string;
  bank_branch?: string;
  bank_name?: string;
  date_of_birth?: string;
  gender?: string;
  national_id?: string;
  phone?: string;
}

// --- Approval Workflows ---
export type ApproverType = "department_manager" | "role" | "permission" | "specific_user";

/** Configuration for the approver in a workflow step.
 *  Shape depends on approver_type:
 *  - department_manager / role / permission → `{ role_id?: string; permission_key?: string }`
 *  - specific_user → `{ user_id: string }`
 */
export interface ApproverConfig {
  permission_key?: string;
  role_id?: string;
  user_id?: string;
}

export interface ApprovalWorkflowStep {
  action_required: "approve" | "review" | "acknowledge";
  approver_config: ApproverConfig;
  approver_type: ApproverType;
  can_skip: boolean;
  created_at?: string;
  id: string;
  step_name: string;
  step_order: number;
  workflow_id: string;
}

export interface ApprovalWorkflow {
  created_at: string;
  created_by?: string | null;
  description?: string | null;
  entity_type: string;
  id: string;
  is_active: boolean;
  name: string;
  steps?: ApprovalWorkflowStep[];
  total_steps: number;
  updated_at: string;
}

// --- Overtime Requests ---
export type OtRequestStatus = "draft" | "pending_approval" | "approved" | "partially_accepted" | "active" | "completed" | "cancelled" | "rejected";
export type ConsentStatus = "pending" | "accepted" | "declined";
