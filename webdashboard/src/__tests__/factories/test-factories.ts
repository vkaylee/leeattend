// If ToastContextType does not exist, we'll use any internally for now, but we'll try to import it:
import type { UseMyProfileReturn, UseMyClockReturn, UseMyAttendanceReturn, UseMyAttendanceSummariesReturn } from '@/hooks/use-ess';
import type { Employee, Shift, LeaveType, AttendanceAdjustmentRequest } from '@/lib/api/types';

// Basic Type for Toast Context if it's not exported
export interface ToastContextType {
  dismiss: (toastId?: string) => void;
  toast: (props: any) => { id: string; dismiss: () => void; update: (props: any) => void };
  toasts: any[];
}


export function createMockEmployee(overrides?: Partial<Employee>): Employee {
  return {
    id: "emp-123",
    employee_id_number: "EMP001",
    full_name: "John Doe",
    active_employment_id: "employment-123",
    workspace_id: "ws-123",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    tags: [],
    ...overrides,
  };
}

export function createMockShift(overrides?: Partial<Shift>): Shift {
  return {
    id: "shift-123",
    name: "Morning Shift",
    is_active: true,
    workspace_id: "ws-123",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    intervals: [{
      start_time: "09:00:00",
      end_time: "17:00:00",
      break_duration_minutes: 60,
      is_break_auto_deduct: false,
      break_is_overtime: false,
      early_punch_in_grace_period_minutes: 0,
      allow_early_ot: false,
    }],
    ...overrides,
  } as Shift;
}

export function createMockLeaveType(overrides?: Partial<LeaveType>): LeaveType {
  return {
    id: "leave-type-123",
    code: "AL",
    name: "Annual Leave",
    is_paid: true,
    workspace_id: "ws-123",
    policy_config: {},
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

export function createMockAttendanceAdjustment(overrides?: Partial<AttendanceAdjustmentRequest>): AttendanceAdjustmentRequest {
  return {
    id: "adj-123",
    employment_id: "employment-123",
    is_break_overtime: false,
    requested_punch_time: "2023-10-01T09:00:00Z",
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
    updateProfile: async () => Promise.resolve({
      id: 'prof-123',
      employee_id: 'emp-123',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }),
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
    punch: async () => Promise.resolve({
      id: 'entry-123',
      employment_id: 'employment-123',
      entry_type: 'Work',
      is_break_overtime: false,
      punch_in_time: new Date().toISOString(),
      workspace_id: 'ws-123',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }),
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
