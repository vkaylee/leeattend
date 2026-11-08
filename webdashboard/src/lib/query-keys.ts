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
