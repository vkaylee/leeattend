import type { UseMutationResult } from "@tanstack/react-query";

import { useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { format, subMonths, startOfMonth, endOfMonth } from "date-fns";

import type { ComplianceViolation } from "@/lib/api/errors";
import type { AssignPatternRequest, Employee, EmployeeSchedule, EmployeeWorkingPattern } from "@/lib/api/types";

import { translateRosterEntriesToMonth } from "@/components/shifts/roster/roster-utils";
import { useToast } from "@/components/ui/toast";
import { fetchWorkspaceSchedules, bulkUpsertRoster, deleteRoster } from "@/lib/api";

import { useComplianceRetry } from "./use-compliance-retry";
import { useMonthlyRosterCachePatch } from "./use-monthly-cache-patch";
import { useWeeklyRosterHandlers } from "./use-weekly-roster-handlers";
import { runRosterBulk } from "./roster-bulk-utils";

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

  // ── Composed sub-hooks ────────────────────────────────────────────────

  const { refetchAndPatchRow } = useMonthlyRosterCachePatch({
    wsId,
    monthDays,
    queryClient,
  });

  const weekly = useWeeklyRosterHandlers({
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
    t,
    toast,
  });

  const { pendingRetryRef, onComplianceOverride } = useComplianceRetry({
    wsId,
    queryClient,
    toast,
    t,
    refetchAndPatchRow,
    setComplianceModalOpen,
    setIsComplianceResolving,
  });

  // ── Monthly handlers ──────────────────────────────────────────────────

  const handleMonthBulkAssign = async (
    assignments: { employeeId: string; employmentId?: string; dates: string[] }[],
    shiftId: string | null,
    onRowStatus?: (employeeId: string, status: "saving" | "saved" | "error") => void,
  ) => {
    await runRosterBulk({
      items: assignments,
      worker: async ({ employeeId, employmentId, dates }) => {
        const items = dates.map((date) => ({ date, shift_id: shiftId }));
        await bulkUpsertRoster(wsId, employeeId, items);
        if (employmentId) {
          try {
            await refetchAndPatchRow(employmentId);
          } catch {
            /* row will refresh on next full load */
          }
        }
      },
      queryClient,
      invalidateKeys: [["assignment_history"]],
      toast,
      t,
      partialFailureKey: "shifts.messages.bulk_partial_failure",
      successKey: "shifts.messages.bulk_update_success",
      callbacks: {
        onItemStart: ({ employeeId }) => {
          onRowStatus?.(employeeId, "saving");
        },
        onItemSettle: ({ employeeId }, _index, error) => {
          onRowStatus?.(employeeId, error ? "error" : "saved");
        },
      },
      onComplianceFailure: (cf) => {
        pendingRetryRef.current = {
          assignments,
          shiftId,
          onRowStatus,
        };
        setComplianceViolations(cf.error.violations);
        setComplianceModalOpen(true);
      },
    });
  };

  const handleMonthDeleteRoster = async (
    assignments: { employeeId: string; dates: string[] }[],
  ) => {
    const cells = assignments.flatMap(({ employeeId, dates }) =>
      dates.map((date) => ({ employeeId, date })),
    );
    await runRosterBulk({
      items: cells,
      worker: async ({ employeeId, date }) => {
        await deleteRoster(wsId, employeeId, date);
      },
      queryClient,
      invalidateKeys: [["schedules"], ["assignment_history"]],
      toast,
      t,
      partialFailureKey: "shifts.messages.bulk_partial_failure",
      successKey: "shifts.messages.remove_override_success",
    });
  };

  const handleCopyPreviousMonth = async () => {
    const prevMonth = subMonths(currentMonth, 1);
    const prevStartStr = format(startOfMonth(prevMonth), "yyyy-MM-dd");
    const prevEndStr = format(endOfMonth(prevMonth), "yyyy-MM-dd");

    try {
      const prevSchedules = await fetchWorkspaceSchedules(
        wsId,
        prevStartStr,
        prevEndStr,
      );
      const currentYearMonth = format(currentMonth, "yyyy-MM");
      const monthDayStrings = new Set(
        monthDays.map((d) => format(d, "yyyy-MM-dd")),
      );

      const employeeEntries = translateRosterEntriesToMonth(
        prevSchedules,
        currentYearMonth,
        monthDayStrings,
      );

      if (employeeEntries.length === 0) {
        toast.warning(t("shifts.messages.no_previous_data"));
        return;
      }

      await runRosterBulk({
        items: employeeEntries,
        worker: async ({ employeeId, rosterEntries }) => {
          await bulkUpsertRoster(wsId, employeeId, rosterEntries);
        },
        queryClient,
        invalidateKeys: [["schedules"], ["assignment_history"]],
        toast,
        t,
        partialFailureKey: "shifts.messages.bulk_partial_failure",
        successKey: "shifts.messages.bulk_update_success",
      });
    } catch (e: unknown) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  return {
    handleAssign: weekly.handleAssign,
    handleAssignAllWeek: weekly.handleAssignAllWeek,
    handleAssignBulk: weekly.handleAssignBulk,
    handleCopyPreviousWeek: weekly.handleCopyPreviousWeek,
    handleMonthBulkAssign,
    handleMonthDeleteRoster,
    handleCopyPreviousMonth,
    onComplianceOverride,
  };
}
