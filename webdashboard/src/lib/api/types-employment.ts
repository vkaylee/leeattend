import type { components } from "@/types/api-generated";

export interface EmploymentPeriod {
  created_at: string;
  employee_id: string;
  hire_date: string;
  id: string;
  is_rehire: boolean;
  termination_date?: string;
  updated_at: string;
}

export const AssignmentStatus = {
  Scheduled: "scheduled",
  Active: "active",
  Completed: "completed",
  Cancelled: "cancelled",
} as const;
export type AssignmentStatus = typeof AssignmentStatus[keyof typeof AssignmentStatus];

export type EmployeeAssignment = Omit<components["schemas"]["EmployeeAssignment"], "status"> & { status: AssignmentStatus };
export type UpdateEmployeeAssignmentRequest = components["schemas"]["UpdateEmployeeAssignmentRequest"] & { host_location_id?: string | null };

export type EmployeeSchedule = Omit<components["schemas"]["EmployeeSchedule"], "employee_id_number"> & {
  employee_id_number?: string;
  employee_id?: string;
  department_name?: string | null;
};

export type WorkingPattern = Omit<components["schemas"]["WorkingPattern"], "description"> & {
  description?: string;
};

export interface UpdateWorkingPatternRequest {
  cycle_length_days?: number;
  description?: string;
  details: components["schemas"]["WorkingPatternDetailRequest"][];
  name: string;
  pattern_type?: string;
}

export type WorkingCondition = Omit<components["schemas"]["WorkingCondition"], "type_"> & { type: string };

export interface EmploymentPhaseRuleConfig {
  duration_days?: number;
}

export type EmploymentPhaseDefinition = Omit<components["schemas"]["EmploymentPhaseDefinition"], "rule_config"> & {
  rule_config?: EmploymentPhaseRuleConfig;
};

export type EmployeePhase = Omit<components["schemas"]["EmployeePhase"], "rule_config"> & {
  rule_config?: EmploymentPhaseRuleConfig;
};
