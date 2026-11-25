
/* eslint-disable max-lines-per-function, max-lines */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useBlocker, useNavigate } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { Shift, CreateShiftRequest, UpdateShiftRequest } from "@/lib/api/types";
import type { components } from "@/types/api-generated";

type CreateIntervalRequest = components["schemas"]["CreateIntervalRequest"];

import { useToast } from "@/components/ui/toast";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import { useMyPermissions } from "@/hooks/use-permissions";
import {
  fetchShifts,
  createShift,
  updateShift,
  fetchDefaultAttendanceRules,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";


export interface IntervalDraft {
  breakDuration: number;
  breakEnd: string;
  breakStart: string;
  endTime: string;
  endDayOffset: number;
  intervalName: string;
  breakPunchMode: "none" | "required";
  isFixedBreak: boolean;
  isNightShift: boolean;
  key: string;
  startDayOffset: number;
  startTime: string;
}

export type BreakValidationKey =
  | "break_start_required"
  | "break_end_required"
  | "break_order"
  | "break_outside_interval"
  | "break_duration_exceeds_interval"
  | "interval_order";

export function parseTimeMinutes(value: string): number | null {
  const match = /^(\d{2}):(\d{2})/.exec(value);
  if (!match) return null;
  const hours = Number(match[1]);
  const minutes = Number(match[2]);
  return hours >= 0 && hours < 24 && minutes >= 0 && minutes < 60
    ? hours * 60 + minutes
    : null;
}

export function intervalAbsoluteMinutes(interval: Pick<IntervalDraft, "startTime" | "endTime" | "startDayOffset" | "endDayOffset">) {
  const start = parseTimeMinutes(interval.startTime);
  const end = parseTimeMinutes(interval.endTime);
  if (start === null || end === null) return null;
  return {
    start: interval.startDayOffset * 1440 + start,
    end: interval.endDayOffset * 1440 + end,
  };
}

function resolveClockInInterval(clock: string, start: number, end: number): number | null {
  const minutes = parseTimeMinutes(clock);
  if (minutes === null) return null;
  const firstDay = Math.floor(start / 1440);
  const lastDay = Math.floor(end / 1440);
  for (let day = firstDay; day <= lastDay; day += 1) {
    const candidate = day * 1440 + minutes;
    if (candidate >= start && candidate <= end) return candidate;
  }
  return null;
}

export function getBreakValidation(interval: IntervalDraft): { key: BreakValidationKey | null; breakMinutes: number | null } {
  const range = intervalAbsoluteMinutes(interval);
  if (!range) return { key: "interval_order", breakMinutes: null };
  if (range.end <= range.start) return { key: "interval_order", breakMinutes: null };
  if (!interval.isFixedBreak) {
    if (interval.breakPunchMode === "none" && interval.breakDuration > range.end - range.start) {
      return { key: "break_duration_exceeds_interval", breakMinutes: null };
    }
    return { key: null, breakMinutes: null };
  }
  if (!interval.breakStart) return { key: "break_start_required", breakMinutes: null };
  if (!interval.breakEnd) return { key: "break_end_required", breakMinutes: null };
  const breakStart = resolveClockInInterval(interval.breakStart, range.start, range.end);
  const breakEnd = resolveClockInInterval(interval.breakEnd, range.start, range.end);
  if (breakStart === null || breakEnd === null) return { key: "break_outside_interval", breakMinutes: null };
  if (breakEnd <= breakStart) return { key: "break_order", breakMinutes: null };
  return { key: null, breakMinutes: breakEnd - breakStart };
}

export function getIntervalPreview(interval: IntervalDraft): { workMinutes: number; breakMinutes: number | null; breakLabel: "scheduled" | "actual" } | null {
  const range = intervalAbsoluteMinutes(interval);
  if (!range || range.end <= range.start) return null;
  const validation = getBreakValidation(interval);
  if (validation.key) return null;
  if (interval.breakPunchMode === "required") {
    return { workMinutes: range.end - range.start, breakMinutes: null, breakLabel: "actual" };
  }
  const breakMinutes = interval.isFixedBreak ? validation.breakMinutes : Math.max(0, interval.breakDuration);
  return { workMinutes: range.end - range.start - (breakMinutes ?? 0), breakMinutes, breakLabel: "scheduled" };
}

interface ShiftFormState {
  changeReason: string;
  correctionEffectiveDate: string;
  employerId: string | null;
  isCorrection: boolean;
  locationId: string | null;
  shiftName: string;
  shiftSymbol: string;
}

const EMPTY_SHIFT_FORM: ShiftFormState = {
  shiftName: "",
  shiftSymbol: "",
  changeReason: "",
  isCorrection: false,
  correctionEffectiveDate: format(new Date(), 'yyyy-MM-dd'),
  locationId: null,
  employerId: null,
};

let _intervalCounter = 0;
function nextIntervalKey(): string {
  _intervalCounter += 1;
  return `iv_${Date.now()}_${_intervalCounter}`;
}

function makeDefaultInterval(): IntervalDraft {
  return {
    key: nextIntervalKey(),
    intervalName: "",
    breakPunchMode: "none",
    startTime: "08:00",
    endTime: "17:00",
    startDayOffset: 0,
    endDayOffset: 0,
    breakDuration: 60,
    breakStart: "",
    breakEnd: "",
    isFixedBreak: false,
    isNightShift: false,
  };
}

export function useShiftEditor(shiftId?: string) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const queryClient = useQueryClient();


  const isEditMode = !!shiftId;

  // Fetch all shifts to find editing shift — must come before useMemo that uses editingShift
  const { data: shifts, isLoading: isLoadingShifts } = useQuery<Shift[]>({
    queryKey: createQueryKey("shifts"),
    queryFn: () => fetchShifts(),
    enabled: isEditMode,
  });

  const editingShift = shifts?.find((s) => s.id === shiftId) ?? null;

  // Compute external shift form state — stabilized via useMemo.
  // The editingShift reference only changes when the shift being edited changes.
  const externalShiftForm = useMemo((): ShiftFormState => {
    if (isEditMode && editingShift) {
      return {
        shiftName: editingShift.name,
        shiftSymbol: editingShift.report_symbol ?? "",
        locationId: editingShift.location_id ?? null,
        employerId: editingShift.employer_id ?? null,
        isCorrection: false,
        correctionEffectiveDate: format(new Date(), 'yyyy-MM-dd'),
        changeReason: "",
      };
    }
    return EMPTY_SHIFT_FORM;
  }, [isEditMode, editingShift]);

  const [shiftForm, setShiftForm] = useDerivedStateFrom(externalShiftForm);

  // Convenience accessors
  const shiftName = shiftForm.shiftName;
  const shiftSymbol = shiftForm.shiftSymbol;
  const locationId = shiftForm.locationId;
  const employerId = shiftForm.employerId;
  const isCorrection = shiftForm.isCorrection;
  const correctionEffectiveDate = shiftForm.correctionEffectiveDate;
  const changeReason = shiftForm.changeReason;

  const setLocationId = (v: string | null) => {
    setShiftForm(prev => ({ ...prev, locationId: v }));
    setIsDirty(true);
  };

  const setEmployerId = (v: string | null) => {
    setShiftForm(prev => ({ ...prev, employerId: v }));
    setIsDirty(true);
  };

  // Interval state — all time/break/policy lives here
  // (initialized via useDerivedStateFrom from externalIntervals below)

  const addInterval = () => {
    setIntervalDrafts(prev => [...prev, makeDefaultInterval()]);
    setIsDirty(true);
  };

  const removeInterval = (key: string) => {
    setIntervalDrafts(prev => {
      if (prev.length <= 1) return prev; // must keep at least 1
      return prev.filter(iv => iv.key !== key);
    });
    setIsDirty(true);
  };

  const updateIntervalDraft = (key: string, patch: Partial<IntervalDraft>) => {
    setIntervalDrafts(prev => prev.map(iv => {
      if (iv.key !== key) return iv;
      const next = { ...iv, ...patch };
      if (patch.isFixedBreak === false) {
        next.breakStart = "";
        next.breakEnd = "";
      }
      return next;
    }));
    setIsDirty(true);
  };

  const [isDirty, setIsDirty] = useState(false);
  const { hasPermission } = useMyPermissions();

  const { data: attendanceRules } = useQuery({
    queryKey: createQueryKey("attendance-rules"),
    queryFn: () => fetchDefaultAttendanceRules(),
    enabled: hasPermission("organization:settings:read"),
  });

  // Compute external intervals — stabilized via useMemo.
  const externalIntervals = useMemo((): IntervalDraft[] => {
    if (isEditMode && editingShift?.intervals?.length) {
      return editingShift.intervals.map((iv): IntervalDraft => ({
        key: nextIntervalKey(),
        intervalName: iv.interval_name ?? "",
        breakPunchMode: iv.break_punch_mode === "required" ? "required" : "none",
        startTime: iv.start_time.substring(0, 5),
        endTime: iv.end_time.substring(0, 5),
        startDayOffset: iv.start_day_offset ?? 0,
        endDayOffset: iv.end_day_offset ?? (iv.start_time > iv.end_time ? 1 : 0),
        breakDuration: iv.break_duration_minutes,
        breakStart: (iv.break_start ?? "").substring(0, 5),
        breakEnd: (iv.break_end ?? "").substring(0, 5),
        isFixedBreak: !!iv.break_start,
        isNightShift: iv.start_time > iv.end_time,
      }));
    }
    return [makeDefaultInterval()];
  }, [isEditMode, editingShift]);

  const [intervalDrafts, setIntervalDrafts] = useDerivedStateFrom(externalIntervals);

  // Mutations
  const createShiftMutation = useMutation<Shift, Error, CreateShiftRequest>({
    mutationFn: (data) => createShift(data),
    onSuccess: (data) => {
      toast.success(t("shifts.messages.create_shift_success", { name: data.name }));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("shifts") });
      setIsDirty(false);
      void navigate({ to: "/shifts", search: { tab: "directory", view: "week" } });
    },
    onError: (error) => {
      toast.error(error.message, t("shifts.messages.create_shift_error"));
    },
  });

  const updateShiftMutation = useMutation<Shift, Error, { id: string; data: UpdateShiftRequest }>({
    mutationFn: ({ id, data }) => updateShift(id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_shift_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("shifts") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("schedules") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("assignment_history") });
      setIsDirty(false);
      void navigate({ to: "/shifts", search: { tab: "directory", view: "week" } });
    },
    onError: (error) => {
      toast.error(error.message, t("shifts.messages.update_shift_error"));
    },
  });

  const isSaving = createShiftMutation.isPending || updateShiftMutation.isPending;
  const hasValidIntervals = intervalDrafts.length > 0 && intervalDrafts.every(iv => Boolean(iv.startTime && iv.endTime) && !getBreakValidation(iv).key);
  const isSaveDisabled = isSaving || !shiftName || !hasValidIntervals;

  const padTime = (t: string) => t.length === 5 ? t + ":00" : t;

  const buildPayload = (): CreateShiftRequest => {
    const intervals: CreateIntervalRequest[] = intervalDrafts.map(iv => ({
      interval_name: iv.intervalName || null,
      start_time: padTime(iv.startTime),
      end_time: padTime(iv.endTime),
      start_day_offset: iv.startDayOffset,
      end_day_offset: iv.endDayOffset,
      break_duration_minutes: iv.isFixedBreak ? getBreakValidation(iv).breakMinutes ?? 0 : iv.breakDuration,
      break_start: iv.isFixedBreak && iv.breakStart ? padTime(iv.breakStart) : null,
      break_end: iv.isFixedBreak && iv.breakEnd ? padTime(iv.breakEnd) : null,
      break_punch_mode: iv.breakPunchMode,
    }));

    return {
      name: shiftName,
      report_symbol: shiftSymbol || null,
      location_id: locationId,
      employer_id: employerId,
      intervals,
    };
  };

  const handleSubmit = (e?: React.SyntheticEvent) => {
    if (e) e.preventDefault();
    if (isSaveDisabled) return;

    const payload = buildPayload();

    if (isEditMode && shiftId) {
      const updatePayload: UpdateShiftRequest = {
        ...payload,
        is_correction: isCorrection,
        effective_date: isCorrection ? undefined : correctionEffectiveDate,
        change_reason: changeReason || undefined,
      };
      updateShiftMutation.mutate({ id: shiftId, data: updatePayload });
    } else {
      createShiftMutation.mutate(payload);
    }
  };

  const blocker = useBlocker({
    shouldBlockFn: () => isDirty,
    enableBeforeUnload: isDirty,
    withResolver: true,
  });

  return {
    isEditMode,
    isLoadingShifts,
    shiftName,
    setShiftName: (v: string) => { setShiftForm(prev => ({ ...prev, shiftName: v })); setIsDirty(true); },
    shiftSymbol,
    setShiftSymbol: (v: string) => { setShiftForm(prev => ({ ...prev, shiftSymbol: v })); setIsDirty(true); },
    locationId,
    setLocationId,
    employerId,
    setEmployerId,
    handleSubmit,
    isSaving,
    isSaveDisabled,
    isDirty,
    setIsDirty,
    blocker,
    attendanceRules,
    // Intervals — all config is per-interval
    intervalDrafts,
    addInterval,
    removeInterval,
    updateIntervalDraft,
    isCorrection,
    setIsCorrection: (v: boolean) => { setShiftForm(prev => ({ ...prev, isCorrection: v })); setIsDirty(true); },
    correctionEffectiveDate,
    setCorrectionEffectiveDate: (v: string) => { setShiftForm(prev => ({ ...prev, correctionEffectiveDate: v })); setIsDirty(true); },
    changeReason,
    setChangeReason: (v: string) => { setShiftForm(prev => ({ ...prev, changeReason: v })); setIsDirty(true); },
  };
}
