import { useMutation, useQuery, useQueryClient, UseMutationResult } from "@tanstack/react-query";
import React, { useState, useCallback } from "react";
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

function useWorkingPatternQueries(wsId: string, expandedPatternId: string | null) {
  const { data: patterns, isLoading } = useQuery<WorkingPattern[]>({ enabled: !!wsId, queryFn: () => fetchWorkingPatterns(wsId), queryKey: ["working-patterns", wsId] });
  const { data: shifts } = useQuery<Shift[]>({ enabled: !!wsId, queryFn: () => fetchShifts(wsId), queryKey: ["shifts", wsId] });
  const { data: patternDetails } = useQuery<WorkingPatternDetail[]>({ enabled: !!wsId && !!expandedPatternId, queryFn: () => expandedPatternId ? fetchPatternDetails(wsId, expandedPatternId) : Promise.resolve([]), queryKey: ["pattern-details", wsId, expandedPatternId] });
  return { patterns: patterns ?? [], shifts: shifts ?? [], patternDetails: patternDetails ?? [], isLoading };
}

function useWorkingPatternMutations(wsId: string, setters: { setIsCreateModalOpen: (v: boolean) => void, setNewPatternName: (v: string) => void, setNewPatternDesc: (v: string) => void, setWeeklyGrid: (v: string[][]) => void, setIsBulkModalOpen: (v: boolean) => void, setSelectedEmployeeIds: (v: string[]) => void }) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const createMutation = useMutation<WorkingPattern, Error, CreateWorkingPatternRequest>({
    mutationFn: (data) => createWorkingPattern(wsId, data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      setters.setIsCreateModalOpen(false); setters.setNewPatternName(""); setters.setNewPatternDesc(""); setters.setWeeklyGrid(Array.from({ length: 7 }, () => []));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
  });

  const bulkAssignMutation = useMutation<BulkAssignResponse, Error, BulkAssignPatternRequestLocal>({
    mutationFn: (data) => bulkAssignPattern(wsId, data),
    onSuccess: (data) => {
      if (data.success_count > 0) {
        toast.success(t("patterns.messages.bulk_assign_success_count", { count: data.success_count }));
        setters.setIsBulkModalOpen(false); setters.setSelectedEmployeeIds([]);
        void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      } else if (data.failed_employees.length > 0) {
        toast.error(t("patterns.messages.bulk_assign_conflict", { count: data.failed_employees.length }));
      }
    },
  });

  const deleteMutation = useMutation<unknown, Error, string>({
    mutationFn: (patternId) => deleteWorkingPattern(wsId, patternId),
    onSuccess: () => {
      toast.success(t("patterns.messages.delete_success"));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
  });

  return { createMutation, bulkAssignMutation, deleteMutation };
}

export function useWorkingPatterns(): UseWorkingPatternsReturn {
  const { ConfirmationDialog, confirm } = useConfirm();
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId() ?? "";

  const [assignStartDate, setAssignStartDate] = useState(format(startOfToday(), "yyyy-MM-01"));
  const [expandedPatternId, setExpandedPatternId] = useState<string | null>(null);
  const [isBulkModalOpen, setIsBulkModalOpen] = useState(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newPatternDesc, setNewPatternDesc] = useState("");
  const [newPatternName, setNewPatternName] = useState("");
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [selectedPatternId, setSelectedPatternId] = useState("");
  const [weeklyGrid, setWeeklyGrid] = useState<string[][]>(Array.from({ length: 7 }, () => []));

  const { patterns, shifts, patternDetails, isLoading } = useWorkingPatternQueries(wsId, expandedPatternId);
  const { createMutation, bulkAssignMutation, deleteMutation } = useWorkingPatternMutations(wsId, { setIsCreateModalOpen, setNewPatternName, setNewPatternDesc, setWeeklyGrid, setIsBulkModalOpen, setSelectedEmployeeIds });

  const addShiftToDay = useCallback((dayIndex: number, shiftId: string) => {
    setWeeklyGrid((prev) => {
      const newGrid = prev.map((a) => [...a]);
      if (!newGrid[dayIndex].includes(shiftId)) newGrid[dayIndex] = [...newGrid[dayIndex], shiftId];
      return newGrid;
    });
  }, []);

  const removeShiftFromDay = useCallback((dayIndex: number, shiftId: string) => {
    setWeeklyGrid((prev) => {
      const newGrid = prev.map((a) => [...a]);
      newGrid[dayIndex] = newGrid[dayIndex].filter((id) => id !== shiftId);
      return newGrid;
    });
  }, []);

  const handleDelete = async (pattern: WorkingPattern) => {
    if (await confirm({ message: t("patterns.confirm_delete", { name: pattern.name }), title: t("common.warning"), variant: "danger" })) {
      deleteMutation.mutate(pattern.id);
    }
  };

  const handleCreate = () => {
    if (!newPatternName) { toast.warning(t("patterns.validation_name_required")); return; }
    const details: WorkingPatternDetailRequest[] = weeklyGrid.flatMap<WorkingPatternDetailRequest>((shiftIds, index) => 
      shiftIds.length > 0 
        ? shiftIds.map((sid) => ({ day_of_week: index, shift_id: sid })) 
        : [{ day_of_week: index, shift_id: null }]
    );
    createMutation.mutate({ description: newPatternDesc, details, name: newPatternName });
  };

  const handleBulkAssign = () => {
    if (selectedPatternId && selectedEmployeeIds.length > 0) {
      bulkAssignMutation.mutate({ employee_ids: selectedEmployeeIds, pattern_id: selectedPatternId, start_date: assignStartDate });
    }
  };

  const prefillStandard = () => {
    if (!shifts.length) return;
    const defaultShiftId = shifts[0].id;
    setWeeklyGrid((prev) => {
      const newGrid = prev.map((a) => [...a]);
      for (let i = 1; i <= 5; i++) newGrid[i] = [defaultShiftId];
      return newGrid;
    });
  };

  return { ConfirmationDialog, addShiftToDay, assignStartDate, bulkAssignMutation, createMutation, deleteMutation, expandedPatternId, handleBulkAssign, handleCreate, handleDelete, isBulkModalOpen, isCreateModalOpen, isLoading, newPatternDesc, newPatternName, patternDetails, patterns, prefillStandard, removeShiftFromDay, selectedEmployeeIds, selectedPatternId, setAssignStartDate, setExpandedPatternId, setIsBulkModalOpen, setIsCreateModalOpen, setNewPatternDesc, setNewPatternName, setSelectedEmployeeIds, setSelectedPatternId, setWeeklyGrid, shifts, weeklyGrid, wsId };
}
