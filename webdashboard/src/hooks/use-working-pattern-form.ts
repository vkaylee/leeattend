import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { CreateWorkingPatternRequest, UpdateWorkingPatternRequest } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import {
  createWorkingPattern,
  fetchPatternDetails,
  fetchShifts,
  fetchWorkingPatterns,
  getCurrentWorkspaceId,
  updateWorkingPattern,
} from "@/lib/api";

import {
  getFirstShiftId,
  initializeGridFromDetails,
  shouldInit,
} from "./use-working-pattern-form-helpers";
import { useWorkingPatternGrid } from "./use-working-pattern-grid";

export { isCycleLengthInvalid, convertGridToDetails } from "./use-working-pattern-form-helpers";

function usePatternGridState() {
  const [patternType, setPatternType] = useState<"weekly" | "rotation">("weekly");
  const [cycleLengthDays, setCycleLengthDays] = useState(7);
  const [isDirty, setIsDirty] = useState(false);

  const {
    grid,
    setGrid,
    addShiftToDay,
    removeShiftFromDay,
    clearAllDays,
    prefillStandard,
  } = useWorkingPatternGrid({
    initialLength: 7,
    onDirty: () => { setIsDirty(true); },
  });

  const handlePatternTypeChange = (type: "weekly" | "rotation") => {
    setPatternType(type);
    setIsDirty(true);
    if (type === "weekly") {
      setGrid((prev) => {
        if (prev.length < 7) {
          return [...prev, ...Array.from({ length: 7 - prev.length }, () => [] as string[])];
        }
        return prev.slice(0, 7);
      });
    } else {
      setGrid((prev) => {
        if (prev.length < cycleLengthDays) {
          return [...prev, ...Array.from({ length: cycleLengthDays - prev.length }, () => [] as string[])];
        }
        return prev.slice(0, cycleLengthDays);
      });
    }
  };

  const handleCycleLengthChange = (len: number) => {
    const validLen = Math.max(1, Math.min(366, len));
    setCycleLengthDays(validLen);
    setIsDirty(true);
    setGrid((prev) => {
      if (prev.length < validLen) {
        return [...prev, ...Array.from({ length: validLen - prev.length }, () => [] as string[])];
      }
      return prev.slice(0, validLen);
    });
  };

  return {
    patternType,
    setPatternType,
    cycleLengthDays,
    setCycleLengthDays,
    grid,
    setGrid,
    isDirty,
    setIsDirty,
    handlePatternTypeChange,
    handleCycleLengthChange,
    addShiftToDay,
    removeShiftFromDay,
    clearAllDays,
    prefillStandard,
  };
}

interface MutationParams {
  patternId?: string;
  setIsDirty: (dirty: boolean) => void;
  t: (key: string) => string;
  wsId: string | null;
}

function useCreatePatternMutation({ wsId, t, setIsDirty }: MutationParams) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const toast = useToast();

  return useMutation({
    mutationFn: (data: CreateWorkingPatternRequest) => createWorkingPattern(wsId ?? "", data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
      setIsDirty(false);
      void navigate({ to: "/shifts/rules", search: { tab: "patterns" } });
    },
    onError: () => {
      toast.error(t("patterns.messages.create_error"));
    },
  });
}

function useUpdatePatternMutation({ wsId, patternId, t, setIsDirty }: MutationParams) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const toast = useToast();

  return useMutation({
    mutationFn: (data: UpdateWorkingPatternRequest) => updateWorkingPattern(wsId ?? "", patternId ?? "", data),
    onSuccess: () => {
      toast.success(t("patterns.messages.update_success"));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
      void queryClient.invalidateQueries({ queryKey: ["pattern-details", wsId ?? "", patternId ?? ""] });
      setIsDirty(false);
      void navigate({ to: "/shifts/rules", search: { tab: "patterns" } });
    },
    onError: () => {
      toast.error(t("patterns.messages.update_error"));
    },
  });
}



function getShiftsQueryOptions(wsId: string | null) {
  return {
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId ?? ""),
    enabled: Boolean(wsId),
  };
}

function getPatternsQueryOptions(wsId: string | null, isEditMode: boolean) {
  return {
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId ?? ""),
    enabled: Boolean(wsId) && isEditMode,
  };
}

function getPatternDetailsQueryOptions(wsId: string | null, patternId: string | undefined, isEditMode: boolean) {
  return {
    queryKey: ["pattern-details", wsId, patternId],
    queryFn: () => (patternId ? fetchPatternDetails(wsId ?? "", patternId) : Promise.resolve([])),
    enabled: Boolean(wsId) && isEditMode,
  };
}

export function useWorkingPatternForm(patternId?: string) {
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId();
  const isEditMode = !!patternId;

  // Form State
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [isInitialized, setIsInitialized] = useState(false);

  const gridState = usePatternGridState();
  const {
    patternType,
    setPatternType,
    cycleLengthDays,
    setCycleLengthDays,
    grid,
    setGrid,
    isDirty,
    setIsDirty,
    handlePatternTypeChange,
    handleCycleLengthChange,
  } = gridState;

  // Fetch queries
  const { data: shifts = [] } = useQuery(getShiftsQueryOptions(wsId));

  const { data: patterns = [], isLoading: isLoadingPatterns } = useQuery(getPatternsQueryOptions(wsId, isEditMode));

  const { data: patternDetails = [], isLoading: isLoadingDetails } = useQuery(getPatternDetailsQueryOptions(wsId, patternId, isEditMode));

  const currentPattern = patterns.find((p) => p.id === patternId);

  // Initialize form state when editing inline during render (allowed for derived/resetting state to prevent effect set-state loops)
  if (currentPattern && shouldInit(isEditMode, currentPattern, isLoadingDetails, isInitialized)) {
    setIsInitialized(true);
    setName(currentPattern.name);
    setDescription(currentPattern.description ?? "");
    setPatternType(currentPattern.pattern_type as "weekly" | "rotation");
    const len = currentPattern.cycle_length_days ?? 7;
    setCycleLengthDays(len);
    setGrid(initializeGridFromDetails(len, patternDetails));
  }

  // Grid actions
  const addShiftToDay = (dayIndex: number, shiftId: string) => {
    gridState.addShiftToDay(dayIndex, shiftId);
  };

  const removeShiftFromDay = (dayIndex: number, shiftId: string) => {
    gridState.removeShiftFromDay(dayIndex, shiftId);
  };

  const clearAllDays = () => {
    gridState.clearAllDays();
  };

  const prefillStandard = () => {
    const firstShift = getFirstShiftId(shifts);
    if (!firstShift) {
      toast.error(t("patterns.messages.no_shifts_available"));
      return;
    }
    gridState.prefillStandard(firstShift);
  };

  // Mutations using extracted hooks
  const createMutation = useCreatePatternMutation({ wsId, t, setIsDirty });
  const updateMutation = useUpdatePatternMutation({ wsId, patternId, t, setIsDirty });

  return {
    wsId,
  // eslint-disable-next-line max-lines
    isEditMode,
    isLoadingPatterns,
    shifts,
    name,
    setName,
    description,
    setDescription,
    patternType,
    handlePatternTypeChange,
    cycleLengthDays,
    handleCycleLengthChange,
    grid,
    addShiftToDay,
    removeShiftFromDay,
    clearAllDays,
    prefillStandard,
    createPattern: createMutation.mutate,
    updatePattern: updateMutation.mutate,
    isDirty,
    setIsDirty,
    isSaving: createMutation.isPending || updateMutation.isPending,
  };
}

