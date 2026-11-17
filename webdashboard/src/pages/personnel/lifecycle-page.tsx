import { Link, useSearch } from "@tanstack/react-router";
import { Clock, ListTodo, Plus, UserMinus, UserPlus, Users, X } from "lucide-react";
import type { ReactNode } from "react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import LifecycleAssignModal from "@/components/features/employees/lifecycle-assign-modal";
import { LifecycleFilters } from "@/components/features/personnel/lifecycle-filters";
import { LifecycleTable } from "@/components/features/personnel/lifecycle-table";
import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { Typography } from "@/components/ui/typography";
import { useOrganizationLifecycleInstances } from "@/hooks/use-lifecycle";
import { type LifecycleInstanceSummary } from "@/lib/api";

interface LifecycleMetrics {
  inProgress: number;
  offboarding: number;
  onboarding: number;
  total: number;
}

interface MetricCardProps {
  icon: ReactNode;
  iconClassName: string;
  label: string;
  value: number;
  valueClassName: string;
}

interface LifecycleMetricsCardsProps {
  metrics: LifecycleMetrics;
}

interface LifecycleFilterChipProps {
  isVisible: boolean;
}

interface LifecyclePageHeaderProps {
  onCreate: () => void;
}

interface LifecycleFilterState {
  employeeFilter?: string;
  search: string;
  selectedStatus: string;
  selectedType: string;
}

function getLifecycleMetrics(instances: LifecycleInstanceSummary[]): LifecycleMetrics {
  return instances.reduce<LifecycleMetrics>(
    (metrics, instance) => {
      metrics.total += 1;
      if (instance.status !== "in_progress") return metrics;

      metrics.inProgress += 1;
      const checklistType = instance.checklist_type.toLowerCase();
      if (checklistType === "onboarding") metrics.onboarding += 1;
      if (checklistType === "offboarding") metrics.offboarding += 1;
      return metrics;
    },
    { inProgress: 0, offboarding: 0, onboarding: 0, total: 0 }
  );
}

function filterLifecycleInstances(
  instances: LifecycleInstanceSummary[],
  { employeeFilter, search, selectedStatus, selectedType }: LifecycleFilterState
): LifecycleInstanceSummary[] {
  const query = search.toLowerCase();

  return instances.filter((instance) => {
    const checklistType = instance.checklist_type.toLowerCase();
    const matchesSearch =
      !search ||
      instance.employee_name.toLowerCase().includes(query) ||
      instance.employee_code.toLowerCase().includes(query) ||
      (instance.department_name?.toLowerCase().includes(query) ?? false) ||
      (instance.template_name?.toLowerCase().includes(query) ?? false) ||
      checklistType.includes(query);
    const matchesType = selectedType === "all" || checklistType === selectedType.toLowerCase();
    const matchesStatus = selectedStatus === "all" || instance.status.toLowerCase() === selectedStatus.toLowerCase();
    const matchesEmployee = !employeeFilter || instance.employee_id === employeeFilter;

    return matchesSearch && matchesType && matchesStatus && matchesEmployee;
  });
}

function MetricCard({ icon, iconClassName, label, value, valueClassName }: MetricCardProps) {
  return (
    <div className="rounded-3xl border border-slate-100 bg-white p-5 shadow-xs dark:border-slate-800 dark:bg-slate-900">
      <div className={`mb-2 flex items-center justify-between ${iconClassName}`}>
        <span className="text-[11px] font-bold uppercase tracking-wider">{label}</span>
        {icon}
      </div>
      <Typography className={`text-2xl font-black ${valueClassName}`} variant="h3">
        {value}
      </Typography>
    </div>
  );
}

function LifecycleMetricsCards({ metrics }: LifecycleMetricsCardsProps) {
  const { t } = useTranslation();

  return (
    <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
      <MetricCard
        icon={<Users size={16} />}
        iconClassName="text-slate-400"
        label={t("lifecycle.metrics.total", { defaultValue: "Tổng quy trình" })}
        value={metrics.total}
        valueClassName="text-slate-800 dark:text-slate-100"
      />
      <MetricCard
        icon={<Clock size={16} />}
        iconClassName="text-blue-500"
        label={t("lifecycle.metrics.in_progress", { defaultValue: "Đang tiến hành" })}
        value={metrics.inProgress}
        valueClassName="text-blue-600 dark:text-blue-400"
      />
      <MetricCard
        icon={<UserPlus size={16} />}
        iconClassName="text-emerald-500"
        label={t("lifecycle.metrics.onboarding", { defaultValue: "Đang Onboarding" })}
        value={metrics.onboarding}
        valueClassName="text-emerald-600 dark:text-emerald-400"
      />
      <MetricCard
        icon={<UserMinus size={16} />}
        iconClassName="text-rose-500"
        label={t("lifecycle.metrics.offboarding", { defaultValue: "Đang Bàn giao" })}
        value={metrics.offboarding}
        valueClassName="text-rose-600 dark:text-rose-400"
      />
    </div>
  );
}

function LifecyclePageHeader({ onCreate }: LifecyclePageHeaderProps) {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <Button
          className="h-10 rounded-2xl px-5 text-xs font-bold uppercase tracking-wider shadow-sm"
          icon={<Plus size={16} />}
          onClick={onCreate}
          variant="secondary"
        >
          {t("lifecycle.actions.create", { defaultValue: "KHỞI TẠO QUY TRÌNH" })}
        </Button>
      }
      icon={<ListTodo size={24} />}
      subtitle={t("lifecycle.subtitle", {
        defaultValue: "Quản lý tập trung tiến độ onboarding, thử việc, điều chuyển và bàn giao nghỉ việc của nhân sự",
      })}
      title={t("lifecycle.title", { defaultValue: "Quản lý Quy trình & Vòng đời Nhân sự" })}
    />
  );
}

function LifecycleFilterChip({ isVisible }: LifecycleFilterChipProps) {
  const { t } = useTranslation();

  if (!isVisible) return null;

  return (
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
  );
}

export default function PersonnelLifecyclePage() {
  const searchParams = useSearch({ strict: false });
  const employeeFilter = searchParams.employeeId;
  const [search, setSearch] = useState(searchParams.q ?? "");
  const [selectedType, setSelectedType] = useState("all");
  const [selectedStatus, setSelectedStatus] = useState("all");
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { data: instances = [], isLoading } = useOrganizationLifecycleInstances();
  const metrics = useMemo(() => getLifecycleMetrics(instances), [instances]);
  const filteredInstances = useMemo(
    () => filterLifecycleInstances(instances, { employeeFilter, search, selectedStatus, selectedType }),
    [employeeFilter, instances, search, selectedStatus, selectedType]
  );

  const clearFilters = () => {
    setSearch("");
    setSelectedType("all");
    setSelectedStatus("all");
  };

  return (
    <PageContainer>
      <LifecyclePageHeader onCreate={() => { setIsModalOpen(true); }} />
      <LifecycleFilterChip isVisible={Boolean(employeeFilter)} />
      <LifecycleMetricsCards metrics={metrics} />
      <LifecycleFilters
        onClearFilters={clearFilters}
        onSearchChange={setSearch}
        onStatusChange={setSelectedStatus}
        onTypeChange={setSelectedType}
        search={search}
        selectedStatus={selectedStatus}
        selectedType={selectedType}
      />
      <div className="relative min-h-[400px] overflow-hidden rounded-2xl border border-slate-100 bg-white shadow-xs dark:border-slate-800 dark:bg-slate-900 sm:rounded-[2.5rem]">
        <LifecycleTable
          instances={filteredInstances}
          isLoading={isLoading}
          onClearFilters={clearFilters}
          search={search}
          selectedStatus={selectedStatus}
          selectedType={selectedType}
        />
      </div>
      {isModalOpen ? (
        <LifecycleAssignModal isOpen={isModalOpen} onClose={() => { setIsModalOpen(false); }} />
      ) : null}
    </PageContainer>
  );
}
