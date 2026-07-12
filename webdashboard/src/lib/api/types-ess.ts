import { components } from "@/types/api-generated";

// --- Employee Self-Service (ESS) Types ---
export type MyProfileResponse = Omit<components["schemas"]["MyProfileResponse"], "profile"> & {
  profile?: components["schemas"]["EmployeeProfile"] | null;
};

export interface UpdateProfileRequest {
  address?: string;
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

export interface ApprovalWorkflowStep {
  action_required: "approve" | "review" | "acknowledge";
  approver_config: Record<string, unknown>;
  approver_type: ApproverType;
  can_skip: boolean;
  created_at: string;
  id: string;
  step_name: string;
  step_order: number;
  workflow_id: string;
}

export type ApprovalWorkflow = components["schemas"]["ApprovalWorkflow"] & {
  steps?: ApprovalWorkflowStep[];
};

// --- Overtime Requests ---
export type OtRequestStatus = "draft" | "pending_approval" | "approved" | "partially_accepted" | "active" | "completed" | "cancelled" | "rejected";
export type ConsentStatus = "pending" | "accepted" | "declined";
