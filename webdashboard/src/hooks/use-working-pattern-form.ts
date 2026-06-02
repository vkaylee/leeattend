import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import {
  createWorkingPattern,
  fetchPatternDetails,
  fetchShifts,
  fetchWorkingPatterns,
  getCurrentWorkspaceId,
  updateWorkingPattern,
} from "@/lib/api";
import { CreateWorkingPatternRequest, UpdateWorkingPatternRequest, WorkingPatternDetailRequest } from "@/lib/api/types";

export function useWorkingPatternForm(patternId?: string) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();
  const isEditMode = !!patternId;

  // Form State
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [patternType, setPatternType] = useState<"weekly" | "rotation">("weekly");
  const [cycleLengthDays, setCycleLengthDays] = useState(6);
  const [grid, setGrid] = useState<string[][]>(Array.from({ length: 7 }, () => []));
  const [isInitialized, setIsInitialized] = useState(false);
  const [isDirty, setIsDirty] = useState(false);

  // Fetch queries
  const { data: shifts = [] } = useQuery({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId ?? ""),
    enabled: !!wsId,
  });

  const { data: patterns = [], isLoading: isLoadingPatterns } = useQuery({
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId ?? ""),
    enabled: !!wsId && isEditMode,
  });

  const { data: patternDetails = [], isLoading: isLoadingDetails } = useQuery({
    queryKey: ["pattern-details", wsId, patternId],
    queryFn: () => (patternId ? fetchPatternDetails(wsId ?? "", patternId) : Promise.resolve([])),
    enabled: !!wsId && isEditMode,
  });

  const currentPattern = patterns.find((p) => p.id === patternId);

  // Initialize form state when editing
  useEffect(() => {
    if (isEditMode && currentPattern && !isLoadingDetails && !isInitialized) {
      setName(currentPattern.name);
      setDescription(currentPattern.description ?? "");
      setPatternType(currentPattern.pattern_type as "weekly" | "rotation");
      const len = currentPattern.cycle_length_days ?? 7;
      setCycleLengthDays(len);

      const newGrid = Array.from({ length: len }, () => [] as string[]);
      patternDetails.forEach((detail) => {
        if (detail.shift_id && detail.schedule_day >= 0 && detail.schedule_day < len) {
          newGrid[detail.schedule_day].push(detail.shift_id);
        }
      });
      setGrid(newGrid);
      setIsInitialized(true);
      setIsDirty(false);
    }
  }, [isEditMode, currentPattern, isLoadingDetails, patternDetails, isInitialized]);

  // Resizing grid when type or cycle length changes
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

  // Grid actions
  const addShiftToDay = (dayIndex: number, shiftId: string) => {
    setIsDirty(true);
    setGrid((prev) => prev.map((shiftsArr, idx) => (idx === dayIndex ? [...shiftsArr, shiftId] : shiftsArr)));
  };

  const removeShiftFromDay = (dayIndex: number, shiftId: string) => {
    setIsDirty(true);
    setGrid((prev) => prev.map((shiftsArr, idx) => (idx === dayIndex ? shiftsArr.filter((id) => id !== shiftId) : shiftsArr)));
  };

  const clearAllDays = () => {
    setIsDirty(true);
    setGrid(Array.from({ length: grid.length }, () => []));
  };

  const prefillStandard = () => {
    setIsDirty(true);
    const firstShift = shifts[0]?.id;
    if (!firstShift) {
      toast.error(t("patterns.messages.no_shifts_available") || "No shifts available to prefill.");
      return;
    }
    setGrid((prev) => prev.map((_, idx) => (idx >= 1 && idx <= 5 ? [firstShift] : [])));
  };

  // Mutations
  const createMutation = useMutation({
    mutationFn: (data: CreateWorkingPatternRequest) => createWorkingPattern(wsId ?? "", data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
      setIsDirty(false);
      void navigate({ to: "/working-patterns" });
    },
    onError: () => {
      toast.error(t("patterns.messages.create_error"));
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateWorkingPatternRequest) => updateWorkingPattern(wsId ?? "", patternId ?? "", data),
    onSuccess: () => {
      toast.success(t("patterns.messages.update_success"));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
      void queryClient.invalidateQueries({ queryKey: ["pattern-details", wsId ?? "", patternId ?? ""] });
      setIsDirty(false);
      void navigate({ to: "/working-patterns" });
    },
    onError: () => {
      toast.error(t("patterns.messages.update_error"));
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      toast.error(t("patterns.validation_name_required"));
      return;
    }
    if (patternType === "rotation" && (cycleLengthDays < 1 || cycleLengthDays > 366)) {
      toast.error(t("patterns.validation_cycle_length_invalid"));
      return;
    }

    const details = grid.flatMap((shiftIds, scheduleDay) => {
      if (shiftIds.length === 0) {
        return [{ schedule_day: scheduleDay, shift_id: null as string | null }];
      }
      return shiftIds.map((shiftId) => ({
        schedule_day: scheduleDay,
        shift_id: shiftId,
      }));
    }) as WorkingPatternDetailRequest[];

    const payload = {
      name: name.trim(),
      description: description.trim() || undefined,
      pattern_type: patternType,
      cycle_length_days: patternType === "rotation" ? cycleLengthDays : undefined,
      details,
    };

    if (isEditMode) {
      updateMutation.mutate(payload);
    } else {
      createMutation.mutate(payload);
    }
  };

  return {
    wsId,
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
    handleSubmit,
    isDirty,
    setIsDirty,
    isSaving: createMutation.isPending || updateMutation.isPending,
  };
}
