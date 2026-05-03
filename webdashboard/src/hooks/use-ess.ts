/**
 * Employee Self-Service (ESS) React Hooks
 * Provides data fetching and mutations for ESS features
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";

import {
  AttendanceEntry,
  AttendanceDailySummary,
  AttendanceParams,
  CreateEmergencyContactRequest,
  EmployeeEmergencyContact,
  Employee,
  EmployeeLeaveBalance,
  LeaveRequest,
  EmployeeDocument,
  MyProfileResponse,
  EmployeeProfile,
  Payslip,
  TimeEntry,
  UpdateProfileRequest,
  addEmergencyContact,
  deleteEmergencyContact,
  fetchEssLeaveBalances,
  fetchEssLeaveRequests,
  fetchMyAttendance,
  fetchMyAttendanceSummaries,
  fetchMyDocuments,
  fetchMyEmergencyContacts,
  fetchMyPayslips,
  fetchMyProfile,
  fetchMyTimeEntries,
  punchInMySelf,
  punchOutMySelf,
  submitMyOtClaim,
  updateMyProfile,
} from "@/lib/api";

// Query Keys
export const essKeys = {
  all: ["ess"] as const,
  profile: () => [...essKeys.all, "profile"] as const,
  emergencyContacts: () => [...essKeys.all, "emergency-contacts"] as const,
  documents: () => [...essKeys.all, "documents"] as const,
  payslips: () => [...essKeys.all, "payslips"] as const,
  attendance: (params?: AttendanceParams) =>
    [...essKeys.all, "attendance", params] as const,
  clockEntries: () => [...essKeys.all, "clock-entries"] as const,
  leaveBalances: () => [...essKeys.all, "leave-balances"] as const,
  leaveRequests: () => [...essKeys.all, "leave-requests"] as const,
};

// --- Profile Hook ---

export interface UseMyProfileReturn {
  data: MyProfileResponse | undefined;
  emergencyContacts: EmployeeEmergencyContact[];
  employee: Employee | undefined;
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  isUpdating: boolean;
  profile: EmployeeProfile | undefined;
  refetch: () => Promise<unknown>;
  updateProfile: (data: UpdateProfileRequest) => Promise<EmployeeProfile>;
}

export function useMyProfile(): UseMyProfileReturn {
  const queryClient = useQueryClient();

  const profileQuery = useQuery<MyProfileResponse>({
    queryKey: essKeys.profile(),
    queryFn: fetchMyProfile,
  });

  const updateMutation = useMutation<EmployeeProfile, Error, UpdateProfileRequest>({
    mutationFn: (data: UpdateProfileRequest) => updateMyProfile(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  return {
    data: profileQuery.data,
    employee: profileQuery.data?.employee,
    profile: profileQuery.data?.profile,
    emergencyContacts: profileQuery.data?.emergency_contacts ?? [],
    isLoading: profileQuery.isLoading,
    isError: profileQuery.isError,
    error: profileQuery.error,
    refetch: profileQuery.refetch, // Cast for simplicity in this specific shared hook

    updateProfile: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
  };
}

// --- Emergency Contacts Hook ---

export interface UseEmergencyContactsReturn {
  addContact: (data: CreateEmergencyContactRequest) => Promise<EmployeeEmergencyContact>;
  contacts: EmployeeEmergencyContact[];
  deleteContact: (contactId: string) => Promise<unknown>;
  error: Error | null;
  isAdding: boolean;
  isDeleting: boolean;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}

export function useEmergencyContacts(): UseEmergencyContactsReturn {
  const queryClient = useQueryClient();

  const contactsQuery = useQuery<EmployeeEmergencyContact[]>({
    queryKey: essKeys.emergencyContacts(),
    queryFn: fetchMyEmergencyContacts,
  });

  const addMutation = useMutation<EmployeeEmergencyContact, Error, CreateEmergencyContactRequest>({
    mutationFn: (data: CreateEmergencyContactRequest) =>
      addEmergencyContact(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.emergencyContacts() });
      void queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  const deleteMutation = useMutation<unknown, Error, string>({
    mutationFn: (contactId: string) => deleteEmergencyContact(contactId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.emergencyContacts() });
      void queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  return {
    contacts: contactsQuery.data ?? [],
    isLoading: contactsQuery.isLoading,
    isError: contactsQuery.isError,
    error: contactsQuery.error,
    refetch: contactsQuery.refetch,

    addContact: addMutation.mutateAsync,
    isAdding: addMutation.isPending,

    deleteContact: deleteMutation.mutateAsync,
    isDeleting: deleteMutation.isPending,
  };
}

// --- Documents Hook ---

export interface UseMyDocumentsReturn {
  documents: EmployeeDocument[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}

export function useMyDocuments(): UseMyDocumentsReturn {
  const documentsQuery = useQuery<EmployeeDocument[]>({
    queryKey: essKeys.documents(),
    queryFn: fetchMyDocuments,
  });

  return {
    documents: documentsQuery.data ?? [],
    isLoading: documentsQuery.isLoading,
    isError: documentsQuery.isError,
    error: documentsQuery.error,
    refetch: documentsQuery.refetch,
  };
}

// --- Payslips Hook ---

export interface UseMyPayslipsReturn {
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  payslips: Payslip[];
  refetch: () => Promise<unknown>;
}

export function useMyPayslips(): UseMyPayslipsReturn {
  const payslipsQuery = useQuery<Payslip[]>({
    queryKey: essKeys.payslips(),
    queryFn: fetchMyPayslips,
  });

  return {
    payslips: payslipsQuery.data ?? [],
    isLoading: payslipsQuery.isLoading,
    isError: payslipsQuery.isError,
    error: payslipsQuery.error,
    refetch: payslipsQuery.refetch,
  };
}

// --- Attendance Hook ---

export interface UseMyAttendanceReturn {
  entries: AttendanceEntry[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
}

export function useMyAttendance(params?: AttendanceParams): UseMyAttendanceReturn {
  const attendanceQuery = useQuery<AttendanceEntry[]>({
    queryKey: essKeys.attendance(params),
    queryFn: () => fetchMyAttendance(params),
  });

  return {
    entries: attendanceQuery.data ?? [],
    isLoading: attendanceQuery.isLoading,
    isError: attendanceQuery.isError,
    error: attendanceQuery.error,
    refetch: attendanceQuery.refetch,
  };
}

// --- Leave Hook ---

export interface UseMyLeaveReturn {
  balances: EmployeeLeaveBalance[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetchBalances: () => Promise<unknown>;
  refetchRequests: () => Promise<unknown>;
  requests: LeaveRequest[];
}

export function useMyLeave(): UseMyLeaveReturn {
  const balancesQuery = useQuery<EmployeeLeaveBalance[]>({
    queryKey: essKeys.leaveBalances(),
    queryFn: fetchEssLeaveBalances,
  });

  const requestsQuery = useQuery<LeaveRequest[]>({
    queryKey: essKeys.leaveRequests(),
    queryFn: fetchEssLeaveRequests,
  });

  return {
    balances: balancesQuery.data ?? [],
    requests: requestsQuery.data ?? [],
    isLoading: balancesQuery.isLoading || requestsQuery.isLoading,
    isError: balancesQuery.isError || requestsQuery.isError,
    error: balancesQuery.error ?? requestsQuery.error,
    refetchBalances: balancesQuery.refetch,
    refetchRequests: requestsQuery.refetch,
  };
}

// --- Clock Hook ---

export interface UseMyClockReturn {
  activeEntry: TimeEntry | undefined;
  entries: TimeEntry[];
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  isOnBreak: boolean;
  isPunching: boolean;
  punchIn: (data: { entry_type: "work" | "break"; note?: string }) => Promise<TimeEntry>;
  punchOut: (data?: { note?: string }) => Promise<TimeEntry>;
  refetch: () => Promise<unknown>;
}

export function useMyClock(): UseMyClockReturn {
  const queryClient = useQueryClient();

  // Fetch current user's time entries (today + 7 days)
  const { startDate, endDate } = useMemo(() => {
    const now = new Date();
    const endDate = now.toISOString();
    const startDate = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString();
    return { startDate, endDate };
  }, []);

  const entriesQuery = useQuery<TimeEntry[]>({
    queryKey: essKeys.clockEntries(),
    queryFn: () => fetchMyTimeEntries({ start_date: startDate, end_date: endDate }),
    refetchInterval: 30000, // Refresh every 30s for real-time status
  });

  const activeEntry = entriesQuery.data?.find((e) => !e.punch_out_time);
  const isOnBreak = activeEntry?.entry_type === "break";

  const punchInMutation = useMutation<TimeEntry, Error, { entry_type: "work" | "break"; note?: string }>({
    mutationFn: (data: { entry_type: "work" | "break"; note?: string }) =>
      punchInMySelf(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.clockEntries() });
      void queryClient.invalidateQueries({ queryKey: essKeys.attendance() });
    },
  });

  const punchOutMutation = useMutation<TimeEntry, Error, { note?: string } | undefined>({
    mutationFn: (data?: { note?: string }) => punchOutMySelf(data),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: essKeys.clockEntries() });
      void queryClient.invalidateQueries({ queryKey: essKeys.attendance() });
    },
  });

  return {
    entries: entriesQuery.data ?? [],
    activeEntry,
    isOnBreak,
    isLoading: entriesQuery.isLoading,
    isError: entriesQuery.isError,
    error: entriesQuery.error,
    refetch: entriesQuery.refetch,

    punchIn: punchInMutation.mutateAsync,
    punchOut: punchOutMutation.mutateAsync,
    isPunching: punchInMutation.isPending || punchOutMutation.isPending,
  };
}

// --- OT Hooks ---

export interface UseMyAttendanceSummariesReturn {
  error: Error | null;
  isError: boolean;
  isLoading: boolean;
  refetch: () => Promise<unknown>;
  summaries: AttendanceDailySummary[];
}

export function useMyAttendanceSummaries(start: string, end: string): UseMyAttendanceSummariesReturn {
  const query = useQuery<AttendanceDailySummary[]>({
    queryKey: [...essKeys.all, "summaries", start, end],
    queryFn: () => fetchMyAttendanceSummaries(start, end),
  });

  return {
    summaries: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    refetch: query.refetch,
  };
}

export interface UseSubmitOtClaimReturn {
  isSubmitting: boolean;
  submitClaim: (data: { date: string; minutes: number; reason: string }) => Promise<unknown>;
}

export function useSubmitOtClaim(): UseSubmitOtClaimReturn {
  const queryClient = useQueryClient();

  const mutation = useMutation<unknown, Error, { date: string; minutes: number; reason: string }>({
    mutationFn: (data: { date: string; minutes: number; reason: string }) =>
      submitMyOtClaim(data.date, data.minutes, data.reason),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: [...essKeys.all, "summaries"] });
    },
  });

  return {
    submitClaim: mutation.mutateAsync,
    isSubmitting: mutation.isPending,
  };
}
