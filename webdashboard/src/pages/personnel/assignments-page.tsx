import { Link, useSearch } from "@tanstack/react-router";
import { ArrowLeftRight, CheckCircle2, Clock, Plus, Users, X } from "lucide-react";
import type { ReactNode } from "react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { EmployeeAssignmentWithEmployee } from "@/lib/api/employee-assignments";

import EmployeeAssignmentFormModal from "@/components/features/employees/employee-assignment-form-modal";
import { AssignmentsFilters } from "@/components/features/personnel/assignments-filters";
import { AssignmentsTable } from "@/components/features/personnel/assignments-table";
import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  useDeleteEmployeeAssignment,
  useAssignments,
} from "@/hooks/use-employee-assignments";

interface AssignmentMetrics {
  active: number;
  completed: number;
  scheduled: number;
  total: number;
}

interface MetricCardProps {
  icon: ReactNode;
  iconClassName: string;
  label: string;
  value: number;
  valueClassName: string;
}

interface AssignmentMetricsCardsProps {
  metrics: AssignmentMetrics;
}

interface AssignmentFilterChipProps {
  isVisible: boolean;
}

interface AssignmentPageHeaderProps {
  onCreate: () => void;
}

interface AssignmentFormProps {
  editingAssignment: EmployeeAssignmentWithEmployee | null;
  isOpen: boolean;
  onClose: () => void;
}

function getAssignmentMetrics(assignments: EmployeeAssignmentWithEmployee[]): AssignmentMetrics {
  return assignments.reduce<AssignmentMetrics>(
    (metrics, assignment) => {
      metrics.total += 1;
      if (assignment.status === "active") metrics.active += 1;
      if (assignment.status === "scheduled") metrics.scheduled += 1;
      if (assignment.status === "completed") metrics.completed += 1;
      return metrics;
    },
    { active: 0, completed: 0, scheduled: 0, total: 0 }
  );
}

function filterAssignments(
  assignments: EmployeeAssignmentWithEmployee[],
  search: string,
  selectedStatus: string,
  employeeFilter?: string
): EmployeeAssignmentWithEmployee[] {
  const query = search.toLowerCase();

  return assignments.filter((assignment) => {
    const matchesSearch =
      !search ||
      assignment.employee_name.toLowerCase().includes(query) ||
      (assignment.employee_code?.toLowerCase().includes(query) ?? false) ||
      (assignment.host_location_name?.toLowerCase().includes(query) ?? false) ||
      (assignment.home_location_name?.toLowerCase().includes(query) ?? false) ||
      (assignment.reason?.toLowerCase().includes(query) ?? false);
    const matchesStatus = selectedStatus === "all" || assignment.status === selectedStatus;
    const matchesEmployee = !employeeFilter || assignment.employee_id === employeeFilter;

    return matchesSearch && matchesStatus && matchesEmployee;
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

function AssignmentMetricsCards({ metrics }: AssignmentMetricsCardsProps) {
  const { t } = useTranslation();

  return (
    <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
      <MetricCard
        icon={<Users size={16} />}
        iconClassName="text-slate-400"
        label={t("assignments.metrics.total", { defaultValue: "Tổng đợt phái cử" })}
        value={metrics.total}
        valueClassName="text-slate-800 dark:text-slate-100"
      />
      <MetricCard
        icon={<ArrowLeftRight size={16} />}
        iconClassName="text-emerald-500"
        label={t("assignments.metrics.active", { defaultValue: "Đang hiệu lực" })}
        value={metrics.active}
        valueClassName="text-emerald-600 dark:text-emerald-400"
      />
      <MetricCard
        icon={<Clock size={16} />}
        iconClassName="text-amber-500"
        label={t("assignments.metrics.scheduled", { defaultValue: "Đã lên lịch" })}
        value={metrics.scheduled}
        valueClassName="text-amber-600 dark:text-amber-400"
      />
      <MetricCard
        icon={<CheckCircle2 size={16} />}
        iconClassName="text-slate-400"
        label={t("assignments.metrics.completed", { defaultValue: "Đã hoàn thành" })}
        value={metrics.completed}
        valueClassName="text-slate-600 dark:text-slate-300"
      />
    </div>
  );
}

function AssignmentPageHeader({ onCreate }: AssignmentPageHeaderProps) {
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
          {t("assignments.actions.create", { defaultValue: "TẠO PHÁI CỬ MỚI" })}
        </Button>
      }
      icon={<ArrowLeftRight size={24} />}
      subtitle={t("assignments.subtitle", {
        defaultValue: "Quản lý tập trung các đợt phái cử, luân chuyển và công tác của nhân sự toàn công ty",
      })}
      title={t("assignments.title", { defaultValue: "Quản lý Phái cử & Luân chuyển" })}
    />
  );
}

function AssignmentFilterChip({ isVisible }: AssignmentFilterChipProps) {
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
        to="/assignments"
      >
        <X aria-hidden size={14} />
      </Link>
    </div>
  );
}

function AssignmentForm({ editingAssignment, isOpen, onClose }: AssignmentFormProps) {
  if (!isOpen) return null;

  return (
    <EmployeeAssignmentFormModal
      editingAssignment={editingAssignment}
      isOpen={isOpen}
      onClose={onClose}
    />
  );
}

export default function PersonnelAssignmentsPage() {
  const { t } = useTranslation();
  const searchParams = useSearch({ strict: false });
  const toast = useToast();
  const { confirm, ConfirmationDialog } = useConfirm();
  const [search, setSearch] = useState(searchParams.q ?? "");
  const [selectedStatus, setSelectedStatus] = useState("all");
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingAssignment, setEditingAssignment] = useState<EmployeeAssignmentWithEmployee | null>(null);
  const employeeFilter = searchParams.employeeId;
  const { data: assignments = [], isLoading } = useAssignments();
  const deleteMutation = useDeleteEmployeeAssignment();
  const metrics = useMemo(() => getAssignmentMetrics(assignments), [assignments]);
  const filteredAssignments = useMemo(
    () => filterAssignments(assignments, search, selectedStatus, employeeFilter),
    [assignments, employeeFilter, search, selectedStatus]
  );

  const clearFilters = () => {
    setSearch("");
    setSelectedStatus("all");
  };

  const handleDelete = async (assignment: EmployeeAssignmentWithEmployee) => {
    const isConfirmed = await confirm({
      title: t("assignments.delete_title", { defaultValue: "Xóa Lệnh Phái cử" }),
      message: t("assignments.delete_confirm", {
        defaultValue: `Bạn có chắc chắn muốn xóa lệnh phái cử của nhân sự ${assignment.employee_name} không?`,
      }),
      confirmLabel: t("common.delete", { defaultValue: "Xóa" }),
      cancelLabel: t("common.cancel", { defaultValue: "Hủy" }),
    });

    if (!isConfirmed) return;

    try {
      await deleteMutation.mutateAsync({
        employeeId: assignment.employee_id,
        assignmentId: assignment.id,
      });
      toast.success(t("assignments.delete_success", { defaultValue: "Đã xóa lệnh phái cử thành công." }));
    } catch (error: unknown) {
      toast.error(error instanceof Error && error.message ? error.message : "Không thể xóa lệnh phái cử.");
    }
  };

  const openCreateForm = () => {
    setEditingAssignment(null);
    setIsFormOpen(true);
  };

  const openEditForm = (assignment: EmployeeAssignmentWithEmployee) => {
    setEditingAssignment(assignment);
    setIsFormOpen(true);
  };

  const closeForm = () => {
    setIsFormOpen(false);
    setEditingAssignment(null);
  };

  return (
    <PageContainer>
      <AssignmentPageHeader onCreate={openCreateForm} />
      <AssignmentFilterChip isVisible={Boolean(employeeFilter)} />
      <AssignmentMetricsCards metrics={metrics} />
      <AssignmentsFilters
        onClearFilters={clearFilters}
        onSearchChange={setSearch}
        onStatusChange={setSelectedStatus}
        search={search}
        selectedStatus={selectedStatus}
      />
      <div className="relative min-h-[400px] overflow-hidden rounded-2xl border border-slate-100 bg-white shadow-xs dark:border-slate-800 dark:bg-slate-900 sm:rounded-[2.5rem]">
        <AssignmentsTable
          assignments={filteredAssignments}
          isLoading={isLoading}
          onClearFilters={clearFilters}
          onDelete={handleDelete}
          onEdit={openEditForm}
          search={search}
          selectedStatus={selectedStatus}
        />
      </div>
      <AssignmentForm editingAssignment={editingAssignment} isOpen={isFormOpen} onClose={closeForm} />
      <ConfirmationDialog />
    </PageContainer>
  );
}
