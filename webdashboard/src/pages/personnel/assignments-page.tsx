import { Link, useSearch } from "@tanstack/react-router";
import { ArrowLeftRight, CheckCircle2, Clock, Plus, Users, X } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { EmployeeAssignmentWithEmployee } from "@/lib/api/employee-assignments";

import EmployeeAssignmentFormModal from "@/components/features/employees/employee-assignment-form-modal";
import { AssignmentsFilters } from "@/components/features/personnel/assignments-filters";
import { AssignmentsTable } from "@/components/features/personnel/assignments-table";
import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import { PageHeader } from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  useAssignments,
  useDeleteEmployeeAssignment,
} from "@/hooks/use-employee-assignments";

export default function PersonnelAssignmentsPage() {
  const { t } = useTranslation();
  const searchParams = useSearch({ strict: false });
  const toast = useToast();

  const { confirm, ConfirmationDialog } = useConfirm();

  const [search, setSearch] = useState(searchParams.q ?? "");
  const [selectedStatus, setSelectedStatus] = useState<string>("all");
  const employeeFilter = searchParams.employeeId;

  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingAssignment, setEditingAssignment] = useState<EmployeeAssignmentWithEmployee | null>(null);

  const { data: assignments = [], isLoading } = useAssignments();
  const deleteMutation = useDeleteEmployeeAssignment();

  // Metrics calculation
  const metrics = useMemo(() => {
    const total = assignments.length;
    const active = assignments.filter((a) => a.status === "active").length;
    const scheduled = assignments.filter((a) => a.status === "scheduled").length;
    const completed = assignments.filter((a) => a.status === "completed").length;
    return { total, active, scheduled, completed };
  }, [assignments]);

  // Filtering
  const filteredAssignments = useMemo(() => {
    const q = search.trim().toLowerCase();
    const matchesSearch = (a: EmployeeAssignmentWithEmployee) => {
      if (!q) return true;
      const haystack = [
        a.employee_name,
        a.employee_code,
        a.host_location_name,
        a.home_location_name,
        a.reason,
      ];
      return haystack.some((val) => val?.toLowerCase().includes(q));
    };

    return assignments.filter((a) => {
      const matchStatus = selectedStatus === "all" || a.status === selectedStatus;
      const matchEmployee = !employeeFilter || a.employee_id === employeeFilter;
      return matchStatus && matchEmployee && matchesSearch(a);
    });
  }, [assignments, search, selectedStatus, employeeFilter]);

  const handleDelete = async (assignment: EmployeeAssignmentWithEmployee) => {
    const isConfirmed = await confirm({
      title: t("assignments.delete_title", { defaultValue: "Xóa Lệnh Phái cử" }),
      message: t("assignments.delete_confirm", {
        defaultValue: `Bạn có chắc chắn muốn xóa lệnh phái cử của nhân sự ${assignment.employee_name} không?`,
      }),
      confirmLabel: t("common.delete", { defaultValue: "Xóa" }),
      cancelLabel: t("common.cancel", { defaultValue: "Hủy" }),
    });

    if (isConfirmed) {
      try {
        await deleteMutation.mutateAsync({
          employeeId: assignment.employee_id,
          assignmentId: assignment.id,
        });
        toast.success(t("assignments.delete_success", { defaultValue: "Đã xóa lệnh phái cử thành công." }));
      } catch (err: unknown) {
        const error = err as Error;
        toast.error(error.message || "Không thể xóa lệnh phái cử.");
      }
    }
  };



  return (
    <PageContainer>
      <PageHeader
        actions={
          <div className="flex flex-wrap items-center gap-3">
            <Button
              className="font-bold text-xs shadow-sm tracking-wider uppercase h-10 px-5 rounded-2xl"
              icon={<Plus size={16} />}
              onClick={() => {
                setEditingAssignment(null);
                setIsFormOpen(true);
              }}
              variant="secondary"
            >
              {t("assignments.actions.create", { defaultValue: "TẠO PHÁI CỬ MỚI" })}
            </Button>
          </div>
        }
        icon={<ArrowLeftRight size={24} />}
        subtitle={t("assignments.subtitle", {
          defaultValue: "Quản lý tập trung các đợt phái cử, luân chuyển và công tác của nhân sự toàn công ty",
        })}
        title={t("assignments.title", { defaultValue: "Quản lý Phái cử & Luân chuyển" })}
      />

      {employeeFilter ? (
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
      ) : null}

      {/* ── KPI Metric Cards ────────────────────────────────────────── */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("assignments.metrics.total", { defaultValue: "Tổng đợt phái cử" })}
            </span>
            <Users size={16} />
          </div>
          <Typography className="text-2xl font-black text-slate-800 dark:text-slate-100" variant="h3">
            {metrics.total}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-emerald-500 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("assignments.metrics.active", { defaultValue: "Đang hiệu lực" })}
            </span>
            <ArrowLeftRight size={16} />
          </div>
          <Typography className="text-2xl font-black text-emerald-600 dark:text-emerald-400" variant="h3">
            {metrics.active}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-amber-500 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("assignments.metrics.scheduled", { defaultValue: "Đã lên lịch" })}
            </span>
            <Clock size={16} />
          </div>
          <Typography className="text-2xl font-black text-amber-600 dark:text-amber-400" variant="h3">
            {metrics.scheduled}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("assignments.metrics.completed", { defaultValue: "Đã hoàn thành" })}
            </span>
            <CheckCircle2 size={16} />
          </div>
          <Typography className="text-2xl font-black text-slate-600 dark:text-slate-300" variant="h3">
            {metrics.completed}
          </Typography>
        </div>
      </div>

      {/* ── Filters ──────────────────────────────────────────────────── */}
      <AssignmentsFilters
        onClearFilters={() => {
          setSearch("");
          setSelectedStatus("all");
        }}
        onSearchChange={setSearch}
        onStatusChange={setSelectedStatus}
        search={search}
        selectedStatus={selectedStatus}
      />

      {/* ── Table Container ─────────────────────────────────────────── */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] shadow-xs overflow-hidden relative min-h-[400px]">
        <AssignmentsTable
          assignments={filteredAssignments}
          isLoading={isLoading}
          onClearFilters={() => {
            setSearch("");
            setSelectedStatus("all");
          }}
          onDelete={handleDelete}
          onEdit={(assignment) => {
            setEditingAssignment(assignment);
            setIsFormOpen(true);
          }}
          search={search}
          selectedStatus={selectedStatus}
        />
      </div>

      {/* ── Dual-Mode Assignment Form Modal ────────────────────────── */}
      {isFormOpen ? (
        <EmployeeAssignmentFormModal
          editingAssignment={editingAssignment}
          isOpen={isFormOpen}
          onClose={() => {
            setIsFormOpen(false);
            setEditingAssignment(null);
          }}
        />
      ) : null}

      <ConfirmationDialog />
    </PageContainer>
  );
}
