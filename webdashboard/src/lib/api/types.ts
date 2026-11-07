import type { components } from "@/types/api-generated";

// Pure 1:1 schema aliases moved to ./api-generated-reexports.
// Types.ts now only carries overrides (Omit/&), hand-written
// interfaces, and schema gaps (Group B/C).
// Anchor aliases: Group A names referenced by overrides below.
// Kept here (not in the barrel) so they can be used in local scope.
export type Department = components["schemas"]["Department"];
export type Employee = components["schemas"]["Employee"] & {
  employment_status?: string | null;
  hire_date?: string | null;
  job_title?: string | null;
  manager_name?: string | null;
};
export type EmployeeProfile = components["schemas"]["EmployeeProfile"];
export type EmployeeSalaryComponent = components["schemas"]["EmployeeSalaryComponent"];
export type Employment = components["schemas"]["Employment"];
export type IncentiveEvaluationResult = components["schemas"]["IncentiveEvaluationResult"];
export type MemberExtraRole = components["schemas"]["MemberExtraRole"];
export type SalaryComponentWithDefinition = components["schemas"]["SalaryComponentWithDefinition"];
export type Shift = components["schemas"]["Shift"];
export type TimeEntry = components["schemas"]["TimeEntry"];
export type WorkerHeartbeat = components["schemas"]["WorkerHeartbeat"];
export type WorkingPatternDetailRequest = components["schemas"]["WorkingPatternDetailRequest"];
export type Workspace = components["schemas"]["Workspace"];

export * from "./api-generated-reexports";


export interface User {
  avatar_url?: string | null;
  bio?: string | null;
  created_at?: string;
  email: string;
  full_name?: string | null;
  id: string;
  preferred_language?: string | null;
}

export interface WorkspaceMember {
  avatar_url?: string | null;
  email: string;
  extra_role_ids?: string[];
  full_name?: string | null;
  role_id: string;
  user_id: string;
  workspace_id: string;
}

export type WorkspaceRole = Omit<components["schemas"]["WorkspaceRole"], "description"> & {
  description?: string | null;
};

export interface PaginatedResponse<T> {
  data: T[];
  limit: number;
  page: number;
  total: number;
  /** `true` when results are capped by a server-side safety ceiling (e.g. report summary with computed sort). */
  truncated?: boolean;
}

export interface MemberRolesResponse {
  extra: MemberExtraRole[];
  primary: string;
}


// --- Workspace Config (EAV) ---
export interface CreateConfigEntryRequest {
  config_key: string;
  effective_from: string;
  effective_to?: string | null;
  scope_id?: string | null;
  scope_type: "workspace" | "employer" | "department" | "employee";
  value_bool?: boolean | null;
  value_decimal?: number | null;
  value_int?: number | null;
  value_json?: unknown;
  value_text?: string | null;
}

// --- Domain type re-exports ---
export * from "./types-attendance";
export * from "./types-leave";
export * from "./types-payroll";
export * from "./types-monitoring";
export * from "./types-employment";
export * from "./types-work-segments";
export * from "./types-ess";
