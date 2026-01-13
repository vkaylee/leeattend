// Trong môi trường Tauri hoặc Production build, chúng ta cần một địa chỉ tuyệt đối nếu không qua proxy.
// Vite sẽ thay thế import.meta.env.VITE_API_URL lúc build.
const BASE_URL_FROM_ENV = import.meta.env.VITE_API_URL?.replace(/\/$/, "");

// Nếu đang chạy web dev (có proxy) thì dùng "/api", nếu build tauri thì dùng URL tuyệt đối.
const isTauri = typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__;

// LOGIC CẢI TIẾN:
// 1. Web Dev: Luôn dùng "/api" để tận dụng Vite Proxy (nơi đã xử lý dynamic port từ .api_port).
//    Tránh lấy trực tiếp BASE_URL_FROM_ENV vì nó thường hardcode port 3000 gây lỗi.
// 2. Tauri: Cần URL tuyệt đối, ưu tiên ENV -> mặc định localhost:3000.
// 3. Web Prod: Dùng ENV (nếu API khác domain) hoặc fallback về "/api".
const API_BASE_URL = isTauri 
  ? (BASE_URL_FROM_ENV || "http://localhost:3000")
  : (import.meta.env.DEV ? "/api" : (BASE_URL_FROM_ENV || "/api"));

export const WS_BASE_URL = BASE_URL_FROM_ENV
  ? `${BASE_URL_FROM_ENV.replace(/^http/, "ws")}/ws`
  : `${window.location.protocol === "https:" ? "wss:" : "ws:"}//${isTauri ? "localhost:3000" : window.location.host}/ws`;

export interface DeviceStatus {
  device_id: string;
  status: "online" | "offline";
  last_sync?: string;
  error?: string;
}

export interface AgentStatus {
  agent_id: string;
  last_seen: string;
  is_online: boolean;
  devices: DeviceStatus[];
}

export interface Device {
  id: string;
  agent_id: string;
  name: string;
  ip: string;
  port: number;
  is_enabled: boolean;
  last_status: string;
  created_at: string;
  workspace_id: string;
}

export interface AuthorizedAgent {
  id: string;
  name: string;
  token: string;
  is_trusted: boolean;
  last_ip?: string;
  last_seen_at: string;
  created_at: string;
  is_active: boolean;
  workspace_id: string;
}

export interface User {
  id: string;
  email: string;
  created_at: string;
  full_name?: string;
  avatar_url?: string;
  bio?: string;
}

export interface Workspace {
  id: string;
  name: string;
  work_unit_minutes: number;
  rounding_policy: string;
  date_format: string;
  time_format: string;
  created_at: string;
}

export interface UpdateUserRequest {
  full_name?: string;
  avatar_url?: string;
  bio?: string;
}

export interface ChangePasswordRequest {
  old_password: string;
  new_password: string;
}

export interface WorkspaceMember {
  workspace_id: string;
  user_id: string;
  role: "owner" | "admin" | "member";
  email: string;
  full_name?: string;
  avatar_url?: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
}

// --- Timekeeping Models ---

export interface Employee {
  id: string;
  workspace_id: string;
  user_id?: string;
  first_name: string;
  last_name: string;
  employee_id_number: string;
  hire_date: string;
  termination_date?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface TimeEntry {
  id: string;
  employee_id: string;
  punch_in_time: string;
  punch_out_time?: string;
  duration_minutes?: number;
  is_break_overtime: boolean;
  entry_type: string;
  note?: string;
  created_at: string;
  updated_at: string;
}

export interface LeaveRequest {
  id: string;
  employee_id: string;
  leave_type: string;
  start_date: string;
  end_date: string;
  reason?: string;
  status: string;
  requested_by_user_id?: string;
  approved_by_user_id?: string;
  created_at: string;
  updated_at: string;
}

export interface EmployeeLeaveBalance {
  leave_type: string;
  entitlement: number;
  carried_over: number;
  used: number;
  remaining: number;
}

export interface AttendanceAdjustmentRequest {
  id: string;
  employee_id: string;
  time_entry_id?: string;
  requested_punch_in_time: string;
  requested_punch_out_time?: string;
  is_break_overtime: boolean;
  reason?: string;
  status: string;
  requested_by_user_id?: string;
  approved_by_user_id?: string;
  created_at: string;
  updated_at: string;
}

export interface AttendanceSummary {
  employee_id: string;
  employee_name: string;
  total_hours: number;
  total_leave_days: number;
}

export interface DailyAttendanceReport {
  date: string;
  employee_id: string;
  employee_name: string;
  scheduled_start: string | null;
  scheduled_end: string | null;
  break_duration_minutes: number | null;
  break_start: string | null;
  break_end: string | null;
  is_break_auto_deduct: boolean | null;
  actual_punch_in: string | null;
  actual_punch_out: string | null;
  total_minutes: number;
  status: string;
}

// --- Payroll and Shift Models ---

export interface Shift {
  id: string;
  workspace_id: string;
  name: string;
  start_time: string;
  end_time: string;
  break_duration_minutes: number;
  break_start?: string;
  break_end?: string;
  is_break_auto_deduct?: boolean;
  created_at: string;
  updated_at: string;
}

export interface SalaryHistory {
  id: string;
  employee_id: string;
  base_salary: string;
  salary_type: "hourly" | "monthly";
  currency: string;
  effective_date: string;
  created_at: string;
  updated_at: string;
}

export interface Payslip {
  id: string;
  employee_id: string;
  period_start: string;
  period_end: string;
  base_pay_amount: string;
  overtime_pay_amount: string;
  deductions_amount: string;
  net_pay_amount: string;
  status: "draft" | "published" | "paid";
  created_at: string;
  updated_at: string;
}

// --- Department Models ---

export interface Department {
  id: string;
  workspace_id: string;
  name: string;
  description?: string;
  manager_id?: string;
  created_at: string;
  updated_at: string;
}

export interface EmployeeDepartmentHistory {
  id: string;
  employee_id: string;
  department_id: string;
  effective_date: string;
  notes?: string;
  created_at: string;
}

export interface ShiftRoster {
  id: string;
  employee_id: string;
  shift_id: string | null;
  date: string;
  status: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

// --- Auth Management ---

export function getAccessToken(): string | null {
  return localStorage.getItem("access_token");
}

export function setAccessToken(token: string) {
  localStorage.setItem("access_token", token);
}

export function getRefreshToken(): string | null {
  return localStorage.getItem("refresh_token");
}

export function setRefreshToken(token: string) {
  localStorage.setItem("refresh_token", token);
}

export function getCurrentWorkspaceId(): string | null {
  return localStorage.getItem("active_workspace_id");
}

export function setCurrentWorkspaceId(id: string) {
  localStorage.setItem("active_workspace_id", id);
}

export function getCurrentUserId(): string | null {
  return localStorage.getItem("active_user_id");
}

export const setCurrentUserId = (id: string) => {
  localStorage.setItem("active_user_id", id);
};

export const clearAuthSession = () => {
  localStorage.removeItem("active_user_id");
  localStorage.removeItem("active_workspace_id");
  localStorage.removeItem("access_token");
  localStorage.removeItem("refresh_token");
};

function getHeaders() {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const wsId = getCurrentWorkspaceId();
  if (wsId) {
    headers["X-Workspace-Id"] = wsId;
  }
  const token = getAccessToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  } else {
    // Fallback for dev mode
    const userId = getCurrentUserId();
    if (userId) headers["X-User-Id"] = userId;
  }
  return headers;
}

/**
 * Wrapper for fetch to handle 401 and refresh token
 */
async function apiFetch(
  url: string,
  options: RequestInit = {},
): Promise<Response> {
  const headers = { ...getHeaders(), ...options.headers };
  
  if (options.body instanceof FormData) {
    delete (headers as any)["Content-Type"];
  }

  const response = await fetch(url, { ...options, headers });

  if (response.status === 401 && getRefreshToken()) {
    console.warn("Access token expired, attempting refresh...");
    try {
      const refreshRes = await fetch(`${API_BASE_URL}/auth/refresh`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: getRefreshToken() }),
      });

      if (refreshRes.ok) {
        const data = await refreshRes.json();
        setAccessToken(data.access_token);
        setRefreshToken(data.refresh_token);

        // Retry original request with new token
        const retryHeaders = { ...getHeaders(), ...options.headers };
        return fetch(url, { ...options, headers: retryHeaders });
      } else {
        console.error("Refresh token invalid, logging out");
        clearAuthSession();
        window.location.href = "/login";
      }
    } catch (e) {
      clearAuthSession();
      window.location.href = "/login";
    }
  }

  return response;
}

// --- Health ---
export async function fetchHealth(): Promise<{ status: string }> {
  const response = await apiFetch(`${API_BASE_URL}/health`);
  return response.json();
}

// --- Auth ---
export async function login(username: string, password: string): Promise<AuthResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email: username, password }),
  });
  if (!response.ok) throw new Error("Login failed");
  const data = await response.json();
  setAccessToken(data.access_token);
  setRefreshToken(data.refresh_token);
  setCurrentUserId(data.user.id);
  return data;
}

export async function logout(): Promise<void> {
  await apiFetch(`${API_BASE_URL}/auth/logout`, { method: "POST" });
  clearAuthSession();
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Registration failed");
  }
  return response.json();
}

export async function changePassword(
  data: ChangePasswordRequest,
): Promise<string> {
  const response = await apiFetch(`${API_BASE_URL}/auth/change-password`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to change password");
  }
  return response.text();
}

// --- Agents & Devices ---
export async function fetchAgents(): Promise<AgentStatus[]> {
  const response = await apiFetch(`${API_BASE_URL}/agents`);
  if (!response.ok) throw new Error("Failed to fetch agents");
  return response.json();
}

export async function sendAgentCommand(agentId: string, command: any): Promise<any> {
  const response = await apiFetch(`${API_BASE_URL}/agent/${agentId}/command`, {
    method: "POST",
    body: JSON.stringify(command),
  });
  return response.json();
}

export async function fetchDevices(): Promise<Device[]> {
  const response = await apiFetch(`${API_BASE_URL}/devices`);
  if (!response.ok) throw new Error("Failed to fetch devices");
  return response.json();
}

export async function createDevice(
  device: Partial<Device>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL}/devices`, {
    method: "POST",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to create device");
  return response.json();
}

export async function updateDevice(
  id: string,
  device: Partial<Device>,
): Promise<Device> {
  const response = await apiFetch(`${API_BASE_URL}/devices/${id}`, {
    method: "PUT",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to update device");
  return response.json();
}

export async function deleteDevice(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/devices/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete device");
}

export async function fetchAuthorizedAgents(
  includeInactive: boolean = false,
): Promise<AuthorizedAgent[]> {
  const url = includeInactive
    ? `${API_BASE_URL}/authorized-agents?include_inactive=true`
    : `${API_BASE_URL}/authorized-agents`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch authorized agents");
  return response.json();
}

export async function fetchAuthorizedAgentById(id: string): Promise<AuthorizedAgent> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`);
  if (!response.ok) throw new Error("Failed to fetch authorized agent");
  return response.json();
}

export async function createAuthorizedAgent(
  agent: Partial<AuthorizedAgent>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents`, {
    method: "POST",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to create authorized agent");
  return response.json();
}

export async function updateAgentTrust(
  id: string,
  is_trusted: boolean,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/authorized-agents/${id}/trust`,
    {
      method: "PUT",
      body: JSON.stringify({ is_trusted }),
    },
  );
  if (!response.ok) throw new Error("Failed to update agent trust");
}

export async function updateAuthorizedAgent(
  id: string,
  agent: Partial<AuthorizedAgent>,
): Promise<AuthorizedAgent> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: "PATCH",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to update authorized agent");
  return response.json();
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete authorized agent");
}

// --- User Profile & Users ---
export async function fetchMe(): Promise<User> {
  const response = await apiFetch(`${API_BASE_URL}/me`);
  if (!response.ok) throw new Error("Failed to fetch profile");
  return response.json();
}

export async function updateMe(data: Partial<User>): Promise<User> {
  const response = await apiFetch(`${API_BASE_URL}/me`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update profile");
  return response.json();
}

export async function fetchUsers(): Promise<User[]> {
  const response = await apiFetch(`${API_BASE_URL}/users`);
  if (!response.ok) throw new Error("Failed to fetch users");
  return response.json();
}

// --- Workspace Management ---
export async function fetchWorkspaces(): Promise<Workspace[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces`);
  if (!response.ok) throw new Error("Failed to fetch workspaces");
  return response.json();
}

export async function createWorkspace(data: { name: string }): Promise<Workspace> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create workspace");
  return response.json();
}

export async function fetchWorkspace(wsId: string): Promise<Workspace> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}`);
  if (!response.ok) throw new Error("Failed to fetch workspace");
  return response.json();
}

export async function updateWorkspace(
  wsId: string,
  data: Partial<Workspace>,
): Promise<Workspace> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update workspace");
  return response.json();
}

export async function fetchWorkspaceMembers(
  wsId: string,
): Promise<WorkspaceMember[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/members`);
  if (!response.ok) throw new Error("Failed to fetch members");
  return response.json();
}

export async function addWorkspaceMember(
  wsId: string,
  email: string,
  role: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members`,
    {
      method: "POST",
      body: JSON.stringify({ email, role }),
    },
  );
  if (!response.ok) throw new Error("Failed to add member");
}

export async function removeWorkspaceMember(
  wsId: string,
  userId: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}`,
    {
      method: "DELETE",
    },
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to remove member");
  }
}

export async function updateWorkspaceMemberRole(
  wsId: string,
  userId: string,
  role: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}`,
    {
      method: "PATCH",
      body: JSON.stringify({ role }),
    },
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to update member role");
  }
}

export interface MemberRolesResponse {
  primary: string;
  extra: MemberExtraRole[];
}

export interface MemberExtraRole {
  id: string;
  workspace_id: string;
  user_id: string;
  role: string;
  assigned_at: string;
  expires_at: string;
}

export async function fetchMemberRoles(
  wsId: string,
  userId: string,
): Promise<MemberRolesResponse> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles`,
  );
  if (!response.ok) throw new Error("Failed to fetch member roles");
  return response.json();
}

export async function assignExtraRole(
  wsId: string,
  userId: string,
  role: string,
  expiresAt: string,
): Promise<MemberExtraRole> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles`,
    {
      method: "POST",
      body: JSON.stringify({ role, expires_at: expiresAt }),
    },
  );
  if (!response.ok) throw new Error("Failed to assign role");
  return response.json();
}

export async function revokeExtraRole(
  wsId: string,
  userId: string,
  roleId: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles/${roleId}`,
    {
      method: "DELETE",
    },
  );
  if (!response.ok) throw new Error("Failed to revoke role");
}

// --- Employee Management ---
export async function fetchEmployees(wsId: string): Promise<Employee[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees`);
  if (!response.ok) throw new Error("Failed to fetch employees");
  return response.json();
}

export async function fetchEmployee(wsId: string, id: string): Promise<Employee> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}`);
  if (!response.ok) throw new Error("Failed to fetch employee");
  return response.json();
}

export async function createEmployee(
  wsId: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees`, {
    method: "POST",
    body: JSON.stringify(employee),
  });
  if (!response.ok) throw new Error("Failed to create employee");
  return response.json();
}

export async function updateEmployee(
  wsId: string,
  id: string,
  employee: Partial<Employee>,
): Promise<Employee> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}`, {
    method: "PUT",
    body: JSON.stringify(employee),
  });
  if (!response.ok) throw new Error("Failed to update employee");
  return response.json();
}

export async function deleteEmployee(wsId: string, id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete employee");
}

// --- Time Tracking ---
export async function punchIn(
  wsId: string,
  data: { employee_id: string; entry_type?: string; note?: string },
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/punch-in`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to punch in");
  return response.json();
}

export async function punchOut(
  wsId: string,
  employeeId: string,
  data: { note?: string },
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/punch-out/${employeeId}`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to punch out");
  return response.json();
}

export async function fetchTimeHistory(
  wsId: string,
  employeeId: string,
  start: string,
  end: string,
): Promise<TimeEntry[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/time/history/${employeeId}?start=${start}&end=${end}`,
  );
  if (!response.ok) throw new Error("Failed to fetch time history");
  return response.json();
}

export async function updateTimeEntry(
  wsId: string,
  id: string,
  data: Partial<TimeEntry>,
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/entries/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update time entry");
  return response.json();
}

export async function deleteTimeEntry(wsId: string, id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/entries/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete time entry");
}

// --- Leave Management ---
export async function submitLeaveRequest(
  wsId: string,
  data: { employee_id: string; leave_type: string; start_date: string; end_date: string; reason?: string },
): Promise<LeaveRequest> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/submit`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to submit leave request");
  return response.json();
}

export async function fetchWorkspaceLeaveRequests(
  wsId: string,
  status?: string,
): Promise<LeaveRequest[]> {
  const url = status
    ? `${API_BASE_URL}/workspaces/${wsId}/leave/requests?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/leave/requests`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch leave requests");
  return response.json();
}

export async function fetchMyLeaveRequests(
  employeeId: string
): Promise<LeaveRequest[]> {
  const response = await apiFetch(`${API_BASE_URL}/employees/${employeeId}/leave/requests`);
  if (!response.ok) throw new Error("Failed to fetch my leave requests");
  return response.json();
}

export async function approveLeaveRequest(
  wsId: string,
  requestId: string,
  data: { status: string },
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/requests/${requestId}/approve`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to approve leave request");
}

export async function fetchEmployeeLeaveBalances(
  wsId: string,
  employeeId: string,
): Promise<EmployeeLeaveBalance[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/balances`);
  if (!response.ok) throw new Error("Failed to fetch leave balances");
  return response.json();
}

// --- Leave Policies ---
export interface LeavePolicy {
  id: string;
  workspace_id: string;
  name: string;
  leave_type: string;
  annual_entitlement: number;
  accrual_strategy: "monthly" | "lump_sum";
  carry_over_limit: number;
  created_at: string;
}

export async function fetchLeavePolicies(wsId: string): Promise<LeavePolicy[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies`);
  if (!response.ok) throw new Error("Failed to fetch leave policies");
  return response.json();
}

export async function createLeavePolicy(
  wsId: string,
  data: Partial<LeavePolicy>
): Promise<LeavePolicy> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create leave policy");
  return response.json();
}

export async function assignLeavePolicy(
  wsId: string,
  data: { employee_id: string; leave_policy_id: string; effective_date: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/policies/assign`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to assign leave policy");
}

export async function triggerAccrualRun(
  wsId: string,
  data: { date: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/accrual/run`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to trigger accrual run");
}

// --- Attendance Adjustments ---
export async function submitAttendanceAdjustment(
  wsId: string,
  data: {
    employee_id: string;
    time_entry_id?: string;
    requested_punch_in_time: string;
    requested_punch_out_time?: string;
    reason?: string;
  }
): Promise<AttendanceAdjustmentRequest> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to submit attendance adjustment");
  return response.json();
}

export interface AttendanceImportJob {
  id: string;
  workspace_id: string;
  user_id: string;
  status: "processing" | "completed" | "failed";
  total_rows: number;
  processed_rows: number;
  error_log?: string;
  created_at: string;
  updated_at: string;
}

export async function uploadAttendanceExcel(
  wsId: string,
  file: File
): Promise<{ job_id: string }> {
  return uploadAttendance(wsId, file, "/attendance/upload");
}

export async function uploadAttendance(
  wsId: string,
  file: File,
  endpoint: string
): Promise<{ job_id: string }> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}${endpoint}`, {
    method: "POST",
    body: formData,
  });
  if (!response.ok) throw new Error("Failed to upload attendance");
  return response.json();
}

export async function fetchLatestImportJob(
  wsId: string,
  targetType?: string
): Promise<AttendanceImportJob | null> {
  const url = targetType 
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest?target_type=${targetType}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch latest import job");
  return response.json();
}

export async function fetchImportJobStatus(
  wsId: string,
  jobId: string
): Promise<AttendanceImportJob> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/${jobId}`);
  if (!response.ok) throw new Error("Failed to fetch import job status");
  return response.json();
}

export async function fetchWorkspaceAttendanceAdjustments(
  wsId: string,
  status?: string
): Promise<AttendanceAdjustmentRequest[]> {
  const url = status 
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch attendance adjustments");
  return response.json();
}

export async function fetchMyAttendanceAdjustments(
  employeeId: string
): Promise<AttendanceAdjustmentRequest[]> {
  const response = await apiFetch(`${API_BASE_URL}/employees/${employeeId}/attendance/adjustments`);
  if (!response.ok) throw new Error("Failed to fetch my attendance adjustments");
  return response.json();
}

export async function approveAttendanceAdjustment(
  wsId: string,
  requestId: string,
  data: { status: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments/${requestId}/approve`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to approve attendance adjustment");
}

export async function fetchAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: TimeEntry[]; total: number; page: number; limit: number }> {
  const searchParams = new URLSearchParams();
  if (params.employee_id) searchParams.append("employee_id", params.employee_id);
  if (params.from) searchParams.append("from", params.from);
  if (params.to) searchParams.append("to", params.to);
  if (params.page) searchParams.append("page", params.page.toString());
  if (params.limit) searchParams.append("limit", params.limit.toString());
  
  const wsId = getCurrentWorkspaceId();
  const url = wsId 
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/logs?${searchParams.toString()}`
    : `${API_BASE_URL}/attendance/logs?${searchParams.toString()}`;

  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch attendance logs");
  return response.json();
}

export async function fetchAttendanceReport(
  wsId: string,
  start: string,
  end: string,
  type: "summary" | "daily" = "summary",
): Promise<any> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/reports/attendance?start_date=${start}&end_date=${end}&type=${type}`,
  );
  if (!response.ok) throw new Error("Failed to fetch attendance report");
  return response.json();
}

// --- Payroll API ---
export async function fetchSalaryHistory(wsId: string, employeeId: string): Promise<SalaryHistory[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/salary`);
  if (!response.ok) throw new Error("Failed to fetch salary history");
  return response.json();
}

export async function addSalaryHistory(
  wsId: string, 
  employeeId: string, 
  data: Partial<SalaryHistory>
): Promise<SalaryHistory> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/salary`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to add salary history");
  return response.json();
}

export async function calculatePayroll(
  wsId: string,
  employeeId: string,
  year: number,
  month: number,
): Promise<Payslip> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/calculate-payroll?year=${year}&month=${month}`,
    { method: "POST" }
  );
  if (!response.ok) throw new Error("Failed to calculate payroll");
  return response.json();
}

export async function fetchEmployeePayslips(wsId: string, employeeId: string): Promise<Payslip[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/payslips`);
  if (!response.ok) throw new Error("Failed to fetch payslips");
  return response.json();
}

// --- Shift API ---
export async function fetchShifts(wsId: string): Promise<Shift[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts`);
  if (!response.ok) throw new Error("Failed to fetch shifts");
  return response.json();
}

export async function createShift(wsId: string, data: Partial<Shift>): Promise<Shift> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create shift");
  return response.json();
}

export async function updateShift(
  wsId: string, 
  shiftId: string, 
  data: Partial<Shift> & { is_correction?: boolean, effective_date?: string }
): Promise<Shift> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts/${shiftId}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update shift");
  return response.json();
}

export async function assignShift(
  wsId: string,
  data: { employee_id: string; shift_id: string; start_date: string; end_date?: string | null }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/shifts/assign`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to assign shift");
}

// --- Department API ---
export async function fetchDepartments(wsId: string): Promise<Department[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments`);
  if (!response.ok) throw new Error("Failed to fetch departments");
  return response.json();
}

export async function createDepartment(wsId: string, data: Partial<Department>): Promise<Department> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/departments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to create department");
  return response.json();
}

export async function transferEmployee(
  wsId: string,
  employeeId: string,
  data: { department_id: string; effective_date: string; notes?: string }
): Promise<EmployeeDepartmentHistory> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/departments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to transfer employee");
  return response.json();
}

export async function fetchEmployeeDepartmentHistory(wsId: string, employeeId: string): Promise<EmployeeDepartmentHistory[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/departments`);
  if (!response.ok) throw new Error("Failed to fetch dept history");
  return response.json();
}

// --- Roster API ---
export interface EffectiveSchedule {
  date: string;
  shift_id: string | null;
  source: "roster" | "fixed" | "department" | "none";
  shift_name: string | null;
  start_time: string | null;
  end_time: string | null;
  break_duration_minutes: number | null;
  break_start: string | null;
  break_end: string | null;
  is_break_auto_deduct: boolean | null;
}

export interface EmployeeSchedule {
  employee_id: string;
  first_name: string;
  last_name: string;
  schedules: EffectiveSchedule[];
}

export interface AssignmentHistory {
  id: string;
  employee_id: string;
  employee_name: string;
  assignment_type: string;
  shift_name: string | null;
  date_range: string;
  created_at: string;
}

export const fetchWorkspaceSchedules = async (wsId: string, startDate: string, endDate: string): Promise<EmployeeSchedule[]> => {
  const params = new URLSearchParams({ start_date: startDate, end_date: endDate });
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/schedules?${params}`);
  if (!response.ok) throw new Error("Failed to fetch schedules");
  return response.json();
};

export const fetchAssignmentHistory = async (wsId: string): Promise<AssignmentHistory[]> => {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/assignment-history`);
  if (!response.ok) throw new Error("Failed to fetch assignment history");
  return response.json();
};

export const fetchEmployeeRosters = async (
  wsId: string, 
  employeeId: string, 
  startDate?: string, 
  endDate?: string
): Promise<ShiftRoster[]> => {
  let url = `${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters`;
  const params = new URLSearchParams();
  if (startDate) params.append("start_date", startDate);
  if (endDate) params.append("end_date", endDate);
  if (params.toString()) url += `?${params.toString()}`;
  
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch rosters");
  return response.json();
}

export async function upsertRoster(wsId: string, employeeId: string, data: any): Promise<ShiftRoster> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to upsert roster");
  return response.json();
}

export async function bulkUpsertRoster(wsId: string, employeeId: string, items: any[]): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters/bulk`, {
    method: "POST",
    body: JSON.stringify({ items }),
  });
  if (!response.ok) throw new Error("Failed to bulk upsert roster");
}

export async function deleteRoster(wsId: string, employeeId: string, date: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/rosters/${date}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete roster");
}

export async function fetchDevSuperUser(): Promise<{
  user_id: string;
  workspace_id: string;
  access_token?: string;
  refresh_token?: string;
}> {
  const response = await fetch(`${API_BASE_URL}/dev/super-user`);
  if (!response.ok) throw new Error("Failed to fetch dev super user");
  return response.json();
}