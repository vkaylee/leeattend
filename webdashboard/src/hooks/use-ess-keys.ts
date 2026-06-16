import { AttendanceParams } from "@/lib/api";

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
