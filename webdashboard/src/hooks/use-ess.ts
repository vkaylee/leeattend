/**
 * Employee Self-Service (ESS) React Hooks
 * Provides data fetching and mutations for ESS features
 */

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  fetchMyProfile,
  updateMyProfile,
  fetchMyEmergencyContacts,
  addEmergencyContact,
  deleteEmergencyContact,
  fetchMyDocuments,
  fetchMyPayslips,
  fetchMyAttendance,
  fetchEssLeaveBalances,
  fetchEssLeaveRequests,
  UpdateProfileRequest,
  CreateEmergencyContactRequest,
  AttendanceParams,
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
  leaveBalances: () => [...essKeys.all, "leave-balances"] as const,
  leaveRequests: () => [...essKeys.all, "leave-requests"] as const,
};

// --- Profile Hook ---

export function useMyProfile() {
  const queryClient = useQueryClient();

  const profileQuery = useQuery({
    queryKey: essKeys.profile(),
    queryFn: fetchMyProfile,
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateProfileRequest) => updateMyProfile(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  return {
    data: profileQuery.data,
    employee: profileQuery.data?.employee,
    profile: profileQuery.data?.profile,
    emergencyContacts: profileQuery.data?.emergency_contacts || [],
    isLoading: profileQuery.isLoading,
    isError: profileQuery.isError,
    error: profileQuery.error,
    refetch: profileQuery.refetch,

    updateProfile: updateMutation.mutateAsync,
    isUpdating: updateMutation.isPending,
  };
}

// --- Emergency Contacts Hook ---

export function useEmergencyContacts() {
  const queryClient = useQueryClient();

  const contactsQuery = useQuery({
    queryKey: essKeys.emergencyContacts(),
    queryFn: fetchMyEmergencyContacts,
  });

  const addMutation = useMutation({
    mutationFn: (data: CreateEmergencyContactRequest) =>
      addEmergencyContact(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: essKeys.emergencyContacts() });
      queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (contactId: string) => deleteEmergencyContact(contactId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: essKeys.emergencyContacts() });
      queryClient.invalidateQueries({ queryKey: essKeys.profile() });
    },
  });

  return {
    contacts: contactsQuery.data || [],
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

export function useMyDocuments() {
  const documentsQuery = useQuery({
    queryKey: essKeys.documents(),
    queryFn: fetchMyDocuments,
  });

  return {
    documents: documentsQuery.data || [],
    isLoading: documentsQuery.isLoading,
    isError: documentsQuery.isError,
    error: documentsQuery.error,
    refetch: documentsQuery.refetch,
  };
}

// --- Payslips Hook ---

export function useMyPayslips() {
  const payslipsQuery = useQuery({
    queryKey: essKeys.payslips(),
    queryFn: fetchMyPayslips,
  });

  return {
    payslips: payslipsQuery.data || [],
    isLoading: payslipsQuery.isLoading,
    isError: payslipsQuery.isError,
    error: payslipsQuery.error,
    refetch: payslipsQuery.refetch,
  };
}

// --- Attendance Hook ---

export function useMyAttendance(params?: AttendanceParams) {
  const attendanceQuery = useQuery({
    queryKey: essKeys.attendance(params),
    queryFn: () => fetchMyAttendance(params),
  });

  return {
    entries: attendanceQuery.data || [],
    isLoading: attendanceQuery.isLoading,
    isError: attendanceQuery.isError,
    error: attendanceQuery.error,
    refetch: attendanceQuery.refetch,
  };
}

// --- Leave Hook ---

export function useMyLeave() {
  const balancesQuery = useQuery({
    queryKey: essKeys.leaveBalances(),
    queryFn: fetchEssLeaveBalances,
  });

  const requestsQuery = useQuery({
    queryKey: essKeys.leaveRequests(),
    queryFn: fetchEssLeaveRequests,
  });

  return {
    balances: balancesQuery.data || [],
    requests: requestsQuery.data || [],
    isLoading: balancesQuery.isLoading || requestsQuery.isLoading,
    isError: balancesQuery.isError || requestsQuery.isError,
    error: balancesQuery.error || requestsQuery.error,
    refetchBalances: balancesQuery.refetch,
    refetchRequests: requestsQuery.refetch,
  };
}
