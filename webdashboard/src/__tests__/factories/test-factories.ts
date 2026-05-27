// If ToastContextType does not exist, we'll use any internally for now, but we'll try to import it:
import type { UseMyProfileReturn, UseMyClockReturn, UseMyAttendanceReturn, UseMyAttendanceSummariesReturn } from '@/hooks/use-ess';
import type { TenantConnectionStats } from '@/lib/api/system';
import type { Workspace, Employee, Shift, LeaveType, AttendanceAdjustmentRequest } from '@/lib/api/types';

// Basic Type for Toast Context if it's not exported
export interface ToastContextType {
  dismiss: (toastId?: string) => void;
  toast: (props: any) => { id: string; dismiss: () => void; update: (props: any) => void };
  toasts: any[];
}

export function createMockWorkspace(overrides?: Partial<Workspace>): Workspace {
  return {
    id: "ws-123",
    name: "Test Workspace",
    created_at: new Date().toISOString(),
    date_format: "YYYY-MM-DD",
    default_language: "en",
    rounding_policy: "nearest_5",
    theme_primary_color: "#000000",
    time_format: "24h",
    timezone: "UTC",
    work_unit_minutes: 60,
    ...overrides,
  };
}

export function createMockEmployee(overrides?: Partial<Employee>): Employee {
  return {
    id: "emp-123",
    employee_id_number: "EMP001",
    full_name: "John Doe",
    hire_date: new Date().toISOString(),
    original_hire_date: new Date().toISOString(),
    is_active: true,
    workspace_id: "ws-123",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    ...overrides,
  };
}

export function createMockShift(overrides?: Partial<Shift>): Shift {
  return {
    id: "shift-123",
    name: "Morning Shift",
    start_time: "09:00:00",
    end_time: "17:00:00",
    break_duration_minutes: 60,
    workspace_id: "ws-123",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    ...overrides,
  };
}

export function createMockLeaveType(overrides?: Partial<LeaveType>): LeaveType {
  return {
    id: "leave-type-123",
    code: "AL",
    name: "Annual Leave",
    is_paid: true,
    workspace_id: "ws-123",
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

export function createMockAttendanceAdjustment(overrides?: Partial<AttendanceAdjustmentRequest>): AttendanceAdjustmentRequest {
  return {
    id: "adj-123",
    employee_id: "emp-123",
    is_break_overtime: false,
    requested_date: "2023-10-01",
    requested_punch_in_time: "2023-10-01T09:00:00Z",
    status: "pending",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    ...overrides,
  };
}

export function createMockUseMyProfile(overrides?: Partial<UseMyProfileReturn>): UseMyProfileReturn {
  return {
    data: undefined,
    emergencyContacts: [],
    employee: createMockEmployee(),
    error: null,
    isError: false,
    isLoading: false,
    isUpdating: false,
    profile: undefined,
    refetch: async () => Promise.resolve(),
    updateProfile: async () => Promise.resolve({} as any),
    ...overrides,
  };
}

export function createMockUseMyClockReturn(overrides?: Partial<UseMyClockReturn>): UseMyClockReturn {
  return {
    activeEntry: undefined,
    entries: [],
    error: null,
    isError: false,
    isLoading: false,
    isPunching: false,
    punch: async () => Promise.resolve({} as any),
    refetch: async () => Promise.resolve(),
    ...overrides,
  };
}

export function createMockUseMyAttendance(overrides?: Partial<UseMyAttendanceReturn>): UseMyAttendanceReturn {
  return {
    entries: [],
    error: null,
    isError: false,
    isLoading: false,
    refetch: async () => Promise.resolve(),
    ...overrides,
  };
}

export function createMockUseMyAttendanceSummaries(overrides?: Partial<UseMyAttendanceSummariesReturn>): UseMyAttendanceSummariesReturn {
  return {
    error: null,
    isError: false,
    isLoading: false,
    refetch: async () => Promise.resolve(),
    summaries: [],
    ...overrides,
  };
}

export const createMockToastContext = (): ToastContextType => ({
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  toast: () => ({ id: "1", dismiss: () => {}, update: () => {} }),
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  dismiss: () => {},
  toasts: [],
});

export function createMockTenantConnectionStats(overrides?: Partial<TenantConnectionStats>): TenantConnectionStats {
  return {
    config: {},
    connection: {
      host: "localhost",
      port: "5432",
    },
    ...overrides,
  };
}
