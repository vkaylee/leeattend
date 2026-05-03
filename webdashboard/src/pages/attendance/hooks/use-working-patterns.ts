import { useMutation, useQuery, useQueryClient, UseMutationResult } from "@tanstack/react-query";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format, startOfToday } from "date-fns";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import {
  BulkAssignResponse,
  CreateWorkingPatternRequest,
  Shift,
  WorkingPattern,
  WorkingPatternDetail,
  WorkingPatternDetailRequest,
  bulkAssignPattern,
  createWorkingPattern,
  deleteWorkingPattern,
  fetchPatternDetails,
  fetchShifts,
  fetchWorkingPatterns,
  getCurrentWorkspaceId,
} from "@/lib/api";

type BulkAssignPatternRequestLocal = Parameters<typeof bulkAssignPattern>[1];

export interface UseWorkingPatternsReturn {
  addShiftToDay: (dayIndex: number, shiftId: string) => void;
  assignStartDate: string;
  bulkAssignMutation: UseMutationResult<BulkAssignResponse, Error, BulkAssignPatternRequestLocal>;
  ConfirmationDialog: React.ComponentType<unknown>;
  createMutation: UseMutationResult<WorkingPattern, Error, CreateWorkingPatternRequest>;
  deleteMutation: UseMutationResult<unknown, Error, string>;
  expandedPatternId: string | null;
  handleBulkAssign: () => void;
  handleCreate: () => void;
  handleDelete: (pattern: WorkingPattern) => Promise<void>;
  isBulkModalOpen: boolean;
  isCreateModalOpen: boolean;
  isLoading: boolean;
  newPatternDesc: string;
  newPatternName: string;
  patternDetails: WorkingPatternDetail[];
  patterns: WorkingPattern[];
  prefillStandard: () => void;
  removeShiftFromDay: (dayIndex: number, shiftId: string) => void;
  selectedEmployeeIds: string[];
  selectedPatternId: string;
  setAssignStartDate: (v: string) => void;
  setExpandedPatternId: (v: string | null) => void;
  setIsBulkModalOpen: (v: boolean) => void;
  setIsCreateModalOpen: (v: boolean) => void;
  setNewPatternDesc: (v: string) => void;
  setNewPatternName: (v: string) => void;
  setSelectedEmployeeIds: (v: string[]) => void;
  setSelectedPatternId: (v: string) => void;
  setWeeklyGrid: (v: string[][] | ((p: string[][]) => string[][])) => void;
  shifts: Shift[];
  weeklyGrid: string[][];
  wsId: string;
}

export function useWorkingPatterns(): UseWorkingPatternsReturn {
   
  const { ConfirmationDialog, confirm } = useConfirm();
  const queryClient = useQueryClient();
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId() ?? "";
   

  const [assignStartDate, setAssignStartDate] = useState<string>(
    format(startOfToday(), "yyyy-MM-01")
  );
  const [expandedPatternId, setExpandedPatternId] = useState<string | null>(
    null
  );
  const [isBulkModalOpen, setIsBulkModalOpen] = useState<boolean>(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState<boolean>(false);
  const [newPatternDesc, setNewPatternDesc] = useState<string>("");
  const [newPatternName, setNewPatternName] = useState<string>("");
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");
  const [weeklyGrid, setWeeklyGrid] = useState<string[][]>(
    Array.from({ length: 7 }, () => [])
  );

  const { data: patterns, isLoading } = useQuery<WorkingPattern[]>({
    enabled: !!wsId,
    queryFn: () => fetchWorkingPatterns(wsId),
    queryKey: ["working-patterns", wsId],
  });

  const { data: shifts } = useQuery<Shift[]>({
    enabled: !!wsId,
    queryFn: () => fetchShifts(wsId),
    queryKey: ["shifts", wsId],
  });

  const { data: patternDetails } = useQuery<WorkingPatternDetail[]>({
    enabled: !!wsId && !!expandedPatternId,
    queryFn: () => {
      if (!expandedPatternId) {
        return Promise.resolve([] as WorkingPatternDetail[]);
      }
      return fetchPatternDetails(wsId, expandedPatternId);
    },
    queryKey: ["pattern-details", wsId, expandedPatternId],
  });

  const createMutation = useMutation<WorkingPattern, Error, CreateWorkingPatternRequest>({
    mutationFn: (data: CreateWorkingPatternRequest) =>
      createWorkingPattern(wsId, data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      setIsCreateModalOpen(false);
      setNewPatternName("");
      setNewPatternDesc("");
      setWeeklyGrid(Array.from({ length: 7 }, () => []));

      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
  });

  const bulkAssignMutation = useMutation<BulkAssignResponse, Error, BulkAssignPatternRequestLocal>({
    mutationFn: (data: BulkAssignPatternRequestLocal) =>
      bulkAssignPattern(wsId, data),
    onSuccess: (data: BulkAssignResponse) => {
      if (data.success_count > 0) {
        toast.success(
          t("patterns.messages.bulk_assign_success_count", {
            count: data.success_count,
          })
        );
        setIsBulkModalOpen(false);
        setSelectedEmployeeIds([]);
        void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      } else if (data.failed_employees.length > 0) {
        toast.error(
          t("patterns.messages.bulk_assign_conflict", {
            count: data.failed_employees.length,
          })
        );
      }
    },
  });

  const deleteMutation = useMutation<unknown, Error, string>({
    mutationFn: (patternId: string) => deleteWorkingPattern(wsId, patternId),
    onSuccess: () => {
      toast.success(t("patterns.messages.delete_success"));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
  });

  const addShiftToDay = (dayIndex: number, shiftId: string) => {
    setWeeklyGrid((previous) => {
      const newGrid = previous.map((array) => [...array]);
      if (!newGrid[dayIndex].includes(shiftId)) {
        newGrid[dayIndex] = [...newGrid[dayIndex], shiftId];
      }
      return newGrid;
    });
  };

  const removeShiftFromDay = (dayIndex: number, shiftId: string) => {
    setWeeklyGrid((previous) => {
      const newGrid = previous.map((array) => [...array]);
      newGrid[dayIndex] = newGrid[dayIndex].filter((id) => id !== shiftId);
      return newGrid;
    });
  };

  const handleDelete = async (pattern: WorkingPattern) => {
    const ok = await confirm({
      message: t("patterns.confirm_delete", { name: pattern.name }),
      title: t("common.warning", { defaultValue: "Warning" }),
      variant: "danger",
    });
    if (ok) {
      deleteMutation.mutate(pattern.id);
    }
  };

  const handleCreate = () => {
    if (!newPatternName) {
      toast.warning(t("patterns.validation_name_required"));
      return;
    }
    const details: WorkingPatternDetailRequest[] = [];
    weeklyGrid.forEach((shiftIds, index) => {
      if (shiftIds.length > 0) {
        shiftIds.forEach((sid) => {
          details.push({ day_of_week: index, shift_id: sid });
        });
      } else {
        details.push({ day_of_week: index, shift_id: null });
      }
    });

    createMutation.mutate({
      description: newPatternDesc,
      details,
      name: newPatternName,
    });
  };

  const handleBulkAssign = () => {
    if (!selectedPatternId || selectedEmployeeIds.length === 0) {
      return;
    }
    bulkAssignMutation.mutate({
      employee_ids: selectedEmployeeIds,
      pattern_id: selectedPatternId,
      start_date: assignStartDate,
    });
  };

  const prefillStandard = () => {
    if (!shifts || shifts.length === 0) {
      return;
    }
    const firstShift = shifts[0];
    const defaultShiftId = firstShift.id;
    setWeeklyGrid((prev) => {
        const newGrid = prev.map((array) => [...array]);
        for (let index = 1; index <= 5; index++) {
          newGrid[index] = [defaultShiftId];
        }
        return newGrid;
      });
  };

  return {
    ConfirmationDialog,
    addShiftToDay,
    assignStartDate,
    bulkAssignMutation,
    createMutation,
    deleteMutation,
    expandedPatternId,
    handleBulkAssign,
    handleCreate,
    handleDelete,
    isBulkModalOpen,
    isCreateModalOpen,
    isLoading: isLoading,
    newPatternDesc,
    newPatternName,
    patternDetails: patternDetails ?? [],
    patterns: patterns ?? [],
    prefillStandard,
    removeShiftFromDay,
    selectedEmployeeIds,
    selectedPatternId,
    setAssignStartDate,
    setExpandedPatternId,
    setIsBulkModalOpen,
    setIsCreateModalOpen,
    setNewPatternDesc,
    setNewPatternName,
    setSelectedEmployeeIds,
    setSelectedPatternId,
    setWeeklyGrid,
    shifts: shifts ?? [],
    weeklyGrid,
    wsId,
  };
}
