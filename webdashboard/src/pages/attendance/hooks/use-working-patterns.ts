/* eslint-disable max-lines -- Hook module: 320 lines with multiple internal sub-hooks and patterns; splitting would diffract coupling */
import { useMutation, useQuery, useQueryClient, UseMutationResult } from "@tanstack/react-query";
import React, { useState, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";

import { format, startOfToday } from "date-fns";

import type { ComplianceViolation } from "@/lib/api/errors";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useWorkingPatternGrid } from "@/hooks/use-working-pattern-grid";
import {
  BulkAssignResponse,
  CreateWorkingPatternRequest,
  Department,
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
  fetchEmployees,
  fetchDepartments,
} from "@/lib/api";
import { ApiError, ComplianceConflictError, parseComplianceConflict } from "@/lib/api/errors";

type BulkAssignPatternRequestLocal = Parameters<typeof bulkAssignPattern>[1];

export interface UseWorkingPatternsReturn {
  addShiftToDay: (dayIndex: number, shiftId: string) => void;
  assignAnchorDate: string;
  assignStartDate: string;
  bulkAssignMutation: UseMutationResult<BulkAssignResponse, Error, BulkAssignPatternRequestLocal>;
  complianceModalOpen: boolean;
  complianceViolations: ComplianceViolation[];
  ConfirmationDialog: React.ComponentType<unknown>;
  createMutation: UseMutationResult<WorkingPattern, Error, CreateWorkingPatternRequest>;
  cycleLengthDays: number;
  deleteMutation: UseMutationResult<unknown, Error, string>;
  departments: Department[];
  expandedPatternId: string | null;
  handleBulkAssign: () => void;
  handleCreate: () => void;
  handleDelete: (pattern: WorkingPattern) => Promise<void>;
  isBulkModalOpen: boolean;
  isComplianceResolving: boolean;
  isCreateModalOpen: boolean;
  isLoading: boolean;
  newPatternDesc: string;
  newPatternName: string;
  onComplianceOverride: (reason: string) => Promise<void>;
  patternDetails: WorkingPatternDetail[];
  patterns: WorkingPattern[];
  patternType: "weekly" | "rotation";
  prefillStandard: () => void;
  removeShiftFromDay: (dayIndex: number, shiftId: string) => void;
  selectedEmployeeIds: string[];
  selectedPatternId: string;
  setAssignAnchorDate: (v: string) => void;
  setAssignStartDate: (v: string) => void;
  setComplianceModalOpen: (v: boolean) => void;
  setCycleLengthDays: (v: number) => void;
  setExpandedPatternId: (v: string | null) => void;
  setIsBulkModalOpen: (v: boolean) => void;
  setIsCreateModalOpen: (v: boolean) => void;
  setNewPatternDesc: (v: string) => void;
  setNewPatternName: (v: string) => void;
  setPatternType: (v: "weekly" | "rotation") => void;
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

function useWorkingPatternMutations(
  wsId: string,
  setters: {
    setIsCreateModalOpen: (v: boolean) => void;
    setNewPatternName: (v: string) => void;
    setNewPatternDesc: (v: string) => void;
    setWeeklyGrid: (v: string[][]) => void;
    setIsBulkModalOpen: (v: boolean) => void;
    setSelectedEmployeeIds: (v: string[]) => void;
    setPatternType: (v: "weekly" | "rotation") => void;
    setCycleLengthDays: (v: number) => void;
    setComplianceModalOpen: (v: boolean) => void;
    setComplianceViolations: (v: ComplianceViolation[]) => void;
  }
) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const createMutation = useMutation<WorkingPattern, Error, CreateWorkingPatternRequest>({
    mutationFn: (data) => createWorkingPattern(wsId, data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      setters.setIsCreateModalOpen(false);
      setters.setNewPatternName("");
      setters.setNewPatternDesc("");
      setters.setWeeklyGrid(Array.from({ length: 7 }, () => []));
      setters.setPatternType("weekly");
      setters.setCycleLengthDays(6);
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
    onError: (error: Error) => {
      // Detect 409 compliance conflict and surface the override modal.
      if (error instanceof ComplianceConflictError) {
        setters.setComplianceViolations(error.violations);
        setters.setComplianceModalOpen(true);
        return;
      }
      if (error instanceof ApiError && error.isConflict) {
        const retry = parseComplianceConflict(error.errorBody);
        if (retry) {
          setters.setComplianceViolations(
            new ComplianceConflictError(error.status, error.statusText, error.errorBody, retry).violations,
          );
          setters.setComplianceModalOpen(true);
          return;
        }
      }
      toast.error(error.message || t("patterns.messages.bulk_assign_error"));
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

// eslint-disable-next-line max-lines-per-function
export function useWorkingPatterns(): UseWorkingPatternsReturn {
  const { ConfirmationDialog, confirm } = useConfirm();
  const { t } = useTranslation();
  const toast = useToast();
  const wsId = getCurrentWorkspaceId() ?? "";

  const [assignStartDate, setAssignStartDate] = useState(format(startOfToday(), "yyyy-MM-01"));
  const [assignAnchorDate, setAssignAnchorDate] = useState(format(startOfToday(), "yyyy-MM-01"));
  const [expandedPatternId, setExpandedPatternId] = useState<string | null>(null);
  const [isBulkModalOpen, setIsBulkModalOpen] = useState(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newPatternDesc, setNewPatternDesc] = useState("");
  const [newPatternName, setNewPatternName] = useState("");
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [selectedPatternId, setSelectedPatternId] = useState("");

  // ── Compliance modal state ────────────────────────────────────────────
  const [complianceModalOpen, setComplianceModalOpen] = useState(false);
  const [complianceViolations, setComplianceViolations] = useState<ComplianceViolation[]>([]);
  const [isComplianceResolving, setIsComplianceResolving] = useState(false);
  const pendingRetryRef = useRef<BulkAssignPatternRequestLocal | null>(null);

  const {
    grid: weeklyGrid,
    setGrid: setWeeklyGrid,
    addShiftToDay,
    removeShiftFromDay,
    prefillStandard: prefillStandardBase,
  } = useWorkingPatternGrid({
    initialLength: 7,
  });

  const [patternType, setPatternType] = useState<"weekly" | "rotation">("weekly");
  const [cycleLengthDays, setCycleLengthDays] = useState<number>(6);

  const handleSetPatternType = useCallback((type: "weekly" | "rotation") => {
    setPatternType(type);
    const targetLength = type === "weekly" ? 7 : cycleLengthDays;
    setWeeklyGrid((prev) => {
      if (prev.length === targetLength) return prev;
      if (prev.length < targetLength) {
        const diff = targetLength - prev.length;
        return [...prev, ...Array.from({ length: diff }, () => [])];
      } else {
        return prev.slice(0, targetLength);
      }
    });
  }, [cycleLengthDays, setWeeklyGrid]);

  const handleSetCycleLengthDays = useCallback((days: number) => {
    setCycleLengthDays(days);
    if (patternType === "rotation") {
      setWeeklyGrid((prev) => {
        if (prev.length === days) return prev;
        if (prev.length < days) {
          const diff = days - prev.length;
          return [...prev, ...Array.from({ length: diff }, () => [])];
        } else {
          return prev.slice(0, days);
        }
      });
    }
  }, [patternType, setWeeklyGrid]);

  const { patterns, shifts, patternDetails, isLoading } = useWorkingPatternQueries(wsId, expandedPatternId);
  const { createMutation, bulkAssignMutation, deleteMutation } = useWorkingPatternMutations(wsId, {
    setIsCreateModalOpen,
    setNewPatternName,
    setNewPatternDesc,
    setWeeklyGrid,
    setIsBulkModalOpen,
    setSelectedEmployeeIds,
    setPatternType: handleSetPatternType,
    setCycleLengthDays: handleSetCycleLengthDays,
    setComplianceModalOpen,
    setComplianceViolations,
  });

  // addShiftToDay and removeShiftFromDay are provided by useWorkingPatternGrid

  const handleDelete = async (pattern: WorkingPattern) => {
    if (await confirm({ message: t("patterns.confirm_delete", { name: pattern.name }), title: t("common.warning"), variant: "danger" })) {
      deleteMutation.mutate(pattern.id);
    }
  };

  const handleCreate = () => {
    if (!newPatternName) { toast.warning(t("patterns.validation_name_required")); return; }
    if (patternType === "rotation") {
      if (cycleLengthDays < 1 || cycleLengthDays > 366) {
        toast.warning(t("patterns.validation_cycle_length_invalid") || "Please enter a valid cycle length (1-366).");
        return;
      }
    }
    const details: WorkingPatternDetailRequest[] = weeklyGrid.flatMap<WorkingPatternDetailRequest>((shiftIds, index) => 
      shiftIds.length > 0 
        ? shiftIds.map((sid) => ({ schedule_day: index, shift_id: sid })) 
        : [{ schedule_day: index, shift_id: null }]
    );
    createMutation.mutate({
      description: newPatternDesc,
      details,
      name: newPatternName,
      pattern_type: patternType,
      cycle_length_days: patternType === "rotation" ? cycleLengthDays : undefined,
    });
  };

  const handleStartDateChange = useCallback((val: string) => {
    setAssignStartDate(val);
    setAssignAnchorDate(val);
  }, []);

  const { data: employeesData } = useQuery({
    queryKey: ["employees-all-bulk", wsId],
    queryFn: () => fetchEmployees(wsId, 1, 1000),
    enabled: !!wsId,
  });
  const employees = employeesData?.data ?? [];

  const { data: departments = [] } = useQuery<Department[]>({
    queryKey: ["departments", wsId],
    queryFn: () => fetchDepartments(wsId),
    enabled: !!wsId,
  });

  const handleBulkAssign = () => {
    if (selectedPatternId && selectedEmployeeIds.length > 0) {
      const selectedPattern = patterns.find((p) => p.id === selectedPatternId);
      const isRotation = selectedPattern?.pattern_type === "rotation";

      const employmentIds = selectedEmployeeIds
        .map(empId => employees.find(e => e.id === empId)?.active_employment_id)
        .filter((id): id is string => !!id);

      const payload: BulkAssignPatternRequestLocal = {
        employment_ids: employmentIds,
        pattern_id: selectedPatternId,
        start_date: assignStartDate,
        anchor_date: isRotation ? assignAnchorDate : undefined,
      };
      pendingRetryRef.current = payload;
      bulkAssignMutation.mutate(payload);
    }
  };

  // ── Compliance override retry ─────────────────────────────────────────
  const onComplianceOverride = useCallback(async (reason: string) => {
    const pending = pendingRetryRef.current;
    if (!pending) return;
    setIsComplianceResolving(true);
    try {
      await bulkAssignMutation.mutateAsync({ ...pending, allow_warnings: true, override_reason: reason });
      setComplianceModalOpen(false);
      pendingRetryRef.current = null;
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setIsComplianceResolving(false);
    }
  }, [bulkAssignMutation, toast]);

  const prefillStandard = () => {
    if (!shifts.length) return;
    prefillStandardBase(shifts[0].id);
  };

  return {
    ConfirmationDialog,
    addShiftToDay,
    assignAnchorDate,
    assignStartDate,
    bulkAssignMutation,
    complianceModalOpen,
    complianceViolations,
    createMutation,
    cycleLengthDays,
    deleteMutation,
    departments,
    expandedPatternId,
    handleBulkAssign,
    handleCreate,
    handleDelete,
    isBulkModalOpen,
    isComplianceResolving,
    isCreateModalOpen,
    isLoading,
    newPatternDesc,
    newPatternName,
    onComplianceOverride,
    patternDetails,
    patternType,
    patterns,
    prefillStandard,
    removeShiftFromDay,
    selectedEmployeeIds,
    selectedPatternId,
    setAssignAnchorDate,
    setAssignStartDate: handleStartDateChange,
    setComplianceModalOpen,
    setCycleLengthDays: handleSetCycleLengthDays,
    setExpandedPatternId,
    setIsBulkModalOpen,
    setIsCreateModalOpen,
    setNewPatternDesc,
    setNewPatternName,
    setPatternType: handleSetPatternType,
    setSelectedEmployeeIds,
    setSelectedPatternId,
    setWeeklyGrid,
    shifts,
    weeklyGrid,
    wsId,
  };
}
