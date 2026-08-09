export type WorkspaceId = string | null | undefined;

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
  all: (wsId: WorkspaceId) => ["departments", wsId] as const,
  list: (wsId: WorkspaceId, filters?: DepartmentFilters) =>
    [...departmentKeys.all(wsId), "list", normalizeFilters(filters)] as const,
  detail: (wsId: WorkspaceId, departmentId: string) =>
    [...departmentKeys.all(wsId), "detail", departmentId] as const,
  lookup: (wsId: WorkspaceId, filters?: DepartmentFilters) =>
    [...departmentKeys.all(wsId), "lookup", normalizeFilters(filters)] as const,
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
  all: (wsId: WorkspaceId) => ["employees", wsId] as const,
  list: (wsId: WorkspaceId, filters?: EmployeeFilters) =>
    [...employeeKeys.all(wsId), "list", normalizeEmployeeFilters(filters)] as const,
  search: (wsId: WorkspaceId, filters?: EmployeeFilters) =>
    [...employeeKeys.all(wsId), "search", normalizeEmployeeFilters(filters)] as const,
  detail: (wsId: WorkspaceId, employeeId: string) =>
    [...employeeKeys.all(wsId), "detail", employeeId] as const,
  lookup: (wsId: WorkspaceId, filters?: EmployeeFilters) =>
    [...employeeKeys.all(wsId), "lookup", normalizeEmployeeFilters(filters)] as const,
};
