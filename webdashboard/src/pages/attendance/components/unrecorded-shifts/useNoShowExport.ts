/* eslint-disable @typescript-eslint/no-unnecessary-condition, complexity, unicorn/filename-case */
import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  DailyAttendanceReport,
  PaginatedResponse,
} from "@/lib/api";
import { exportNoShowToExcel, type NoShowRow } from "@/lib/excel-no-show";

import { computeNoShowRows } from "../../hooks/use-no-show-rows";
import { zoneLabel } from "./utils";

interface UseNoShowExportParams {
  employeeMap: Map<string, { department_name: string; employee_id_number: string }>;
  fetchNextPage: () => Promise<unknown>;
  queryKey: unknown[];
  selectedDate: string;
  selectedDept: string;
  workspaceTimezone: string;
  wsId: string;
}

interface UseNoShowExportReturn {
  /** Start the export. Idempotent while already exporting. */
  exportFn: () => Promise<void>;
  /** Whether an export is currently in progress. */
  isExporting: boolean;
}

/**
 * Hook: drain all pages of the no-show infinite query, compute zone-aware rows,
 * and export them as an .xlsx file.
 */
export function useNoShowExport({
  wsId,
  queryKey,
  fetchNextPage,
  employeeMap,
  selectedDate,
  workspaceTimezone,
  selectedDept,
}: UseNoShowExportParams): UseNoShowExportReturn {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [isExporting, setIsExporting] = useState(false);

  const exportFn = useCallback(async () => {
    if (!wsId) return;
    setIsExporting(true);
    try {
      // Drain all remaining pages so we have the full dataset in cache.
      let guard = 0;
      while (guard < 1000) {
        const cache = queryClient.getQueryData<{
          pages: PaginatedResponse<DailyAttendanceReport>[];
          pageParams: readonly number[];
        }>(queryKey);
        const pages = cache?.pages ?? [];
        if (pages.length === 0) break;
        const loadedEmployees = new Set(
          pages.flatMap((p) => p.data.map((r) => r.employment_id)),
        ).size;
        const total = pages[pages.length - 1]?.total ?? 0;
        if (loadedEmployees >= total) break;
        await fetchNextPage();
        guard++;
      }

      const fullCache = queryClient.getQueryData<{
        pages: PaginatedResponse<DailyAttendanceReport>[];
        pageParams: readonly number[];
      }>(queryKey);
      const allReportRows = fullCache?.pages?.flatMap((p) => p.data) ?? [];

      const rows = computeNoShowRows({
        rows: allReportRows,
        employeeMap,
        viewingDate: selectedDate,
        workspaceTimezone,
      }).filter((r) =>
        selectedDept === "__all__" ? true : r.department_name === selectedDept,
      );

      const noShowRows: NoShowRow[] = rows.map((r) => ({
        employee_id_number: r.employee_id_number,
        employee_name: r.employee_name,
        department_name: r.department_name,
        shift_name: r.shift_name ?? null,
        scheduled_start: r.scheduled_start ?? null,
        scheduled_end: r.scheduled_end ?? null,
        zone_label: zoneLabel(r, t).key,
      }));
      void exportNoShowToExcel(noShowRows, {
        t,
        dateLabel: selectedDate,
      });
    } finally {
      setIsExporting(false);
    }
  }, [
    wsId,
    queryClient,
    queryKey,
    fetchNextPage,
    employeeMap,
    selectedDate,
    workspaceTimezone,
    selectedDept,
    t,
  ]);

  return { exportFn, isExporting };
}
