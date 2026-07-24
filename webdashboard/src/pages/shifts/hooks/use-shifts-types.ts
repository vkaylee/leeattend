 
import { UseMutationResult } from "@tanstack/react-query";

import type { ComplianceViolation } from "@/lib/api/errors";

import {
  AssignPatternRequest,
  AssignmentHistory,
  Department,
  Employee,
  EmployeeSchedule,
  EmployeeWorkingPattern,
  Shift,
  WorkingPattern,
  CreateShiftRequest,
  UpdateShiftRequest,
} from "@/lib/api/types";

export type TabType = "weekly-roster" | "assignments" | "directory";
export type ViewMode = "week" | "month";

// Runtime extension: intervals exist at runtime but are not in the generated API schema.
export type ShiftWithIntervals = Shift & {
  intervals?: { start_time?: string; end_time?: string }[];
};

export interface UseShiftsReturn {
  activeTab: TabType;
  allowEarlyOt: boolean;
  assignPatternMutation: UseMutationResult<EmployeeWorkingPattern, Error, AssignPatternRequest>;
  bulkRosterMutation: UseMutationResult<unknown, Error, { date: string; shift_id: string | null }[]>;
  complianceModalOpen: boolean;
  complianceViolations: ComplianceViolation[];
  correctionEffectiveDate: string;
  createShiftMutation: UseMutationResult<Shift, Error, CreateShiftRequest>;
  currentDate: Date;
  currentMonth: Date;
  deleteRosterMutation: UseMutationResult<unknown, Error, string>;
  departments: Department[] | undefined;
  editingShift: Shift | null;
  employees: Employee[] | undefined;
  employeeSchedule: EmployeeSchedule | undefined;
  employeesError: Error | null;
  endDate: string;
  handleAssign: () => void;


  handleAssignAllWeek: (shiftId: string | null) => void;
  handleAssignBulk: (shiftId: string | null) => void;
  handleCopyPreviousMonth: () => Promise<void>;
  handleCopyPreviousWeek: () => Promise<unknown>;
  handleCreateShift: () => void;
  handleMonthBulkAssign: (assignments: { employeeId: string; employmentId?: string; dates: string[] }[], shiftId: string | null, onRowStatus?: (employeeId: string, status: "saving" | "saved" | "error") => void) => Promise<void>;
  handleMonthDeleteRoster: (assignments: { employeeId: string; dates: string[] }[]) => Promise<void>;
  handleNavigateMonth: (direction: "prev" | "next") => void;
  handleNavigateWeek: (direction: "prev" | "next") => void;
  handleUpdateShift: () => void;
  history: AssignmentHistory[] | undefined;
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isComplianceResolving: boolean;
  isCorrection: boolean;
  isCreateModalOpen: boolean;
  isEmployeesError: boolean;
  isEmployeesLoading: boolean;
  isFixedBreak: boolean;
  isForever: boolean;
  isMonthError: boolean;
  isMonthLoading: boolean;
  isPatternModalOpen: boolean;
  isShiftPickerOpen: boolean;
  maxEarlyOtMinutes: number;
  monthDays: Date[];
  monthError: Error | null;
  monthSchedules: EmployeeSchedule[] | undefined;
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  newShiftEnd: string;
  newShiftGrace: number;
  newShiftName: string;
  newShiftStart: string;
  newShiftSymbol: string;
  onComplianceOverride: (reason: string) => void;
  openEditModal: (shift: Shift) => void;
  patterns: WorkingPattern[] | undefined;
  refetchEmployees: () => Promise<unknown>;
  refetchMonthSchedules: () => Promise<unknown>;
  selectedDays: string[];
  selectedEmployee: Employee | null;
  selectedEmployeeIds: string[];
  selectedEmployees: Employee[];
  selectedPatternId: string;
  setActiveTab: (v: TabType) => void;
  setAllowEarlyOt: (v: boolean) => void;
  setComplianceModalOpen: (v: boolean) => void;
  setCorrectionEffectiveDate: (v: string) => void;
  setCurrentDate: (v: Date) => void;
  setEditingShift: (v: Shift | null) => void;
  setEndDate: (v: string) => void;
  setIsAutoDeduct: (v: boolean) => void;
  setIsBreakOvertime: (v: boolean) => void;
  setIsCorrection: (v: boolean) => void;
  setIsCreateModalOpen: (v: boolean) => void;
  setIsFixedBreak: (v: boolean) => void;
  setIsForever: (v: boolean) => void;
  setIsPatternModalOpen: (v: boolean) => void;
  setIsShiftPickerOpen: (v: boolean) => void;
  setMaxEarlyOtMinutes: (v: number) => void;
  setNewBreakEnd: (v: string) => void;
  setNewBreakStart: (v: string) => void;
  setNewShiftBreak: (v: number) => void;
  setNewShiftEnd: (v: string) => void;
  setNewShiftGrace: (v: number) => void;
  setNewShiftName: (v: string) => void;
  setNewShiftStart: (v: string) => void;
  setNewShiftSymbol: (v: string) => void;
  setSelectedDays: (v: string[]) => void;
  setSelectedEmployee: (v: Employee | null) => void;
  setSelectedEmployeeIds: (v: string[]) => void;
  setSelectedEmployees: (v: Employee[]) => void;
  setSelectedPatternId: (v: string) => void;
  setStartDate: (v: string) => void;
  setTargetDates: (v: string[]) => void;
  setViewMode: (v: ViewMode) => void;
  shifts: Shift[] | undefined;
  startDate: string;
  targetDates: string[];
  updateShiftMutation: UseMutationResult<Shift, Error, { id: string; data: UpdateShiftRequest }>;
  viewMode: ViewMode;
  weekDays: Date[];
  wsId: string;
}
