 
/* eslint-disable complexity, max-lines-per-function, max-lines */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useBlocker, useNavigate } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import type { Shift, CreateShiftRequest, UpdateShiftRequest } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import {
  getCurrentWorkspaceId,
  fetchShifts,
  createShift,
  updateShift,
} from "@/lib/api";

export function useShiftEditor(shiftId?: string) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId() ?? "";

  const isEditMode = !!shiftId;

  // Form State
  const [shiftName, setShiftName] = useState("");
  const [shiftSymbol, setShiftSymbol] = useState("");
  const [shiftStart, setShiftStart] = useState("08:00");
  const [shiftEnd, setShiftEnd] = useState("17:00");
  const [shiftBreak, setShiftBreak] = useState(60);
  const [shiftGrace, setShiftGrace] = useState(15);
  const [breakStart, setBreakStart] = useState("");
  const [breakEnd, setBreakEnd] = useState("");
  const [isFixedBreak, setIsFixedBreakState] = useState(false);
  const [isAutoDeduct, setIsAutoDeduct] = useState(true);
  const [isBreakOvertime, setIsBreakOvertime] = useState(false);
  const [allowEarlyOt, setAllowEarlyOt] = useState(false);
  const [maxEarlyOtMinutes, setMaxEarlyOtMinutes] = useState(60);
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

  const [isDirty, setIsDirty] = useState(false);

  const setIsFixedBreak = (val: boolean) => {
    setIsFixedBreakState(val);
    setIsDirty(true);
    if (!val && isBreakOvertime) {
      setIsBreakOvertime(false);
      setIsAutoDeduct(true);
    }
  };

  // Fetch all shifts to find editing shift
  const { data: shifts, isLoading: isLoadingShifts } = useQuery<Shift[]>({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId && isEditMode,
  });

  const editingShift = shifts?.find((s) => s.id === shiftId) ?? null;

  // Initialize form state when editingShift is loaded
  /* eslint-disable react-hooks/set-state-in-effect */
  useEffect(() => {
    if (isEditMode && editingShift) {
      setShiftName(editingShift.name);
      setShiftSymbol(editingShift.report_symbol ?? "");
      setShiftStart(editingShift.start_time.substring(0, 5));
      setShiftEnd(editingShift.end_time.substring(0, 5));
      setShiftBreak(editingShift.break_duration_minutes);
      setShiftGrace(editingShift.early_punch_in_grace_period_minutes);
      setBreakStart(editingShift.break_start?.substring(0, 5) ?? "");
      setBreakEnd(editingShift.break_end?.substring(0, 5) ?? "");
      setIsFixedBreakState(!!editingShift.break_start);
      setIsAutoDeduct(editingShift.is_break_auto_deduct);
      setIsBreakOvertime(editingShift.break_is_overtime);
      setAllowEarlyOt(editingShift.allow_early_ot);
      setMaxEarlyOtMinutes(editingShift.max_early_ot_minutes ?? 60);
      setLocationIdState(editingShift.location_id ?? null);
      setEmployerIdState(editingShift.employer_id ?? null);
      setIsCorrection(false);
      setCorrectionEffectiveDate(format(new Date(), 'yyyy-MM-dd'));
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
  const isSaveDisabled = isSaving || !shiftName || !shiftStart || !shiftEnd || (allowEarlyOt && (!maxEarlyOtMinutes || maxEarlyOtMinutes <= 0));

  const buildPayload = () => {
    const base = {
      name: shiftName,
      report_symbol: shiftSymbol || null,
      start_time: shiftStart + (shiftStart.length === 5 ? ":00" : ""),
      end_time: shiftEnd + (shiftEnd.length === 5 ? ":00" : ""),
      break_duration_minutes: shiftBreak,
      early_punch_in_grace_period_minutes: shiftGrace,
      break_is_overtime: isBreakOvertime,
      allow_early_ot: allowEarlyOt,
      max_early_ot_minutes: allowEarlyOt ? maxEarlyOtMinutes : null,
      is_break_auto_deduct: isAutoDeduct,
      location_id: locationId,
      employer_id: employerId,
    };

    if (isFixedBreak && breakStart && breakEnd) {
      return {
        ...base,
        break_start: breakStart + (breakStart.length === 5 ? ":00" : ""),
        break_end: breakEnd + (breakEnd.length === 5 ? ":00" : ""),
      };
    }

    return {
      ...base,
      break_start: null,
      break_end: null,
    };
  };

  const handleSubmit = (e?: React.SyntheticEvent) => {
    if (e) e.preventDefault();
    if (isSaveDisabled) return;

    const payload = buildPayload();

    if (isEditMode && shiftId) {
      updateShiftMutation.mutate({
        id: shiftId,
        data: {
          ...payload,
          is_correction: isCorrection,
          effective_date: isCorrection ? undefined : correctionEffectiveDate,
        },
      });
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
    shiftStart,
    setShiftStart: (v: string) => { setShiftStart(v); setIsDirty(true); },
    shiftEnd,
    setShiftEnd: (v: string) => { setShiftEnd(v); setIsDirty(true); },
    shiftBreak,
    setShiftBreak: (v: number) => { setShiftBreak(v); setIsDirty(true); },
    shiftGrace,
    setShiftGrace: (v: number) => { setShiftGrace(v); setIsDirty(true); },
    breakStart,
    setBreakStart: (v: string) => { setBreakStart(v); setIsDirty(true); },
    breakEnd,
    setBreakEnd: (v: string) => { setBreakEnd(v); setIsDirty(true); },
    isFixedBreak,
    setIsFixedBreak,
    isAutoDeduct,
    setIsAutoDeduct: (v: boolean) => { setIsAutoDeduct(v); setIsDirty(true); },
    isBreakOvertime,
    setIsBreakOvertime: (v: boolean) => { setIsBreakOvertime(v); setIsDirty(true); },
    allowEarlyOt,
    setAllowEarlyOt: (v: boolean) => { setAllowEarlyOt(v); setIsDirty(true); },
    maxEarlyOtMinutes,
    setMaxEarlyOtMinutes: (v: number) => { setMaxEarlyOtMinutes(v); setIsDirty(true); },
    isCorrection,
    setIsCorrection: (v: boolean) => { setIsCorrection(v); setIsDirty(true); },
    correctionEffectiveDate,
    setCorrectionEffectiveDate: (v: string) => { setCorrectionEffectiveDate(v); setIsDirty(true); },
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
  };
}
