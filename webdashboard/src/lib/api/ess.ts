/**
 * Employee Self-Service (ESS) API Client
 * Endpoints for employees to manage their own data
 */

import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import {
  MyProfileResponse,
  EmployeeProfile,
  EmployeeEmergencyContact,
  EmployeeDocument,
  EmployeeLeaveBalance,
  LeaveRequest,
  TimeEntry,
  Payslip,
  UpdateProfileRequest,
  CreateEmergencyContactRequest,
} from "./types";

// --- Profile ---

export async function fetchMyProfile(): Promise<MyProfileResponse> {
  const response = await apiFetch(`${API_BASE_URL}/ess/profile`);
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to fetch profile");
  }
  return response.json();
}

export async function updateMyProfile(
  data: UpdateProfileRequest
): Promise<EmployeeProfile> {
  const response = await apiFetch(`${API_BASE_URL}/ess/profile`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to update profile");
  }
  return response.json();
}

// --- Emergency Contacts ---

export async function fetchMyEmergencyContacts(): Promise<
  EmployeeEmergencyContact[]
> {
  const response = await apiFetch(`${API_BASE_URL}/ess/emergency-contacts`);
  if (!response.ok) throw new Error("Failed to fetch emergency contacts");
  return response.json();
}

export async function addEmergencyContact(
  data: CreateEmergencyContactRequest
): Promise<EmployeeEmergencyContact> {
  const response = await apiFetch(`${API_BASE_URL}/ess/emergency-contacts`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to add emergency contact");
  }
  return response.json();
}

export async function deleteEmergencyContact(contactId: string): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/ess/emergency-contacts/${contactId}`,
    {
      method: "DELETE",
    }
  );
  if (!response.ok) throw new Error("Failed to delete emergency contact");
}

// --- Documents ---

export async function fetchMyDocuments(): Promise<EmployeeDocument[]> {
  const response = await apiFetch(`${API_BASE_URL}/ess/documents`);
  if (!response.ok) throw new Error("Failed to fetch documents");
  return response.json();
}

// Document upload will be implemented in Phase 4

// --- Payslips ---

export async function fetchMyPayslips(): Promise<Payslip[]> {
  const response = await apiFetch(`${API_BASE_URL}/ess/payslips`);
  if (!response.ok) throw new Error("Failed to fetch payslips");
  return response.json();
}

// --- Attendance ---

export interface AttendanceParams {
  start_date?: string;
  end_date?: string;
}

export async function fetchMyAttendance(
  params?: AttendanceParams
): Promise<TimeEntry[]> {
  const searchParams = new URLSearchParams();
  if (params?.start_date) searchParams.set("start_date", params.start_date);
  if (params?.end_date) searchParams.set("end_date", params.end_date);

  const query = searchParams.toString();
  const url = `${API_BASE_URL}/ess/attendance${query ? `?${query}` : ""}`;

  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch attendance");
  return response.json();
}

// --- Time Clock ---

export async function fetchMyTimeEntries(
  params?: { start_date?: string; end_date?: string }
): Promise<TimeEntry[]> {
  const searchParams = new URLSearchParams();
  if (params?.start_date) searchParams.set("start_date", params.start_date);
  if (params?.end_date) searchParams.set("end_date", params.end_date);

  const query = searchParams.toString();
  const url = `${API_BASE_URL}/ess/time-entries${query ? `?${query}` : ""}`;

  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch time entries");
  return response.json();
}

export async function punchInMySelf(
  data: { entry_type: "work" | "break"; note?: string }
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/ess/punch-in`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to punch in");
  }
  return response.json();
}

export async function punchOutMySelf(
  data?: { note?: string }
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/ess/punch-out`, {
    method: "POST",
    body: JSON.stringify(data || {}),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to punch out");
  }
  return response.json();
}

// --- Leave ---

export async function fetchEssLeaveBalances(): Promise<EmployeeLeaveBalance[]> {
  const response = await apiFetch(`${API_BASE_URL}/ess/leave/balances`);
  if (!response.ok) throw new Error("Failed to fetch leave balances");
  return response.json();
}

export async function fetchEssLeaveRequests(): Promise<LeaveRequest[]> {
  const response = await apiFetch(`${API_BASE_URL}/ess/leave/requests`);
  if (!response.ok) throw new Error("Failed to fetch leave requests");
  return response.json();
}

// --- OT Claim (ESS Wrapper) ---

import { claimOt, fetchAttendanceSummaries } from "./attendance";
import { AttendanceDailySummary } from "./types";
import { getCurrentWorkspaceId, getCurrentUserId } from "./helpers";

// This requires fetching the employee ID first or having the backend resolve it from token.
// The backend handler for claimOt expects `employeeId` in path.
// For ESS, we need to know our own employee ID.
// Ideally, `fetchMyProfile` returns employee ID.

export async function fetchMyAttendanceSummaries(
  start: string,
  end: string
): Promise<AttendanceDailySummary[]> {
  // We need to get the employee ID.
  // This is a bit tricky if we don"t have it stored.
  // Option 1: Call fetchMyProfile first.
  // Option 2: Backend supports "me" as employee ID? No, current implementation expects UUID.
  
  // Strategy: We will rely on the caller to provide employeeId or fetch it here.
  // But wait, the backend `get_attendance_summaries` logic I just wrote supports `employee_id` query param.
  // If I am not admin, I can only see my own.
  // But I need to pass the workspace ID.
  
  const wsId = getCurrentWorkspaceId();
  if (!wsId) throw new Error("Workspace not found");
  
  // We don"t necessarily need to pass employeeId if the backend resolves it from user_id for non-admins?
  // Checking backend code:
  // "let target_employee_id = if is_admin { query.employee_id } else { ... find my id ... }"
  // So if I don"t pass employee_id, and I am not admin, it defaults to MY id.
  // PERFECT!
  
  return fetchAttendanceSummaries(wsId, start, end);
}

export async function submitMyOtClaim(
  date: string,
  minutes: number,
  reason: string
): Promise<void> {
  const wsId = getCurrentWorkspaceId();
  if (!wsId) throw new Error("Workspace not found");
  
  // We need employee ID for the PATH param.
  // fetchMyProfile cached or stored?
  // Let"s fetch profile quickly.
  const profile = await fetchMyProfile();
  const employeeId = profile.employee.id;
  
  return claimOt(wsId, employeeId, { date, claimed_minutes: minutes, reason });
}
