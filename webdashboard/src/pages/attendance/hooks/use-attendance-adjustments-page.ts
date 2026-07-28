import { useInfiniteQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { AdjustmentSearch } from "@/pages/attendance/search-schema";

import { useToast } from "@/components/ui/toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import {
  fetchWorkspaceAttendanceAdjustments,
  approveAttendanceAdjustment,
  getCurrentWorkspaceId,
  PaginatedResponse,
  AttendanceAdjustmentRequest,
} from "@/lib/api";
import { logError } from "@/lib/logger";
import { useSearchPatch } from "@/lib/state";
import { Route } from "@/routes/_authenticated/_workspace/attendance-adjustments";

export type TabType = AdjustmentSearch["tab"];
export type AttendanceStatusFilter = AdjustmentSearch["status"];

const PAGE_SIZE = 20;

export function useAttendanceAdjustmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const toast = useToast();

  const search = Route.useSearch();
  const patchSearch = useSearchPatch<AdjustmentSearch>();

  const activeTab = search.tab;
  const statusFilter = search.status;
  const employeeId = search.employee ?? "all";

  const setActiveTab = useCallback((tab: TabType) => { patchSearch({ tab }); }, [patchSearch]);
  const setStatusFilter = useCallback((status: AttendanceStatusFilter) => { patchSearch({ status }); }, [patchSearch]);
  const setEmployeeId = useCallback((id: string) => { patchSearch({ employee: id === "all" ? undefined : id }); }, [patchSearch]);

  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);

  const {
    data: paginatedRequests,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery<PaginatedResponse<AttendanceAdjustmentRequest>>({
    queryKey: ["attendance-adjustments", wsId, activeTab, employeeId, statusFilter],
    queryFn: ({ pageParam }) => {
      return wsId
        ? fetchWorkspaceAttendanceAdjustments(wsId, {
            status: activeTab !== "behalf_assignments" && statusFilter !== "all" ? statusFilter : undefined,
            employee_id: employeeId,
            page: pageParam as number,
            limit: PAGE_SIZE,
          })
        : Promise.reject(new Error("No workspace ID"));
    },
    getNextPageParam: (lastPage, allPages) => {
      const totalLoaded = allPages.reduce((sum, p) => sum + (p.data.length || 0), 0);
      return totalLoaded >= (lastPage.total || 0) ? undefined : allPages.length + 1;
    },
    initialPageParam: 1,
    enabled: !!wsId && activeTab !== "behalf_assignments",
  });

  const requests = useMemo(
    () => paginatedRequests?.pages.flatMap((p) => p.data) ?? [],
    [paginatedRequests]
  );
  const requestsTotal = paginatedRequests?.pages[0]?.total ?? 0;

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage: () => { void fetchNextPage(); },
  });

  const { employees } = useAllEmployees(wsId);

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: "pending" | "approved" | "rejected" }) =>
      wsId ? approveAttendanceAdjustment(wsId, data.id, { status: data.status }) : Promise.reject(new Error("No workspace ID")),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] });
      void queryClient.invalidateQueries({ queryKey: ["time-history"] });
      toast.success(t("common.success"));
    },
  });

  const downloadTemplate = async () => {
    try {
      const excelJSModule = await import("exceljs");
      const ExcelJS = "default" in excelJSModule ? excelJSModule.default : excelJSModule;
      const { saveAs } = await import("file-saver");

      const workbook = new ExcelJS.Workbook();
      const worksheet = workbook.addWorksheet("Template");
      worksheet.columns = [
        { header: "employee_code", key: "employee_code", width: 20 },
        { header: "punch_time", key: "punch_time", width: 25 },
      ];
      worksheet.addRow({
        employee_code: "EMP001",
        punch_time: "11/01/2026 08:00",
      });

      const buffer = await workbook.xlsx.writeBuffer();
      const blob = new Blob([buffer], {
        type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      });
      saveAs(blob, "attendance_template.xlsx");
    } catch (error) {
      logError(error, { component: "useAttendanceAdjustmentsPage" });
    }
  };

  const getEmployeeName = (employmentId: string) => {
    const emp = employees.find((e) => e.active_employment_id === employmentId);
    return emp ? emp.full_name : "Unknown";
  };

  const getEmployeeID = (employmentId: string) => {
    const emp = employees.find((e) => e.active_employment_id === employmentId);
    return emp ? emp.employee_id_number : "N/A";
  };

  return {
    wsId,
    t,
    formatDate,
    formatTime,
    requests,
    isLoading,
    activeTab,
    setActiveTab,
    statusFilter,
    setStatusFilter,
    employeeId,
    setEmployeeId,
    requestsTotal,
    hasNextPage,
    isFetchingNextPage,
    loadMoreRef,
    isImportDialogOpen,
    setIsImportDialogOpen,
    approveMutation,
    downloadTemplate,
    getEmployeeName,
    getEmployeeID,
  };
}
