
/* eslint-disable complexity, max-lines-per-function, max-lines */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useBlocker, useNavigate } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { Shift, CreateShiftRequest, UpdateShiftRequest } from "@/lib/api/types";
import type { components } from "@/types/api-generated";

type CreateIntervalRequest = components["schemas"]["CreateIntervalRequest"];

import { useToast } from "@/components/ui/toast";
import {
  getCurrentWorkspaceId,
  fetchShifts,
  createShift,
  updateShift,
  fetchDefaultAttendanceRules,
} from "@/lib/api";

export interface IntervalDraft {
  allowEarlyOt: boolean;
  breakDuration: number;
  breakEnd: string;
  breakStart: string;
  endTime: string;
  gracePeriodMinutes: number;
  intervalName: string;
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  isNightShift: boolean;
  key: string;
  maxEarlyOtMinutes: number | null;
  otAggregationMode: "aggregate" | "per_segment";
  startTime: string;
}

let _intervalCounter = 0;
function nextIntervalKey(): string {
  _intervalCounter += 1;
  return `iv_${Date.now()}_${_intervalCounter}`;
}

function makeDefaultInterval(): IntervalDraft {
  return {
    key: nextIntervalKey(),
    intervalName: "",
    startTime: "08:00",
    endTime: "17:00",
    breakDuration: 60,
    breakStart: "",
    breakEnd: "",
    isFixedBreak: false,
    isAutoDeduct: true,
    isBreakOvertime: false,
    gracePeriodMinutes: 15,
    allowEarlyOt: false,
    maxEarlyOtMinutes: null,
    otAggregationMode: "aggregate",
    isNightShift: false,
  };
}

export function useShiftEditor(shiftId?: string) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId() ?? "";

  const isEditMode = !!shiftId;

  // Form State — shift envelope only
  const [shiftName, setShiftName] = useState("");
  const [shiftSymbol, setShiftSymbol] = useState("");
  const [isCorrection, setIsCorrection] = useState(false);
  const [correctionEffectiveDate, setCorrectionEffectiveDate] = useState(format(new Date(), 'yyyy-MM-dd'));
  const [locationId, setLocationIdState] = useState<string | null>(null);
  const [employerId, setEmployerIdState] = useState<string | null>(null);

  const setLocationId = (v: string | null) => {
    setLocationIdState(v);
    setIsDirty(true);
  };

  const setEmployerId = (v: string | null) => {
    setEmployerIdState(v);
    setIsDirty(true);
  };

  // Interval state — all time/break/policy lives here
  const [intervalDrafts, setIntervalDrafts] = useState<IntervalDraft[]>([makeDefaultInterval()]);

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
    setIntervalDrafts(prev => prev.map(iv => iv.key === key ? { ...iv, ...patch } : iv));
    setIsDirty(true);
  };

  const [isDirty, setIsDirty] = useState(false);

  // Fetch all shifts to find editing shift
  const { data: shifts, isLoading: isLoadingShifts } = useQuery<Shift[]>({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId && isEditMode,
  });

  const { data: workspaceRules } = useQuery({
    queryKey: ["attendance-rules", wsId],
    queryFn: () => (wsId ? fetchDefaultAttendanceRules(wsId) : Promise.reject(new Error("No workspace ID"))),
    enabled: !!wsId,
  });

  const editingShift = shifts?.find((s) => s.id === shiftId) ?? null;

  // Initialize form state when editingShift is loaded
  /* eslint-disable react-hooks/set-state-in-effect */
  useEffect(() => {
    if (isEditMode && editingShift) {
      setShiftName(editingShift.name);
      setShiftSymbol((editingShift as Record<string, unknown>).report_symbol as string | undefined ?? "");
      setLocationIdState((editingShift as Record<string, unknown>).location_id as string | null | undefined ?? null);
      setEmployerIdState((editingShift as Record<string, unknown>).employer_id as string | null | undefined ?? null);
      setIsCorrection(false);
      setCorrectionEffectiveDate(format(new Date(), 'yyyy-MM-dd'));

      // Load intervals from shift — if present, use them; otherwise create default
      const intervals = (editingShift as Record<string, unknown>).intervals as Record<string, unknown>[] | undefined;
      if (intervals && intervals.length > 0) {
        setIntervalDrafts(intervals.map((iv: Record<string, unknown>) => ({
          key: nextIntervalKey(),
          intervalName: (iv.interval_name as string | undefined) ?? "",
          startTime: ((iv.start_time as string | undefined) ?? "08:00").substring(0, 5),
          endTime: ((iv.end_time as string | undefined) ?? "17:00").substring(0, 5),
          breakDuration: (iv.break_duration_minutes as number | undefined) ?? 60,
          breakStart: ((iv.break_start as string | undefined) ?? "").substring(0, 5),
          breakEnd: ((iv.break_end as string | undefined) ?? "").substring(0, 5),
          isFixedBreak: !!(iv.break_start as string | undefined),
          isAutoDeduct: (iv.is_break_auto_deduct as boolean | undefined) ?? true,
          isBreakOvertime: (iv.break_is_overtime as boolean | undefined) ?? false,
          gracePeriodMinutes: (iv.early_punch_in_grace_period_minutes as number | undefined) ?? 15,
          allowEarlyOt: (iv.allow_early_ot as boolean | undefined) ?? false,
          maxEarlyOtMinutes: (iv.max_early_ot_minutes as number | null | undefined) ?? null,
          otAggregationMode: (iv.ot_aggregation_mode as "aggregate" | "per_segment" | undefined) ?? "aggregate",
          isNightShift: (iv.is_night_shift as boolean | undefined) ?? false,
        })));
      }
      setIsDirty(false);
    }
  }, [editingShift, isEditMode]);
  /* eslint-enable react-hooks/set-state-in-effect */

  // Mutations
  const createShiftMutation = useMutation<Shift, Error, CreateShiftRequest>({
    mutationFn: (data) => createShift(wsId, data),
    onSuccess: (data) => {
      toast.success(t("shifts.messages.create_shift_success", { name: data.name }));
      void queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
      setIsDirty(false);
      void navigate({ to: "/shifts", search: { tab: "directory" } });
    },
    onError: (error) => {
      toast.error(error.message, t("shifts.messages.create_shift_error"));
    },
  });

  const updateShiftMutation = useMutation<Shift, Error, { id: string; data: UpdateShiftRequest }>({
    mutationFn: ({ id, data }) => updateShift(wsId, id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_shift_success"));
      void queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
      setIsDirty(false);
      void navigate({ to: "/shifts", search: { tab: "directory" } });
    },
    onError: (error) => {
      toast.error(error.message, t("shifts.messages.update_shift_error"));
    },
  });

  const isSaving = createShiftMutation.isPending || updateShiftMutation.isPending;
  const hasValidIntervals = intervalDrafts.length > 0 && intervalDrafts.every(iv => iv.startTime && iv.endTime);
  const isSaveDisabled = isSaving || !shiftName || !hasValidIntervals;

  const padTime = (t: string) => t.length === 5 ? t + ":00" : t;

  const buildPayload = (): CreateShiftRequest => {
    const intervals: CreateIntervalRequest[] = intervalDrafts.map(iv => ({
      interval_name: iv.intervalName || null,
      start_time: padTime(iv.startTime),
      end_time: padTime(iv.endTime),
      break_duration_minutes: iv.breakDuration,
      break_start: iv.isFixedBreak && iv.breakStart ? padTime(iv.breakStart) : null,
      break_end: iv.isFixedBreak && iv.breakEnd ? padTime(iv.breakEnd) : null,
      is_break_auto_deduct: iv.isAutoDeduct,
      break_is_overtime: iv.isBreakOvertime,
      early_punch_in_grace_period_minutes: iv.gracePeriodMinutes,
      allow_early_ot: iv.allowEarlyOt,
      max_early_ot_minutes: iv.allowEarlyOt ? iv.maxEarlyOtMinutes : null,
      ot_aggregation_mode: iv.otAggregationMode,
      is_night_shift: iv.isNightShift,
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
      };
      updateShiftMutation.mutate({ id: shiftId, data: updatePayload });
    } else {
      createShiftMutation.mutate(payload);
    }
  };

  const blocker = useBlocker({
    shouldBlockFn: () => isDirty,
    withResolver: true,
  });

  return {
    wsId,
    isEditMode,
    isLoadingShifts,
    shiftName,
    setShiftName: (v: string) => { setShiftName(v); setIsDirty(true); },
    shiftSymbol,
    setShiftSymbol: (v: string) => { setShiftSymbol(v); setIsDirty(true); },
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
    workspaceRules,
    // Intervals — all config is per-interval
    intervalDrafts,
    addInterval,
    removeInterval,
    updateIntervalDraft,
    isCorrection,
    setIsCorrection: (v: boolean) => { setIsCorrection(v); setIsDirty(true); },
    correctionEffectiveDate,
    setCorrectionEffectiveDate: (v: string) => { setCorrectionEffectiveDate(v); setIsDirty(true); },
  };
}
