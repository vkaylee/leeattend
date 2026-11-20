/**
 * Employee Self-Service (ESS) API Client
 * Endpoints for employees to manage their own data
 */

import { claimOt, fetchAttendanceSummaries } from "./attendance";
import { client } from "./client";
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
  AttendanceDailySummary,
} from "./types";

// --- Profile ---

export async function fetchMyProfile(): Promise<MyProfileResponse> {
  const { data: resData } = await client.GET<MyProfileResponse>("/api/v1/ess/profile");
  return resData;
}

export async function updateMyProfile(
  data: UpdateProfileRequest
): Promise<EmployeeProfile> {
  const { data: resData } = await client.PUT<EmployeeProfile>("/api/v1/ess/profile", {
    body: data,
  });
  return resData;
}

// --- Emergency Contacts ---

export async function fetchMyEmergencyContacts(): Promise<
  EmployeeEmergencyContact[]
> {
  const { data: resData } = await client.GET<EmployeeEmergencyContact[]>("/api/v1/ess/emergency-contacts");
  return resData;
}

export async function addEmergencyContact(
  data: CreateEmergencyContactRequest
): Promise<EmployeeEmergencyContact> {
  const { data: resData } = await client.POST<EmployeeEmergencyContact>("/api/v1/ess/emergency-contacts", {
    body: data,
  });
  return resData;
}

export async function deleteEmergencyContact(contactId: string): Promise<void> {
  await client.DELETE("/api/v1/ess/emergency-contacts/{contact_id}", {
    params: {
      path: { contact_id: contactId },
    },
  });
}

// --- Documents ---

export async function fetchMyDocuments(): Promise<EmployeeDocument[]> {
  const { data: resData } = await client.GET<EmployeeDocument[]>("/api/v1/ess/documents");
  return resData;
}

// Document upload will be implemented in Phase 4

// --- Payslips ---

export async function fetchMyPayslips(): Promise<Payslip[]> {
  const { data: resData } = await client.GET<Payslip[]>("/api/v1/ess/payslips");
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
  const { data: resData } = await client.GET<TimeEntry[]>("/api/v1/ess/attendance", {
    params: {
      query: {
        start_date: params?.start_date,
        end_date: params?.end_date,
      },
    },
  });
  return resData;
}

// --- Time Clock ---

export type AttendanceAction = "work_in" | "work_out" | "break_start" | "break_end";

export interface AttendanceActionResponse {
  state: "off" | "working" | "on_break" | "completed" | "exception";
  next_allowed_actions: AttendanceAction[];
  entry?: TimeEntry;
  break?: {
    id: string;
    time_entry_id: string;
    start_at: string;
    end_at?: string | null;
    source: string;
    close_reason?: string | null;
  };
  exception_code?: string | null;
  deduplicated: boolean;
}

export async function fetchMyTimeEntries(
  params?: { start_date?: string; end_date?: string }
): Promise<TimeEntry[]> {
  const { data: resData } = await client.GET<TimeEntry[]>("/api/v1/ess/time-entries", {
    params: {
      query: {
        start_date: params?.start_date,
        end_date: params?.end_date,
      },
    },
  });
  return resData;
}

export async function fetchMyAttendanceState(): Promise<AttendanceActionResponse> {
  const { data: resData } = await client.GET<AttendanceActionResponse>("/api/v1/ess/attendance/state");
  return resData;
}

export async function submitMyAttendanceAction(
  action: AttendanceAction,
  idempotencyKey: string,
  note?: string,
): Promise<AttendanceActionResponse> {
  const { data: resData } = await client.POST<AttendanceActionResponse>("/api/v1/ess/attendance/actions", {
    body: { action, ...(note ? { note } : {}) },
    headers: { "Idempotency-Key": idempotencyKey },
  });
  return resData;
}

// --- Leave ---

export async function fetchEssLeaveBalances(): Promise<EmployeeLeaveBalance[]> {
  const { data: resData } = await client.GET<EmployeeLeaveBalance[]>("/api/v1/ess/leave/balances");
  return resData;
}

export async function fetchEssLeaveRequests(): Promise<LeaveRequest[]> {
  const { data: resData } = await client.GET<LeaveRequest[]>("/api/v1/ess/leave/requests");
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
  return fetchAttendanceSummaries(start, end);
}

export async function submitMyOtClaim(
  date: string,
  minutes: number,
  reason: string
): Promise<void> {
  const profile = await fetchMyProfile();
  const employeeId = profile.employee.id;

  return claimOt(employeeId, { date, claimed_minutes: minutes, reason });
}
