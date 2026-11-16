import { Link, useSearch } from "@tanstack/react-router";
import { Clock, ListTodo, Plus, UserMinus, UserPlus, Users, X } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import LifecycleAssignModal from "@/components/features/employees/lifecycle-assign-modal";
import { LifecycleFilters } from "@/components/features/personnel/lifecycle-filters";
import { LifecycleTable } from "@/components/features/personnel/lifecycle-table";
import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";
import { Typography } from "@/components/ui/typography";
import { useOrganizationLifecycleInstances } from "@/hooks/use-lifecycle";
import { type LifecycleInstanceSummary } from "@/lib/api";

export default function PersonnelLifecyclePage() {
  const { t } = useTranslation();
  const searchParams = useSearch({ strict: false });
  const employeeFilter = searchParams.employeeId;

  const [search, setSearch] = useState(searchParams.q ?? "");
  const [selectedType, setSelectedType] = useState<string>("all");
  const [selectedStatus, setSelectedStatus] = useState<string>("all");
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { data: instances = [], isLoading } = useOrganizationLifecycleInstances();

  // Metrics
  const metrics = useMemo(() => {
    const total = instances.length;
    const inProgress = instances.filter((c) => c.status === "in_progress").length;
    const onboarding = instances.filter(
      (c) => c.status === "in_progress" && c.checklist_type.toLowerCase() === "onboarding"
    ).length;
    const offboarding = instances.filter(
      (c) => c.status === "in_progress" && c.checklist_type.toLowerCase() === "offboarding"
    ).length;
    return { total, inProgress, onboarding, offboarding };
  }, [instances]);

  // Filtering
  const filteredInstances = useMemo(() => {
    return instances.filter((c: LifecycleInstanceSummary) => {
      const q = search.toLowerCase();
      const matchSearch =
        !search ||
        c.employee_name.toLowerCase().includes(q) ||
        c.employee_code.toLowerCase().includes(q) ||
        (c.department_name?.toLowerCase().includes(q) ?? false) ||
        (c.template_name?.toLowerCase().includes(q) ?? false) ||
        c.checklist_type.toLowerCase().includes(q);

      const matchType = selectedType === "all" || c.checklist_type.toLowerCase() === selectedType.toLowerCase();
      const matchStatus = selectedStatus === "all" || c.status.toLowerCase() === selectedStatus.toLowerCase();

      const matchEmployee = !employeeFilter || c.employee_id === employeeFilter;
      return matchSearch && matchType && matchStatus && matchEmployee;
    });
  }, [instances, search, selectedType, selectedStatus, employeeFilter]);

  

  return (
    <PageContainer>
      <PageHeader
        actions={
          <div className="flex flex-wrap items-center gap-3">
            <Button
              className="font-bold text-xs shadow-sm tracking-wider uppercase h-10 px-5 rounded-2xl"
              icon={<Plus size={16} />}
              onClick={() => {
                setIsModalOpen(true);
              }}
              variant="secondary"
            >
              {t("lifecycle.actions.create", { defaultValue: "KHỞI TẠO QUY TRÌNH" })}
            </Button>
          </div>
        }
        icon={<ListTodo size={24} />}
        subtitle={t("lifecycle.subtitle", {
          defaultValue: "Quản lý tập trung tiến độ onboarding, thử việc, điều chuyển và bàn giao nghỉ việc của nhân sự",
        })}
        title={t("lifecycle.title", { defaultValue: "Quản lý Quy trình & Vòng đời Nhân sự" })}
      />

      {!!employeeFilter && (
        <div className="flex items-center gap-2" data-testid="employee-filter-chip">
          <span className="rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs font-semibold text-primary-700 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-300">
            {t("employees.details.filtered_employee", { defaultValue: "Đang lọc theo nhân viên" })}
          </span>
          <Link
            aria-label={t("common.clear_filter", { defaultValue: "Bỏ bộ lọc" })}
            className="rounded-full p-1 text-slate-500 hover:bg-slate-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:hover:bg-slate-800"
            search={{}}
            to="/lifecycle"
          >
            <X aria-hidden size={14} />
          </Link>
        </div>
      )}

      {/* ── KPI Metric Cards ────────────────────────────────────────── */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("lifecycle.metrics.total", { defaultValue: "Tổng quy trình" })}
            </span>
            <Users size={16} />
          </div>
          <Typography className="text-2xl font-black text-slate-800 dark:text-slate-100" variant="h3">
            {metrics.total}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-blue-500 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("lifecycle.metrics.in_progress", { defaultValue: "Đang tiến hành" })}
            </span>
            <Clock size={16} />
          </div>
          <Typography className="text-2xl font-black text-blue-600 dark:text-blue-400" variant="h3">
            {metrics.inProgress}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-emerald-500 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("lifecycle.metrics.onboarding", { defaultValue: "Đang Onboarding" })}
            </span>
            <UserPlus size={16} />
          </div>
          <Typography className="text-2xl font-black text-emerald-600 dark:text-emerald-400" variant="h3">
            {metrics.onboarding}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-rose-500 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("lifecycle.metrics.offboarding", { defaultValue: "Đang Bàn giao" })}
            </span>
            <UserMinus size={16} />
          </div>
          <Typography className="text-2xl font-black text-rose-600 dark:text-rose-400" variant="h3">
            {metrics.offboarding}
          </Typography>
        </div>
      </div>

      {/* ── Filters ──────────────────────────────────────────────────── */}
      <LifecycleFilters
        onClearFilters={() => {
          setSearch("");
          setSelectedType("all");
          setSelectedStatus("all");
        }}
        onSearchChange={setSearch}
        onStatusChange={setSelectedStatus}
        onTypeChange={setSelectedType}
        search={search}
        selectedStatus={selectedStatus}
        selectedType={selectedType}
      />

      {/* ── Table Container ─────────────────────────────────────────── */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] shadow-xs overflow-hidden relative min-h-[400px]">
        <LifecycleTable
          instances={filteredInstances}
          isLoading={isLoading}
          onClearFilters={() => {
            setSearch("");
            setSelectedType("all");
            setSelectedStatus("all");
          }}
          search={search}
          selectedStatus={selectedStatus}
          selectedType={selectedType}
        />
      </div>

      {/* ── Assign Modal ────────────────────────────────────────────── */}
      {isModalOpen ? (
        <LifecycleAssignModal isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false);
        }}  />
      ) : null}
    </PageContainer>
  );
}
