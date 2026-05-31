// @ts-nocheck
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useAllEmployees } from "@/hooks/use-employees";
import {
  fetchWorkspaceAttendanceAdjustments,
  approveAttendanceAdjustment,
  getCurrentWorkspaceId,
} from "@/lib/api";

export type TabType = "pending" | "approved" | "rejected";

export function useAttendanceAdjustmentsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { formatDate, formatTime } = useDateTimeFormatter();
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>("pending");
  const toast = useToast();

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ["attendance-adjustments", wsId, activeTab],
    queryFn: () => wsId ? fetchWorkspaceAttendanceAdjustments(wsId, activeTab) : Promise.reject(new Error("No workspace ID")),
    enabled: !!wsId,
  });

  const { employees } = useAllEmployees(wsId);

  const approveMutation = useMutation({
    mutationFn: (data: { id: string; status: string }) =>
      wsId ? approveAttendanceAdjustment(wsId, data.id, { status: data.status }) : Promise.reject(new Error("No workspace ID")),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["attendance-adjustments"] });
      void queryClient.invalidateQueries({ queryKey: ["time-history"] });
      toast.success(t("common.success"));
    },
  });

  const downloadTemplate = () => {
    const headers = "employee_code,punch_time";
    const example = "EMP001,11/01/2026 08:00";
    const blob = new Blob([`\uFEFF${headers}\n${example}`], { type: "text/csv;charset=utf-8;" });
    const link = document.body.appendChild(document.createElement("a"));
    link.href = URL.createObjectURL(blob);
    link.download = "attendance_template.csv";
    link.click();
    document.body.removeChild(link);
  };

  const getEmployeeName = (id: string) => {
    const emp = employees.find((e) => e.id === id);
    return emp ? emp.full_name : "Unknown";
  };

  const getEmployeeID = (id: string) => {
    const emp = employees.find((e) => e.id === id);
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
    isImportDialogOpen,
    setIsImportDialogOpen,
    approveMutation,
    downloadTemplate,
    getEmployeeName,
    getEmployeeID,
  };
}
