export const MOCK_WORKSPACE = {
  id: "ws-mock-123",
  name: "Mock Enterprise",
  timezone: "Asia/Ho_Chi_Minh",
  rounding_policy: "nearest",
  work_unit_minutes: 30,
  early_arrival_strategy: "block" as const,
};

export const MOCK_USER = {
  id: "user-mock-1",
  email: "test@example.com",
  full_name: "Mock Admin",
  avatar_url: null,
  preferred_language: "vi"
};

export const MOCK_EMPLOYEES = [
  { id: "emp-mock-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001", department_id: "dept-1", status: "active" },
  { id: "emp-mock-2", first_name: "Jane", last_name: "Smith", employee_id_number: "EMP002", department_id: "dept-1", status: "active" }
];

export const MOCK_DEPARTMENTS = [
  { id: "dept-1", workspace_id: "ws-mock-123", name: "Kỹ thuật", code: "TECH" }
];
