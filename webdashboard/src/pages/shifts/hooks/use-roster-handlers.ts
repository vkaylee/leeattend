 
import type { UseMutationResult } from "@tanstack/react-query";

import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";

import { format, addDays, subMonths, startOfMonth, endOfMonth } from "date-fns";

import type { ComplianceViolation } from "@/lib/api/errors";
import type { AssignPatternRequest, Employee, EmployeeSchedule, EmployeeWorkingPattern, EffectiveSchedule } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import { fetchWorkspaceSchedules, bulkUpsertRoster, deleteRoster } from "@/lib/api";
import { ComplianceConflictError } from "@/lib/api/errors";
import { runPool } from "@/lib/async-pool";

// Cap concurrent roster requests so large monthly selections (e.g. 20 employees)
// don't fan out into a burst that exhausts the backend DB pool or trips rate limits.
const ROSTER_REQUEST_CONCURRENCY = 4;

interface UseRosterHandlersProps {
  allSchedules: EmployeeSchedule[] | undefined;
  currentMonth: Date;
  endDate: string;
  isForever: boolean;
  monthDays: Date[];
  mutations: {
    assignPatternMutation: UseMutationResult<EmployeeWorkingPattern, Error, AssignPatternRequest>;
    bulkRosterMutation: UseMutationResult<unknown, Error, { date: string; shift_id: string | null }[]>;
  };
  selectedDays: string[];
  selectedEmployee: Employee | null;
  selectedPatternId: string;
  setComplianceModalOpen: (v: boolean) => void;
  setComplianceViolations: (v: ComplianceViolation[]) => void;
  setIsComplianceResolving: (v: boolean) => void;
  startDate: string;
  startOfWeek: Date;
  weekDays: Date[];
  wsId: string;
}

 
// eslint-disable-next-line max-lines-per-function -- hook with multiple mutation handlers for roster operations (save, clear, publish)
export function useRosterHandlers({
  wsId,
  selectedEmployee,
  selectedPatternId,
  startDate,
  endDate,
  isForever,
  selectedDays,
  weekDays,
  startOfWeek,
  allSchedules,
  mutations,
  currentMonth,
  monthDays,
  setComplianceModalOpen,
  setComplianceViolations,
  setIsComplianceResolving,
}: UseRosterHandlersProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  // ── Compliance retry ref ──────────────────────────────────────────────
  const pendingRetryRef = useRef<{
    assignments: { employeeId: string; employmentId?: string; dates: string[] }[];
    shiftId: string | null;
    onRowStatus?: (employeeId: string, status: "saving" | "saved" | "error") => void;
  } | null>(null);

  const handleAssign = () => {
    if (!selectedEmployee || !selectedPatternId) {
      toast.warning(t("shifts.messages.validation_missing_fields"));
      return;
    }
    mutations.assignPatternMutation.mutate({
      pattern_id: selectedPatternId,
      start_date: startDate,
      end_date: isForever ? null : endDate,
    });
  };

  const handleBulkAssign = (shiftId: string | null) => {
    if (!selectedEmployee || selectedDays.length === 0) return;
    const items = selectedDays.map(date => ({ date, shift_id: shiftId }));
    mutations.bulkRosterMutation.mutate(items);
  };

  const handleAssignAllWeek = (shiftId: string | null) => {
    if (!selectedEmployee) return;
    const items = weekDays.map(day => ({ date: format(day, 'yyyy-MM-dd'), shift_id: shiftId }));
    mutations.bulkRosterMutation.mutate(items);
  };

  const handleCopyPreviousWeek = async () => {
    if (!selectedEmployee || !allSchedules) return;
    const previousWeekStart = addDays(startOfWeek, -7);
    const previousWeekEnd = addDays(startOfWeek, -1);
    try {
      const previousSchedules = await fetchWorkspaceSchedules(wsId, format(previousWeekStart, 'yyyy-MM-dd'), format(previousWeekEnd, 'yyyy-MM-dd'));
      const empPrevious = previousSchedules.find((s: EmployeeSchedule) =>
        s.employee_id === selectedEmployee.id ||
        s.employment_id === selectedEmployee.active_employment_id
      );
      if (!empPrevious) { toast.warning(t("shifts.messages.no_previous_data")); return; }
      const items = empPrevious.schedules.map((s: EffectiveSchedule, index: number) => ({ date: format(weekDays[index], 'yyyy-MM-dd'), shift_id: s.shift_id ?? null }));
      mutations.bulkRosterMutation.mutate(items);
    } catch (e: unknown) { toast.error(e instanceof Error ? e.message : String(e)); }
  };

  // Monthly handlers

  // The monthly grid is backed by a single ["schedules", …, "monthly", …] query
  // holding every selected employee's row. To refresh one row without refetching
  // the whole month, fetch just that employment's schedules and splice the fresh
  // `schedules` array into the cached row, preserving its display fields.
  const monthStartStr = monthDays.length > 0 ? format(monthDays[0], 'yyyy-MM-dd') : null;
  const monthEndStr = monthDays.length > 0 ? format(monthDays[monthDays.length - 1], 'yyyy-MM-dd') : null;

  const refetchAndPatchRow = async (employmentId: string) => {
    if (!monthStartStr || !monthEndStr) return;
    const fresh = await fetchWorkspaceSchedules(wsId, monthStartStr, monthEndStr, [employmentId]);
    const freshRow = fresh.find((s) => s.employment_id === employmentId) ?? fresh[0];
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- .find() can return undefined at runtime without noUncheckedIndexedAccess
    if (!freshRow) return;
    queryClient.setQueriesData<EmployeeSchedule[]>(
      { predicate: (q) => q.queryKey[0] === "schedules" && q.queryKey[3] === "monthly" },
      (old) =>
        old?.map((row) =>
          row.employment_id === employmentId ? { ...row, schedules: freshRow.schedules } : row
        ),
    );
  };

  const handleMonthBulkAssign = async (
    assignments: { employeeId: string; employmentId?: string; dates: string[] }[],
    shiftId: string | null,
    onRowStatus?: (employeeId: string, status: "saving" | "saved" | "error") => void,
  ) => {
    const failures = await runPool(
      assignments,
      ROSTER_REQUEST_CONCURRENCY,
      async ({ employeeId, employmentId, dates }) => {
        const items = dates.map(date => ({ date, shift_id: shiftId }));
        await bulkUpsertRoster(wsId, employeeId, items);
        // Best-effort per-row refresh: a refetch failure must not mark the write
        // (already durable) as failed, so swallow it and let the row stay stale.
        if (employmentId) {
          try {
            await refetchAndPatchRow(employmentId);
          } catch {
            /* row will refresh on next full load */
          }
        }
      },
      {
        onItemStart: ({ employeeId }) => { onRowStatus?.(employeeId, "saving"); },
        onItemSettle: ({ employeeId }, _index, error) => {
          onRowStatus?.(employeeId, error ? "error" : "saved");
        },
      },
    );
    void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });

    // Check for compliance conflicts before generic failure handling.
    const complianceFailure = failures.find(
      (f): f is { item: (typeof assignments)[number]; error: ComplianceConflictError } =>
        f.error instanceof ComplianceConflictError,
    );
    if (complianceFailure) {
      const err = complianceFailure.error;
      pendingRetryRef.current = { assignments, shiftId, onRowStatus };
      setComplianceViolations(err.violations);
      setComplianceModalOpen(true);
      return;
    }

    if (failures.length > 0) {
      toast.error(t("shifts.messages.bulk_partial_failure", { count: failures.length }));
    } else {
      toast.success(t("shifts.messages.bulk_update_success"));
    }
  };

  const handleMonthDeleteRoster = async (
    assignments: { employeeId: string; dates: string[] }[]
  ) => {
    const cells = assignments.flatMap(({ employeeId, dates }) =>
      dates.map(date => ({ employeeId, date }))
    );
    const failures = await runPool(cells, ROSTER_REQUEST_CONCURRENCY, async ({ employeeId, date }) => {
      await deleteRoster(wsId, employeeId, date);
    });
    void queryClient.invalidateQueries({ queryKey: ["schedules"] });
    void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    if (failures.length > 0) {
      toast.error(t("shifts.messages.bulk_partial_failure", { count: failures.length }));
    } else {
      toast.success(t("shifts.messages.remove_override_success"));
    }
  };

  const handleCopyPreviousMonth = async () => {
    const prevMonth = subMonths(currentMonth, 1);
    const prevStartStr = format(startOfMonth(prevMonth), 'yyyy-MM-dd');
    const prevEndStr = format(endOfMonth(prevMonth), 'yyyy-MM-dd');

    try {
      const prevSchedules = await fetchWorkspaceSchedules(wsId, prevStartStr, prevEndStr);
      const currentYearMonth = format(currentMonth, 'yyyy-MM');
      const monthDayStrings = new Set(monthDays.map(d => format(d, 'yyyy-MM-dd')));

      const employeeEntries = prevSchedules
        .map((empSchedule) => {
          const rosterEntries = empSchedule.schedules
            .filter((s: EffectiveSchedule) => s.source === "roster" && !!s.shift_id)
            .map((s: EffectiveSchedule) => {
              const targetDateStr = `${currentYearMonth}-${s.date.substring(8)}`;
              if (monthDayStrings.has(targetDateStr)) {
                return { date: targetDateStr, shift_id: s.shift_id ?? null };
              }
              return null;
            })
            .filter((item): item is { date: string; shift_id: string | null } => item !== null);
          return { employeeId: empSchedule.employee_id, rosterEntries };
        })
        .filter(({ rosterEntries }) => rosterEntries.length > 0);

      if (employeeEntries.length === 0) {
        toast.warning(t("shifts.messages.no_previous_data"));
        return;
      }

      const failures = await runPool(employeeEntries, ROSTER_REQUEST_CONCURRENCY, async ({ employeeId, rosterEntries }) => {
        await bulkUpsertRoster(wsId, employeeId, rosterEntries);
      });
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
      if (failures.length > 0) {
        toast.error(t("shifts.messages.bulk_partial_failure", { count: failures.length }));
      } else {
        toast.success(t("shifts.messages.bulk_update_success"));
      }
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  // ── Compliance override retry ─────────────────────────────────────────
  const onComplianceOverride = useCallback(async (reason: string) => {
    const pending = pendingRetryRef.current;
    if (!pending) return;
    setIsComplianceResolving(true);
    try {
      const { assignments, shiftId, onRowStatus } = pending;
      const failures = await runPool(
        assignments,
        ROSTER_REQUEST_CONCURRENCY,
        async ({ employeeId, employmentId, dates }) => {
          const items = dates.map(date => ({ date, shift_id: shiftId }));
          await bulkUpsertRoster(wsId, employeeId, items, { allow_warnings: true, override_reason: reason });
          if (employmentId) {
            try { await refetchAndPatchRow(employmentId); } catch { /* stale OK */ }
          }
        },
        {
          onItemStart: ({ employeeId }) => { onRowStatus?.(employeeId, "saving"); },
          onItemSettle: ({ employeeId }, _index, error) => {
            onRowStatus?.(employeeId, error ? "error" : "saved");
          },
        },
      );
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
      if (failures.length > 0) {
        toast.error(t("shifts.messages.bulk_partial_failure", { count: failures.length }));
      } else {
        toast.success(t("shifts.messages.bulk_update_success"));
      }
      setComplianceModalOpen(false);
      pendingRetryRef.current = null;
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setIsComplianceResolving(false);
    }
  }, [wsId, queryClient, toast, t, refetchAndPatchRow, setComplianceModalOpen, setIsComplianceResolving]);

  return {
    handleAssign,
    handleAssignAllWeek,
    handleAssignBulk: handleBulkAssign,
    handleCopyPreviousWeek,
    handleMonthBulkAssign,
    handleMonthDeleteRoster,
    handleCopyPreviousMonth,
    onComplianceOverride,
  };
}
