/* eslint-disable @typescript-eslint/no-redundant-type-constituents, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-unsafe-call */
import type { UseMutationResult } from "@tanstack/react-query";

import { format } from "date-fns";

import type { Shift } from "@/lib/api/types";

interface UseShiftModalHandlersProps {
  correctionEffectiveDate: string;
  editingShift: Shift | null;
  formActions: {
    setEditingShift: (v: Shift | null) => void;
    setNewShiftName: (v: string) => void;
    setNewShiftSymbol: (v: string) => void;
    setNewShiftStart: (v: string) => void;
    setNewShiftEnd: (v: string) => void;
    setNewShiftBreak: (v: number) => void;
    setNewShiftGrace: (v: number) => void;
    setNewBreakStart: (v: string) => void;
    setNewBreakEnd: (v: string) => void;
    setIsFixedBreak: (v: boolean) => void;
    setIsAutoDeduct: (v: boolean) => void;
    setIsBreakOvertime: (v: boolean) => void;
    setIsCorrection: (v: boolean) => void;
    setCorrectionEffectiveDate: (v: string) => void;
    setAllowEarlyOt: (v: boolean) => void;
    setMaxEarlyOtMinutes: (v: number) => void;
  };
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isCorrection: boolean;
  isFixedBreak: boolean;
  allowEarlyOt: boolean;
  maxEarlyOtMinutes: number;
  mutations: {
    createShiftMutation: UseMutationResult<Shift, Error, Partial<Shift>>;
    updateShiftMutation: UseMutationResult<Shift, Error, { id: string; data: Partial<Shift> }>;
  };
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  newShiftEnd: string;
  newShiftGrace: number;
  newShiftName: string;
  newShiftStart: string;
  newShiftSymbol: string;
}

export function useShiftModalHandlers({
  newShiftName,
  newShiftStart,
  newShiftEnd,
  newShiftSymbol,
  newShiftBreak,
  newShiftGrace,
  isFixedBreak,
  newBreakStart,
  newBreakEnd,
  isAutoDeduct,
  isBreakOvertime,
  allowEarlyOt,
  maxEarlyOtMinutes,
  editingShift,
  isCorrection,
  correctionEffectiveDate,
  mutations,
  formActions
}: UseShiftModalHandlersProps) {

  const handleCreateShift = () => {
    if (!newShiftName || !newShiftStart || !newShiftEnd) return;
    mutations.createShiftMutation.mutate({
      name: newShiftName,
      report_symbol: newShiftSymbol || undefined,
      start_time: newShiftStart + ":00",
      end_time: newShiftEnd + ":00",
      break_duration_minutes: newShiftBreak,
      early_punch_in_grace_period_minutes: newShiftGrace,
      break_is_overtime: isBreakOvertime,
      allow_early_ot: allowEarlyOt,
      max_early_ot_minutes: allowEarlyOt ? maxEarlyOtMinutes : null,
      ...(isFixedBreak && newBreakStart && newBreakEnd ? {
        break_start: newBreakStart + ":00",
        break_end: newBreakEnd + ":00"
      } : {}),
      is_break_auto_deduct: isAutoDeduct
    });
  };

  const handleUpdateShift = () => {
    if (!editingShift) return;
    mutations.updateShiftMutation.mutate({
      id: editingShift.id,
      data: {
        name: newShiftName,
        report_symbol: newShiftSymbol || null,
        start_time: newShiftStart,
        end_time: newShiftEnd,
        break_duration_minutes: newShiftBreak,
        early_punch_in_grace_period_minutes: newShiftGrace,
        break_is_overtime: isBreakOvertime,
        allow_early_ot: allowEarlyOt,
        max_early_ot_minutes: allowEarlyOt ? maxEarlyOtMinutes : null,
        ...(isFixedBreak && newBreakStart && newBreakEnd ? {
          break_start: newBreakStart,
          break_end: newBreakEnd
        } : {
          break_start: null,
          break_end: null
        }),
        is_break_auto_deduct: isAutoDeduct,
        is_correction: isCorrection,
        effective_date: isCorrection ? undefined : correctionEffectiveDate
      }
    });
  };

  const openEditModal = (shift: Shift) => {
    formActions.setEditingShift(shift);
    formActions.setNewShiftName(shift.name);
    formActions.setNewShiftSymbol(shift.report_symbol ?? "");
    formActions.setNewShiftStart(shift.start_time.substring(0, 5));
    formActions.setNewShiftEnd(shift.end_time.substring(0, 5));
    formActions.setNewShiftBreak(shift.break_duration_minutes);
    formActions.setNewShiftGrace(shift.early_punch_in_grace_period_minutes ?? 15);
    formActions.setNewBreakStart(shift.break_start?.substring(0, 5) ?? "");
    formActions.setNewBreakEnd(shift.break_end?.substring(0, 5) ?? "");
    formActions.setIsFixedBreak(!!shift.break_start);
    formActions.setIsAutoDeduct(shift.is_break_auto_deduct ?? true);
    formActions.setIsBreakOvertime(shift.break_is_overtime ?? false);
    formActions.setAllowEarlyOt(shift.allow_early_ot ?? false);
    formActions.setMaxEarlyOtMinutes(shift.max_early_ot_minutes ?? 60);
    formActions.setIsCorrection(false);
    formActions.setCorrectionEffectiveDate(format(new Date(), 'yyyy-MM-dd'));
  };

  return {
    handleCreateShift,
    handleUpdateShift,
    openEditModal
  };
}
