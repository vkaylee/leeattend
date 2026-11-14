import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { addMonths, endOfMonth, format, startOfMonth, subMonths } from "date-fns";

import type { AttendanceDailySummary, Employee, PaginatedResponse } from "@/lib/api";

import { useToast } from "@/components/ui/toast";
import { useApproveOt, useAttendanceSummariesInfinite } from "@/hooks/use-attendance";
import { useOrganization } from "@/hooks/use-organization";
import { useMyPermissions } from "@/hooks/use-permissions";
import { fetchEmployees } from "@/lib/api";
import { logError } from "@/lib/logger";
import { invalidateAttendanceQueries } from "@/lib/utils/query-invalidation";

import type { SourceFilter } from "../components/ot-approvals-filters";
import type { OtSummary } from "../components/ot-approvals-table";
import { createQueryKey } from "@/lib/query-keys";


// ── Helpers ──────────────────────────────────────────────────────────

export function getPendingSummaries(
  summaries: AttendanceDailySummary[],
  sourceFilter: SourceFilter,
  employeeNameMap: Map<string, { idNumber: string; name: string }>
): OtSummary[] {
  return summaries
    .filter((s) => sourceFilter === "all" || s.entered_via === sourceFilter)
    .map((s) => {
      const emp = employeeNameMap.get(s.employment_id);
      return {
        ...s,
        employee_name: emp?.name ?? s.employment_id,
        employee_id_number: emp?.idNumber ?? "",
      };
    });
}

// ── Hook ─────────────────────────────────────────────────────────────

export function useOtApprovals() {
  const toast = useToast();
  const { t } = useTranslation(["common", "attendance"]);
  const { organization } = useOrganization();
  const queryClient = useQueryClient();
  const { hasPermission, isLoading: permissionsLoading } = useMyPermissions();

  const canApprove = hasPermission("overtime:approve");
  const canClaimOnBehalf = hasPermission("overtime:claim_on_behalf");
  const canManagePolicies = hasPermission("overtime:policies:manage");
  const canCreateOnBehalf = hasPermission("overtime:request:create_on_behalf");

  // ── State ────────────────────────────────────────────────────────
  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [bulkUploadDialogOpen, setBulkUploadDialogOpen] = useState(false);
  const [photoTargetSummary, setPhotoTargetSummary] = useState<OtSummary | null>(null);
  const [sourceFilter, setSourceFilter] = useState<SourceFilter>("all");
  const [activeTab, setActiveTab] = useState<"pre" | "post" | "delegated_claim" | "behalf_assignments">("pre");
  const [tabInitialized, setTabInitialized] = useState(false);

  const [previewFileUrl, setPreviewFileUrl] = useState<string | null>(null);
  const [previewTitle, setPreviewTitle] = useState<string>("");
  const [isPreviewOpen, setIsPreviewOpen] = useState<boolean>(false);

  // ── Tab initialization ───────────────────────────────────────────
  useEffect(() => {
    if (!tabInitialized && !permissionsLoading) {
      setTimeout(() => {
        if (!canApprove && !canCreateOnBehalf && canClaimOnBehalf) {
          setActiveTab("delegated_claim");
        } else if (!canApprove && !canCreateOnBehalf && !canClaimOnBehalf && canManagePolicies) {
          setActiveTab("behalf_assignments");
        }
        setTabInitialized(true);
      }, 0);
    }
  }, [tabInitialized, permissionsLoading, canApprove, canCreateOnBehalf, canClaimOnBehalf, canManagePolicies]);

  // ── Derived dates ────────────────────────────────────────────────
  const startDate = format(startOfMonth(currentMonth), "yyyy-MM-dd");
  const endDate = format(endOfMonth(currentMonth), "yyyy-MM-dd");

  // ── Data fetching ────────────────────────────────────────────────
  const {
    summaries,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useAttendanceSummariesInfinite(startDate, endDate, { otStatus: "pending_manager" });
  const { approveOt, isProcessing } = useApproveOt();

  const { data: employeesData } = useQuery<PaginatedResponse<Employee>>({
    queryKey: createQueryKey("employees-list-all", organization?.id),
    queryFn: () => fetchEmployees(1, 1000),
    enabled: !!organization?.id,
  });

  // ── Memoized maps ────────────────────────────────────────────────
  const employmentToEmployeeMap = useMemo(() => {
    const map = new Map<string, string>();
    employeesData?.data.forEach((emp: Employee) => {
      if (emp.active_employment_id) {
        map.set(emp.active_employment_id, emp.id);
      }
    });
    return map;
  }, [employeesData]);

  const employeeNameMap = useMemo(() => {
    const map = new Map<string, { idNumber: string; name: string }>();
    employeesData?.data.forEach((emp: Employee) => {
      if (emp.active_employment_id) {
        map.set(emp.active_employment_id, {
          name: emp.full_name,
          idNumber: emp.employee_id_number,
        });
      }
    });
    return map;
  }, [employeesData]);

  // ── Handlers ─────────────────────────────────────────────────────
  const handleApprove = async (summary: OtSummary, approved: boolean) => {
    const employeeId = employmentToEmployeeMap.get(summary.employment_id);
    if (!employeeId) {
      toast.error(t("ot_approvals.no_active_employee", "Không tìm thấy thông tin nhân viên hoạt động"));
      return;
    }
    try {
      await approveOt({
        approved_minutes: approved ? summary.claimed_ot_minutes : 0,
        date: summary.date,
        employeeId,
        manager_note: approved ? "Approved via Manager Portal" : "Rejected via Manager Portal",
        status: approved ? "approved" : "rejected",
      });
      toast.success(
        approved ? "Overtime approved successfully" : "Overtime rejected successfully"
      );
      invalidateAttendanceQueries(queryClient);
    } catch (e: unknown) {
      logError(e, { component: "OtApprovalsPage" });
      toast.error("Action failed");
    }
  };

  const handleMonthYearChange = (month: number, year: number) => {
    setCurrentMonth(new Date(year, month - 1, 1));
  };

  const handlePreviousMonth = () => {
    setCurrentMonth(subMonths(currentMonth, 1));
  };

  const handleNextMonth = () => {
    setCurrentMonth(addMonths(currentMonth, 1));
  };

  return {
    // Permissions
    canApprove,
    canClaimOnBehalf,
    canManagePolicies,
    canCreateOnBehalf,
    permissionsLoading,
    // Date
    currentMonth,
    startDate,
    endDate,
    // State
    bulkUploadDialogOpen,
    setBulkUploadDialogOpen,
    photoTargetSummary,
    setPhotoTargetSummary,
    sourceFilter,
    setSourceFilter,
    activeTab,
    setActiveTab,
    // Preview
    previewFileUrl,
    setPreviewFileUrl,
    previewTitle,
    setPreviewTitle,
    isPreviewOpen,
    setIsPreviewOpen,
    // Data
    summaries,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isProcessing,
    // Maps
    employmentToEmployeeMap,
    employeeNameMap,
    // Handlers
    handleApprove,
    handleMonthYearChange,
    handlePreviousMonth,
    handleNextMonth,
    // Misc
    queryClient,
    organization,
    t,
  };
}
