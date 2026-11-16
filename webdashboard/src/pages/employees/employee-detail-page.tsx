import { useQuery } from "@tanstack/react-query";
import { Link, useNavigate, useParams, useSearch } from "@tanstack/react-router";
import {
  ArrowLeft,
  ArrowLeftRight,
  Briefcase,
  Building2,
  Calendar,
  Mail,
  Pencil,
  Trash2,
  UserPlus,
} from "lucide-react";
import React, { createElement, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import {
  EMPLOYEE_DRAWER_TAB_CONFIG,
  type DetailTabType,
} from "@/components/features/employees/drawer-header";
import { DrawerTabContent } from "@/components/features/employees/employee-detail-drawer";
import { useEmployeeDetail } from "@/components/features/employees/hooks/use-employee-detail";
import Button from "@/components/ui/button";
import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDepartments } from "@/hooks/use-departments";
import {
  fetchEmployee,
  fetchEmployeeDepartmentHistory,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { cn } from "@/lib/utils";

import { EmployeeOverlays } from "./components/employee-overlays";
import { useEmployeesModals } from "./hooks/use-employees-modals";


export default function EmployeeDetailPage() {
  const { t } = useTranslation();

  const { formatDate } = useDateTimeFormatter();
  const params: { employeeId?: string } = useParams({ strict: false });
  const search: { tab?: DetailTabType } = useSearch({ strict: false });
  const navigate = useNavigate();

  const employeeId = params.employeeId ?? "";
  const modals = useEmployeesModals();

  // Active tab state synced with search param ?tab=
  const [localTab, setLocalTab] = useState<DetailTabType>(() => search.tab ?? "profile");
  const activeDetailTab = search.tab ?? localTab;

  const handleTabChange = (tabId: DetailTabType) => {
    setLocalTab(tabId);
    void navigate({
      search: (prev: Record<string, unknown>) => ({ ...prev, tab: tabId }) as never,
      replace: true,
    });
  };

  const { data: employee, isLoading, error, refetch } = useQuery({
    enabled: Boolean(employeeId),
    queryKey: createQueryKey("employee", employeeId),
    queryFn: () => fetchEmployee(employeeId),
  });

  const {
    activeEmployment,
    linkedUser,
    locations,
    scrollRef,
  } = useEmployeeDetail({
    employee: employee ?? null,
    isOpen: true,
  });

  const { data: departments } = useDepartments();

  const { data: deptHistory } = useQuery({
    enabled: Boolean(employeeId),
    queryKey: createQueryKey("deptHistory", employeeId),
    queryFn: () => fetchEmployeeDepartmentHistory(employeeId),
  });

  const tabs = useMemo(
    () =>
      EMPLOYEE_DRAWER_TAB_CONFIG.map((tab) => ({
        id: tab.id,
        label: t(tab.i18nKey),
        icon: tab.icon,
      })),
    [t]
  );

  if (isLoading) {
    return (
      <div className="space-y-6 pb-20 animate-pulse">
        <div className="h-8 w-48 bg-slate-200 dark:bg-slate-800 rounded-xl" />
        <div className="h-44 bg-slate-200 dark:bg-slate-800 rounded-3xl" />
        <div className="h-12 bg-slate-200 dark:bg-slate-800 rounded-2xl" />
        <div className="h-96 bg-slate-200 dark:bg-slate-800 rounded-3xl" />
      </div>
    );
  }

  if (error || !employee) {
    return (
      <div className="flex flex-col items-center justify-center p-12 text-center bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 space-y-4">
        <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100" variant="h3">
          {t("employees.not_found", "Không tìm thấy thông tin nhân sự")}
        </Typography>
        <Link to="/employees">
          <Button icon={<ArrowLeft size={16} />} variant="outline">
            {t("common.back_to_list", "Quay lại danh sách")}
          </Button>
        </Link>
      </div>
    );
  }

  const isActive = Boolean(employee.active_employment_id);

  return (
    <div className="space-y-6 pb-24 animate-in fade-in duration-500">
      {/* ── Breadcrumb & Navigation Bar ─────────────────────────────── */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-xs font-semibold text-slate-500 dark:text-slate-400">
          <Link
            className="flex items-center gap-1.5 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
            to="/employees"
          >
            <ArrowLeft size={14} />
            <span>{t("employees.title", "Nhân sự")}</span>
          </Link>
          <span className="text-slate-300 dark:text-slate-600">/</span>
          <span className="text-slate-800 dark:text-slate-200 font-bold">{employee.full_name}</span>
          <span className="font-mono text-[10px] px-2 py-0.5 rounded-md bg-slate-100 dark:bg-slate-800 text-slate-500 font-bold">
            {employee.employee_id_number}
          </span>
        </div>

        <Link to="/employees">
          <Button
            className="h-9 text-xs font-bold"
            icon={<ArrowLeft size={14} />}
            variant="ghost"
          >
            {t("common.back_to_list", "Danh sách")}
          </Button>
        </Link>
      </div>

      {/* ── Executive Hero Banner ──────────────────────────────────── */}
      <div className="relative overflow-hidden rounded-3xl border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 p-6 sm:p-8 shadow-sm">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
          <div className="flex items-start sm:items-center gap-5">
            <div className="size-16 sm:size-20 bg-primary-600 text-white rounded-2xl flex items-center justify-center font-bold text-2xl sm:text-3xl shadow-lg shadow-primary-500/20 border-2 border-white dark:border-slate-800 uppercase shrink-0">
              {employee.full_name[0]}
            </div>

            <div className="space-y-1.5 min-w-0">
              <div className="flex flex-wrap items-center gap-2.5">
                <Typography className="text-2xl sm:text-3xl font-black text-slate-900 dark:text-slate-100 tracking-tight" variant="h1">
                  {employee.full_name}
                </Typography>
                <StatusBadge variant={isActive ? "success" : "neutral"}>
                  {isActive ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}
                </StatusBadge>
              </div>

              <div className="flex flex-wrap items-center gap-x-4 gap-y-1.5 text-xs text-slate-600 dark:text-slate-300">
                <span className="font-mono font-bold text-primary-600 dark:text-primary-400">
                  {employee.employee_id_number}
                </span>

                {!!employee.job_title && (
                  <span className="flex items-center gap-1.5 font-semibold">
                    <Briefcase className="text-slate-400" size={13} />
                    <span>{employee.job_title}</span>
                  </span>
                )}

                {!!employee.department_name && (
                  <span className="flex items-center gap-1.5 font-semibold">
                    <Building2 className="text-slate-400" size={13} />
                    <span>{employee.department_name}</span>
                  </span>
                )}

                {!!employee.hire_date && (
                  <span className="flex items-center gap-1.5 text-slate-500 dark:text-slate-400">
                    <Calendar className="text-slate-400" size={13} />
                    <span>{t("common.hire_date", "Vào làm")}: {formatDate(employee.hire_date)}</span>
                  </span>
                )}

                {!!linkedUser?.email && (
                  <span className="flex items-center gap-1.5 text-slate-500 dark:text-slate-400">
                    <Mail className="text-slate-400" size={13} />
                    <span>{linkedUser.email}</span>
                  </span>
                )}
              </div>
            </div>
          </div>

          {/* Action Toolbar */}
          <div className="flex flex-wrap items-center gap-2.5 pt-4 lg:pt-0 border-t lg:border-t-0 border-slate-100 dark:border-slate-800">
            <PermissionGate permission="employees:update">
              <Button
                className="h-10 text-xs font-bold"
                data-testid="edit-employee-btn"
                icon={<Pencil size={14} />}
                onClick={() => {
                  modals.openModal({ type: "edit", employee });
                }}
                variant="outline"
              >
                {t("employees.details.edit_profile", "Sửa thông tin")}
              </Button>

              <Button
                className="h-10 text-xs font-bold"
                data-testid="transfer-employee-btn"
                icon={<ArrowLeftRight size={14} />}
                onClick={() => {
                  modals.openModal({ type: "transfer", employee });
                }}
                variant="outline"
              >
                {t("employees.details.transfer", "Chuyển phòng ban")}
              </Button>

              {isActive ? (
                <Button
                  className="h-10 text-xs font-bold text-rose-600 dark:text-rose-400 border-rose-200 dark:border-rose-900/50 hover:bg-rose-50 dark:hover:bg-rose-950"
                  data-testid="terminate-employee-btn"
                  icon={<Trash2 size={14} />}
                  onClick={() => {
                    modals.openModal({ type: "terminate", employee });
                  }}
                  variant="outline"
                >
                  {t("employees.details.terminate", "Thôi việc")}
                </Button>
              ) : (
                <Button
                  className="h-10 text-xs font-bold"
                  data-testid="rehire-employee-btn"
                  icon={<UserPlus size={14} />}
                  onClick={() => {
                    modals.openModal({ type: "rehire", employee });
                  }}
                  variant="primary"
                >
                  {t("employees.details.rehire", "Tuyển dụng lại")}
                </Button>
              )}
            </PermissionGate>
          </div>
        </div>
      </div>

      {/* ── Sticky Full-Width Tab Navigation ───────────────────────── */}
      <div className="sticky top-0 z-30 -mx-4 sm:-mx-6 px-4 sm:px-6 py-2 bg-slate-50/90 dark:bg-slate-950/90 backdrop-blur-md border-b border-slate-200 dark:border-slate-800">
        <div className="flex items-center gap-1.5 overflow-x-auto no-scrollbar scroll-smooth">
          {tabs.map((tab) => {
            const isTabActive = activeDetailTab === tab.id;
            return (
              <Button
                aria-selected={isTabActive}
                className={cn(
                  "flex items-center gap-2 h-auto py-2.5 px-4 rounded-xl text-xs font-bold uppercase tracking-wider transition-all whitespace-nowrap cursor-pointer shrink-0 shadow-none border",
                  isTabActive
                    ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 border-slate-200 dark:border-slate-700 shadow-sm"
                    : "bg-transparent text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800/50 border-transparent"
                )}
                data-testid={`tab-${tab.id}`}
                key={tab.id}
                onClick={() => { handleTabChange(tab.id); }}
                role="tab"
                type="button"
                variant="ghost"
              >
                {createElement(tab.icon, {
                  size: 14,
                  className: isTabActive ? "text-primary-600 dark:text-primary-400" : "opacity-60",
                })}
                <span>{tab.label}</span>
              </Button>
            );
          })}
        </div>
      </div>

      {/* ── Full-Width Tab Content Organization ───────────────────────── */}
      <div
        aria-labelledby={`tab-${activeDetailTab}`}
        className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-200 dark:border-slate-800 p-6 sm:p-10 shadow-xs min-h-[500px]"
        id={`tabpanel-${activeDetailTab}`}
        ref={scrollRef}
        role="tabpanel"
      >
        <DrawerTabContent activeDetailTab={activeDetailTab}
        activeEmployment={activeEmployment}
        departments={departments}
        deptHistory={deptHistory}
        employee={employee}
        formatDate={formatDate}
        linkedUser={linkedUser}
        locations={locations}
        onEdit={() => { modals.openModal({ type: "edit", employee }); }}
        onEmployeeUpdate={() => { void refetch(); }}
        onRehire={() => { modals.openModal({ type: "rehire", employee }); }}
        onTerminate={() => { modals.openModal({ type: "terminate", employee }); }}
        onTransfer={(deptId) => {
          modals.setSelectedDeptId(deptId);
          modals.openModal({ type: "transfer", employee, departmentId: deptId });
        }}
        t={t}  />
      </div>

      {/* ── Unified Overlays (Modals) ──────────────────────────────── */}
      <EmployeeOverlays activeModal={modals.activeModal}
      closeModal={modals.closeModal}
      departments={departments}
      deptHistory={deptHistory}
      formatDate={formatDate}
      handleDrawerClose={() => undefined}
      openModal={modals.openModal}
      prefillEnrollmentId={modals.prefillEnrollmentId}
      selectedDeptId={modals.selectedDeptId}
      selectedEmployee={null}
      setSelectedDeptId={modals.setSelectedDeptId}
      setSelectedEmployee={() => { void refetch(); }}  />
    </div>
  );
}
