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
  updateWorkingPattern,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";

import { getFirstShiftId, initializeGridFromDetails, shouldInit } from "./use-working-pattern-form-helpers";
import { useWorkingPatternGrid } from "./use-working-pattern-grid";

function resizeGrid(grid: string[][], length: number): string[][] {
  return grid.length < length
    ? [...grid, ...Array.from({ length: length - grid.length }, () => [] as string[])]
    : grid.slice(0, length);
}

function usePatternGridState() {
  const [patternType, setPatternType] = useState<"weekly" | "rotation">("weekly");
  const [cycleLengthDays, setCycleLengthDays] = useState(7);
  const [isDirty, setIsDirty] = useState(false);
  const gridState = useWorkingPatternGrid({
    initialLength: 7,
    onDirty: () => {
      setIsDirty(true);
    },
  });

  const handlePatternTypeChange = (type: "weekly" | "rotation") => {
    setPatternType(type);
    setIsDirty(true);
    const length = type === "weekly" ? 7 : cycleLengthDays;
    gridState.setGrid((grid) => resizeGrid(grid, length));
  };

  const handleCycleLengthChange = (length: number) => {
    const validLength = Math.max(1, Math.min(366, length));
    setCycleLengthDays(validLength);
    setIsDirty(true);
    gridState.setGrid((grid) => resizeGrid(grid, validLength));
  };

  return { ...gridState, patternType, setPatternType, cycleLengthDays, setCycleLengthDays, isDirty, setIsDirty, handlePatternTypeChange, handleCycleLengthChange };
}

interface MutationParams {
  patternId?: string;
  setIsDirty: (dirty: boolean) => void;
  t: (key: string) => string;
}

function useCreatePatternMutation({ t, setIsDirty }: MutationParams) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const toast = useToast();
  return useMutation({
    mutationFn: (data: CreateWorkingPatternRequest) => createWorkingPattern(data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("working-patterns") });
      setIsDirty(false);
      void navigate({ to: "/shifts/rules", search: { tab: "patterns" } });
    },
    onError: () => {
      toast.error(t("patterns.messages.create_error"));
    },
  });
}

function useUpdatePatternMutation({ patternId, t, setIsDirty }: MutationParams) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const toast = useToast();
  return useMutation({
    mutationFn: (data: UpdateWorkingPatternRequest) => updateWorkingPattern(patternId ?? "", data),
    onSuccess: () => {
      toast.success(t("patterns.messages.update_success"));
      void queryClient.invalidateQueries({ queryKey: createQueryKey("working-patterns") });
      void queryClient.invalidateQueries({ queryKey: createQueryKey("pattern-details", patternId ?? "") });
      setIsDirty(false);
      void navigate({ to: "/shifts/rules", search: { tab: "patterns" } });
    },
    onError: () => {
      toast.error(t("patterns.messages.update_error"));
    },
  });
}

function getShiftsQueryOptions() {
  return { queryKey: ["shifts"], queryFn: () => fetchShifts() };
}

function getPatternsQueryOptions(isEditMode: boolean) {
  return { queryKey: ["working-patterns"], queryFn: fetchWorkingPatterns, enabled: isEditMode };
}

function getPatternDetailsQueryOptions(patternId: string | undefined, isEditMode: boolean) {
  return {
    queryKey: ["pattern-details", patternId],
    queryFn: () => (patternId ? fetchPatternDetails(patternId) : Promise.resolve([])),
    enabled: isEditMode,
  };
}

export function useWorkingPatternForm(patternId?: string) {
  const { t } = useTranslation();
  const toast = useToast();
  const isEditMode = Boolean(patternId);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [isInitialized, setIsInitialized] = useState(false);
  const gridState = usePatternGridState();
  const { data: shifts = [] } = useQuery(getShiftsQueryOptions());
  const { data: patterns = [], isLoading: isLoadingPatterns } = useQuery(getPatternsQueryOptions(isEditMode));
  const { data: patternDetails = [], isLoading: isLoadingDetails } = useQuery(getPatternDetailsQueryOptions(patternId, isEditMode));
  const currentPattern = patterns.find((pattern) => pattern.id === patternId);

  if (currentPattern && shouldInit(isEditMode, currentPattern, isLoadingDetails, isInitialized)) {
    setIsInitialized(true);
    setName(currentPattern.name);
    setDescription(currentPattern.description ?? "");
    setPatternTypeFromPattern(gridState, currentPattern.pattern_type);
    const length = currentPattern.cycle_length_days ?? 7;
    gridState.setCycleLengthDays(length);
    gridState.setGrid(initializeGridFromDetails(length, patternDetails));
  }

  const prefillStandard = () => {
    const firstShift = getFirstShiftId(shifts);
    if (!firstShift) {
      toast.error(t("patterns.messages.no_shifts_available"));
      return;
    }
    gridState.prefillStandard(firstShift);
  };
  const createMutation = useCreatePatternMutation({ t, setIsDirty: gridState.setIsDirty });
  const updateMutation = useUpdatePatternMutation({ patternId, t, setIsDirty: gridState.setIsDirty });

  return {
    isEditMode,
    isLoadingPatterns,
    shifts,
    name,
    setName,
    description,
    setDescription,
    patternType: gridState.patternType,
    handlePatternTypeChange: gridState.handlePatternTypeChange,
    cycleLengthDays: gridState.cycleLengthDays,
    handleCycleLengthChange: gridState.handleCycleLengthChange,
    grid: gridState.grid,
    addShiftToDay: gridState.addShiftToDay,
    removeShiftFromDay: gridState.removeShiftFromDay,
    clearAllDays: gridState.clearAllDays,
    prefillStandard,
    createPattern: createMutation.mutate,
    updatePattern: updateMutation.mutate,
    isDirty: gridState.isDirty,
    setIsDirty: gridState.setIsDirty,
    isSaving: createMutation.isPending || updateMutation.isPending,
  };
}

function setPatternTypeFromPattern(
  gridState: ReturnType<typeof usePatternGridState>,
  patternType: string,
): void {
  gridState.setPatternType(patternType as "weekly" | "rotation");
}

export type { CreateWorkingPatternRequest, UpdateWorkingPatternRequest };
