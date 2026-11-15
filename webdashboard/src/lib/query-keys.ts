function normalizeObject(value: Record<string, unknown>): Record<string, unknown> {
  const normalized: Record<string, unknown> = {};
  const keys = Object.keys(value).filter((key) => value[key] !== undefined).sort();

  for (const key of keys) {
    normalized[key] = normalizeValue(value[key]);
  }

  return normalized;
}

function normalizeValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(normalizeValue);
  }

  if (value && typeof value === "object") {
    return normalizeObject(value as Record<string, unknown>);
  }

  return value;
}

function normalizeFilters(
  filters: Readonly<Record<string, unknown>> | undefined,
): Readonly<Record<string, unknown>> {
  return normalizeObject({ ...(filters ?? {}) });
}

export type DepartmentFilters = Readonly<{
  managerId?: string;
  search?: string;
}>;

export const departmentKeys = {
  all: () => ["departments"] as const,
  list: (filters?: DepartmentFilters) =>
    [...departmentKeys.all(), "list", normalizeFilters(filters)] as const,
  detail: (departmentId: string) =>
    [...departmentKeys.all(), "detail", departmentId] as const,
  lookup: (filters?: DepartmentFilters) =>
    [...departmentKeys.all(), "lookup", normalizeFilters(filters)] as const,
};

export type EmployeeFilters = Readonly<{
  customFieldFilters?: Readonly<Record<string, string>>;
  departmentIds?: readonly string[];
  emptyQueryBehavior?: "all" | "prompt-to-type";
  permission?: string;
  search?: string;
  sortDir?: string;
  sortKey?: string | null;
  status?: string;
}>;

function normalizeEmployeeFilters(filters?: EmployeeFilters) {
  return normalizeFilters(
    filters
      ? {
          ...filters,
          departmentIds: filters.departmentIds
            ? [...filters.departmentIds].sort()
            : undefined,
        }
      : undefined,
  );
}

export const employeeKeys = {
  all: () => ["employees"] as const,
  list: (filters?: EmployeeFilters) =>
    [...employeeKeys.all(), "list", normalizeEmployeeFilters(filters)] as const,
  search: (filters?: EmployeeFilters) =>
    [...employeeKeys.all(), "search", normalizeEmployeeFilters(filters)] as const,
  detail: (employeeId: string) =>
    [...employeeKeys.all(), "detail", employeeId] as const,
  lookup: (filters?: EmployeeFilters) =>
    [...employeeKeys.all(), "lookup", normalizeEmployeeFilters(filters)] as const,
};

export function createQueryKey<T extends readonly unknown[]>(...parts: T): T {
  return parts;
}

export const payrollKeys = {
  all: () => ["payroll"] as const,
  calculationRuns: (employmentId?: string, year?: number, month?: number) =>
    ["payroll-calculation-runs", employmentId, year, month] as const,
  settings: () => ["payroll-settings"] as const,
  settingsHistory: () => ["payroll-settings-history"] as const,
  workPolicyGroups: () => ["payroll-work-policy-groups"] as const,
  workPolicyHistory: (groupId?: string) => ["payroll-work-policy-history", groupId] as const,
  workPolicyMemberships: (groupId?: string) => ["payroll-work-policy-memberships", groupId] as const,
  workPolicyResolved: (employmentId?: string, asOf?: string) => ["payroll-work-policy-resolved", employmentId, asOf] as const,
  rules: () => ["payroll-rules"] as const,
  ruleTiers: (ruleId?: string) => ["payroll-rule-tiers", ruleId] as const,
  ruleAssignments: (ruleId?: string) => ["payroll-rule-assignments", ruleId] as const,
  formulaVersions: (ruleId?: string) => ["payroll-formula-versions", ruleId] as const,
  salaryComponents: () => ["salary-components"] as const,
  employeeSalaryComponents: (employmentId?: string) => ["employee-salary-components", employmentId] as const,
  employeePayrollConfigs: (employeeId?: string) => ["employee-payroll-configs", employeeId] as const,
  overtimePolicies: (policyId?: string) => policyId ? (["overtime-policy", policyId] as const) : (["overtime-policies"] as const),
  overtimePolicyAssignment: () => ["overtime-policy-assignment"] as const,
  incentivePolicies: () => ["incentive-policies"] as const,
  payslips: (employeeId?: string) => employeeId ? (["payslips", employeeId] as const) : (["payslips"] as const),
  organizationPayslips: (year?: number, month?: number) => ["organization-payslips", year, month] as const,
};

export const shiftKeys = {
  all: () => ["shifts"] as const,
  catalogue: () => ["shifts-catalogue"] as const,
  versions: (shiftId?: string) => ["shift-versions", shiftId] as const,
  revisions: (shiftId?: string) => ["shift-revisions", shiftId] as const,
  schedules: (startDate?: string, employmentId?: string) =>
    ["schedules", startDate, employmentId] as const,
  monthlySchedules: (monthStart?: string, employmentIds?: readonly string[]) =>
    ["schedules", monthStart, "monthly", employmentIds] as const,
  assignmentHistory: () => ["assignment_history"] as const,
  patterns: () => ["working-patterns"] as const,
  patternDetails: (patternId?: string) => ["pattern-details", patternId] as const,
  patternPreview: (patternId?: string) => ["pattern-preview", patternId] as const,
  swaps: (tab?: string) => tab ? (["shift-swaps", tab] as const) : (["shift-swaps"] as const),
  swapStats: () => ["shift-swap-stats"] as const,
  employeeRosters: () => ["employee-rosters"] as const,
};

export const attendanceKeys = {
  all: () => ["attendance"] as const,
  report: () => ["attendance-report"] as const,
  reportDelegated: () => ["attendance-report-delegated"] as const,
  rawLogs: (employeeId?: string, dateRange?: unknown) =>
    ["attendance-raw-logs", employeeId, dateRange] as const,
  stagingLogs: () => ["staging-logs"] as const,
  stagingLogsCount: () => ["staging-logs-count"] as const,
  adjustments: (tab?: string, employeeId?: string, statusFilter?: string) =>
    ["attendance-adjustments", tab, employeeId, statusFilter] as const,
  pendingAdjustments: () => ["adjustments", "pending"] as const,
  rules: () => ["attendance-rules"] as const,
  timeHistory: () => ["time-history"] as const,
  unrecordedShifts: () => ["unrecorded-shifts"] as const,
};

export const leaveKeys = {
  all: () => ["leaves"] as const,
  requests: (tab?: string) => tab ? (["leave-requests", tab] as const) : (["leave-requests"] as const),
  balances: () => ["leave-balances"] as const,
  types: () => ["leave-types"] as const,
  policies: () => ["leave-policies"] as const,
  history: (employeeId?: string) => ["leave-history", employeeId] as const,
};

export const reportKeys = {
  all: () => ["reports"] as const,
  daily: (filters?: unknown) => ["reports", "daily", filters] as const,
  monthly: (filters?: unknown) => ["reports", "monthly", filters] as const,
  stats: (filters?: unknown) => ["reports", "stats", filters] as const,
  metadata: () => ["reports", "metadata"] as const,
};

export const approvalKeys = {
  all: () => ["approvals"] as const,
  inbox: (filters?: unknown) => ["approvals", "inbox", filters] as const,
  overtime: (filters?: unknown) => ["approvals", "overtime", filters] as const,
  behalf: (filters?: unknown) => ["approvals", "behalf", filters] as const,
};

export const settingsKeys = {
  all: () => ["settings"] as const,
  smtp: () => ["smtp-config"] as const,
  general: () => ["general-settings"] as const,
  holidays: () => ["holidays"] as const,
  conditionTypes: () => ["condition-types"] as const,
  econtracts: () => ["econtracts-settings"] as const,
  employers: () => ["employers"] as const,
  locations: () => ["locations"] as const,
  auditLogs: (filters?: unknown) => ["audit-logs", filters] as const,
  customFields: () => ["custom-fields"] as const,
};
