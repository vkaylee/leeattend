import { UseMutationResult } from "@tanstack/react-query";

import {
  AssignPatternRequest,
  AssignmentHistory,
  Employee,
  EmployeeSchedule,
  EmployeeWorkingPattern,
  Shift,
  WorkingPattern,
} from "@/lib/api/types";

export type TabType = "weekly-roster" | "assignments" | "directory";

export interface UseShiftsReturn {
  activeTab: TabType;
  assignPatternMutation: UseMutationResult<EmployeeWorkingPattern, Error, AssignPatternRequest>;
  bulkRosterMutation: UseMutationResult<unknown, Error, { date: string; shift_id: string | null }[]>;
  correctionEffectiveDate: string;
  createShiftMutation: UseMutationResult<Shift, Error, Partial<Shift>>;
  currentDate: Date;
  deleteRosterMutation: UseMutationResult<unknown, Error, string>;
  editingShift: Shift | null;
  employeeSchedule: EmployeeSchedule | undefined;
  endDate: string;
  handleAssign: () => void;
  handleAssignAllWeek: (shiftId: string | null) => void;
  handleAssignBulk: (shiftId: string | null) => void;
  handleCopyPreviousWeek: () => Promise<unknown>;
  handleCreateShift: () => void;
  handleNavigateWeek: (direction: "prev" | "next") => void;
  handleUpdateShift: () => void;
  history: AssignmentHistory[] | undefined;
  isAutoDeduct: boolean;
  isCorrection: boolean;
  isCreateModalOpen: boolean;
  isFixedBreak: boolean;
  isForever: boolean;
  isShiftPickerOpen: boolean;
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  newShiftEnd: string;
  newShiftGrace: number;
  newShiftName: string;
  newShiftStart: string;
  newShiftSymbol: string;
  openEditModal: (shift: Shift) => void;
  patterns: WorkingPattern[] | undefined;
  selectedDays: string[];
  selectedEmployee: Employee | null;
  selectedPatternId: string;
  setActiveTab: (v: TabType) => void;
  setCorrectionEffectiveDate: (v: string) => void;
  setCurrentDate: (v: Date) => void;
  setEditingShift: (v: Shift | null) => void;
  setEndDate: (v: string) => void;
  setIsAutoDeduct: (v: boolean) => void;
  setIsCorrection: (v: boolean) => void;
  setIsCreateModalOpen: (v: boolean) => void;
  setIsFixedBreak: (v: boolean) => void;
  setIsForever: (v: boolean) => void;
  setIsShiftPickerOpen: (v: boolean) => void;
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
  setSelectedPatternId: (v: string) => void;
  setStartDate: (v: string) => void;
  setTargetDates: (v: string[]) => void;
  shifts: Shift[] | undefined;
  startDate: string;
  targetDates: string[];
  updateShiftMutation: UseMutationResult<Shift, Error, { id: string; data: Partial<Shift> }>;
  weekDays: Date[];
  wsId: string;
}
