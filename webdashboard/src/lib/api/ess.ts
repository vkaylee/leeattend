/**
 * Employee Self-Service (ESS) API Client
 * Endpoints for employees to manage their own data
 */

import { claimOt, fetchAttendanceSummaries } from "./attendance";
import { API_BASE_URL } from "./config";
import { apiFetch, getCurrentWorkspaceId } from "./helpers";
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
  CreateEmergencyContactRequest, AttendanceDailySummary 
} from "./types";
import { apiClient } from "./client";

// --- Profile ---

export async function fetchMyProfile(): Promise<MyProfileResponse> {
  const { data: resData } = await apiClient.get<MyProfileResponse>(`/ess/profile`);
    return resData;
}

export async function updateMyProfile(
  data: UpdateProfileRequest
): Promise<EmployeeProfile> {
  const { data: resData } = await apiClient.put<EmployeeProfile>(`/ess/profile`, data);
    return resData;
}

// --- Emergency Contacts ---

export async function fetchMyEmergencyContacts(): Promise<
  EmployeeEmergencyContact[]
> {
  const { data: resData } = await apiClient.get<EmployeeEmergencyContact[]>(`/ess/emergency-contacts`);
    return resData;
}

export async function addEmergencyContact(
  data: CreateEmergencyContactRequest
): Promise<EmployeeEmergencyContact> {
  const { data: resData } = await apiClient.post<EmployeeEmergencyContact>(`/ess/emergency-contacts`, data);
    return resData;
}

export async function deleteEmergencyContact(contactId: string): Promise<void> {
  await apiClient.delete(`/ess/emergency-contacts/${contactId}`);
}

// --- Documents ---

export async function fetchMyDocuments(): Promise<EmployeeDocument[]> {
  const { data: resData } = await apiClient.get<EmployeeDocument[]>(`/ess/documents`);
    return resData;
}

// Document upload will be implemented in Phase 4

// --- Payslips ---

export async function fetchMyPayslips(): Promise<Payslip[]> {
  const { data: resData } = await apiClient.get<Payslip[]>(`/ess/payslips`);
    return resData;
}

// --- Attendance ---

export interface AttendanceParams {
  end_date?: string;
  start_date?: string;
}

export async function fetchMyAttendance(
  params?: AttendanceParams
): Promise<TimeEntry[]> {
  const searchParams = new URLSearchParams();
  if (params?.start_date) searchParams.set("start_date", params.start_date);
  if (params?.end_date) searchParams.set("end_date", params.end_date);

  const query = searchParams.toString();
  const url = `${API_BASE_URL}/ess/attendance${query ? `?${query}` : ""}`;

  const { data: resData } = await apiClient.get<TimeEntry[]>(`/${url}`);
    return resData;
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

  const { data: resData } = await apiClient.get<TimeEntry[]>(`/${url}`);
    return resData;
}

export async function punchMySelf(
  data?: { note?: string }
): Promise<TimeEntry> {
  const { data: resData } = await apiClient.post<TimeEntry>(`/ess/punch`, data ?? {});
    return resData;
}

// --- Leave ---

export async function fetchEssLeaveBalances(): Promise<EmployeeLeaveBalance[]> {
  const { data: resData } = await apiClient.get<EmployeeLeaveBalance[]>(`/ess/leave/balances`);
    return resData;
}

export async function fetchEssLeaveRequests(): Promise<LeaveRequest[]> {
  const { data: resData } = await apiClient.get<LeaveRequest[]>(`/ess/leave/requests`);
    return resData;
}

// --- OT Claim (ESS Wrapper) ---
 

// OT Claim (ESS Wrapper) using attendance helpers

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
