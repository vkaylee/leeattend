import type { QueryClient } from "@tanstack/react-query";
import type { TFunction } from "i18next";

import { format, subMonths, startOfMonth, endOfMonth } from "date-fns";

import type { useToast } from "@/components/ui/toast";

import { translateRosterEntriesToMonth } from "@/components/shifts/roster/roster-utils";
import { fetchWorkspaceSchedules, bulkUpsertRoster } from "@/lib/api";

import { runRosterBulk } from "./roster-bulk-utils";

interface CopyPreviousMonthOptions {
  currentMonth: Date;
  monthDays: Date[];
  queryClient: QueryClient;
  t: TFunction;
  toast: ReturnType<typeof useToast>;
  wsId: string;
}

export async function copyPreviousMonth({
  currentMonth,
  monthDays,
  wsId,
  queryClient,
  toast,
  t,
}: CopyPreviousMonthOptions): Promise<void> {
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
}
