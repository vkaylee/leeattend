
export const MOCK_ORGANIZATION = {
  name: "Mock Enterprise",
  timezone: "Asia/Ho_Chi_Minh",
  rounding_policy: "nearest",
  work_unit_minutes: 30,
  early_arrival_strategy: "block" as const,
  created_at: "2026-01-01T00:00:00Z",
  date_format: "DD/MM/YYYY",
  default_language: "vi",
  theme_primary_color: "#7c3aed",
  time_format: "24h",
};

export const MOCK_USER = {
  id: "user-mock-1",
  email: "test@example.com",
  full_name: "Mock Admin",
  avatar_url: null,
  preferred_language: "vi",
};

export const MOCK_EMPLOYEES = [
  {
    id: "emp-mock-1",
    first_name: "John",
    last_name: "Doe",
    full_name: "John Doe",
    employee_id_number: "EMP001",
    department_id: "dept-1",
    department_name: "Kỹ thuật",
    status: "active",
    hire_date: "2026-01-01",
    original_hire_date: "2026-01-01",
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z"
  },
  {
    id: "emp-mock-2",
    first_name: "Jane",
    last_name: "Smith",
    full_name: "Jane Smith",
    employee_id_number: "EMP002",
    department_id: "dept-1",
    department_name: "Kỹ thuật",
    status: "active",
    hire_date: "2026-01-01",
    original_hire_date: "2026-01-01",
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z"
  }
];

export const MOCK_DEPARTMENTS = [
  {
    id: "dept-1",
    name: "Kỹ thuật",
    code: "TECH",
    active_employee_count: 2,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z"
  }
];
