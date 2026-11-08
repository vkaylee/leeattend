import type { UseMutationResult } from "@tanstack/react-query";

import { useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import type { ComplianceViolation } from "@/lib/api/errors";
import type { AssignPatternRequest, Employee, EmployeeSchedule, EmployeeWorkingPattern } from "@/lib/api/types";

import { useToast } from "@/components/ui/toast";
import { bulkUpsertRoster, deleteRoster } from "@/lib/api";
import { logError } from "@/lib/logger";

import { copyPreviousMonth as copyPrevMonthFn } from "./copy-previous-month";
import { runRosterBulk } from "./roster-bulk-utils";
import { useComplianceRetry } from "./use-compliance-retry";
import { useMonthlyRosterCachePatch } from "./use-monthly-cache-patch";
import { useWeeklyRosterHandlers } from "./use-weekly-roster-handlers";

interface UseRosterHandlersProps { allSchedules: EmployeeSchedule[] | undefined;
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
weekDays: Date[];  }

export function useRosterHandlers({
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
    monthDays,
    queryClient,
  });

  const weekly = useWeeklyRosterHandlers({
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
        await bulkUpsertRoster(employeeId, items);
        if (employmentId) {
          try {
            await refetchAndPatchRow(employmentId);
          } catch (e: unknown) { logError(e, { component: 'useRosterHandlers.rowRefetch' }); /* row will refresh on next full load */ }
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
        await deleteRoster(employeeId, date);
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
    await copyPrevMonthFn({ currentMonth, monthDays, queryClient, toast, t });
  };

  return {
    handleApplyShift: weekly.handleApplyShift,
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
