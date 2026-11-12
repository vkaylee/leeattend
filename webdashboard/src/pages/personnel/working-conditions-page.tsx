import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useSearch } from "@tanstack/react-router";
import { CheckCircle2, DollarSign, HeartHandshake, Plus, Users, X, XCircle } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import WorkingConditionFormModal from "@/components/features/employees/working-condition-form-modal";
import { WorkingConditionsFilters } from "@/components/features/personnel/working-conditions-filters";
import { WorkingConditionsTable } from "@/components/features/personnel/working-conditions-table";
import Button from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";
import PageHeader from "@/components/ui/page-header";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import { useOrganizationWorkingConditions } from "@/hooks/use-employees";
import {
  deleteWorkingCondition,
  fetchConditionTypes,
  type ConditionTypeDefinition,
} from "@/lib/api";
import type { WorkingConditionWithEmployee } from "@/lib/api/employee-conditions";

export default function PersonnelWorkingConditionsPage() {
  const { t, i18n } = useTranslation();
  const searchParams = useSearch({ strict: false }) as { employeeId?: string; q?: string };
  const employeeFilter = searchParams.employeeId;
  const toast = useToast();
  const queryClient = useQueryClient();
  
  const { confirm, ConfirmationDialog } = useConfirm();
  const currentLang = i18n.language.split("-")[0];

  const [search, setSearch] = useState(searchParams.q ?? "");
  const [selectedType, setSelectedType] = useState<string>("all");
  const [paymentFilter, setPaymentFilter] = useState<string>("all");
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { data: conditions = [], isLoading: isLoadingConditions } = useOrganizationWorkingConditions();

  const { data: typeDefs = [], isLoading: isLoadingTypes } = useQuery<ConditionTypeDefinition[]>({
    queryKey: ["condition-types"],
    queryFn: () => fetchConditionTypes(false),
  });

  const deleteMutation = useMutation({
    mutationFn: async ({ employeeId, conditionId }: { employeeId: string; conditionId: string }) => {
      await deleteWorkingCondition(employeeId, conditionId);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["working-conditions"] });
      toast.success(
        t("working_conditions.delete_success", { defaultValue: "Đã xóa điều kiện làm việc thành công" })
      );
    },
    onError: (err: unknown) => {
      const error = err as Error;
      toast.error(error.message || "Không thể xóa điều kiện làm việc");
    },
  });

  const todayStr = useMemo(() => new Date().toISOString().split("T")[0], []);

  // Metrics
  const metrics = useMemo(() => {
    const total = conditions.length;
    const active = conditions.filter((c) => {
      const isStarted = c.start_date <= todayStr;
      const isNotEnded = !c.end_date || c.end_date >= todayStr;
      return isStarted && isNotEnded;
    }).length;
    const paid = conditions.filter((c) => c.is_paid).length;
    const unpaid = conditions.filter((c) => !c.is_paid).length;
    return { total, active, paid, unpaid };
  }, [conditions, todayStr]);

  // Filtering
  const filteredConditions = useMemo(() => {
    return conditions.filter((c) => {
      const q = search.toLowerCase();
      const def = typeDefs.find((d) => d.code === c.type.toUpperCase());
      const displayName = def?.name_i18n?.[currentLang] || def?.name_i18n?.en || c.type;

      const matchSearch =
        !search ||
        c.employee_name.toLowerCase().includes(q) ||
        (c.employee_code?.toLowerCase().includes(q) ?? false) ||
        c.type.toLowerCase().includes(q) ||
        displayName.toLowerCase().includes(q);

      const matchType = selectedType === "all" || c.type.toUpperCase() === selectedType.toUpperCase();

      const matchPayment =
        paymentFilter === "all" ||
        (paymentFilter === "paid" && c.is_paid) ||
        (paymentFilter === "unpaid" && !c.is_paid);

      const matchEmployee = !employeeFilter || c.employee_id === employeeFilter;
      return matchSearch && matchType && matchPayment && matchEmployee;
    });
  }, [conditions, search, selectedType, paymentFilter, typeDefs, currentLang, employeeFilter]);

  const handleDelete = async (condition: WorkingConditionWithEmployee) => {
    const isConfirmed = await confirm({
      title: t("working_conditions.delete_title", { defaultValue: "Xóa Điều kiện Làm việc" }),
      message: t("working_conditions.delete_confirm", {
        defaultValue: `Bạn có chắc chắn muốn xóa chế độ điều kiện làm việc này của nhân sự ${condition.employee_name} không?`,
      }),
      confirmLabel: t("common.delete", { defaultValue: "Xóa" }),
      cancelLabel: t("common.cancel", { defaultValue: "Hủy" }),
    });

    if (isConfirmed) {
      deleteMutation.mutate({
        employeeId: condition.employee_id,
        conditionId: condition.id,
      });
    }
  };

  

  const isLoading = isLoadingConditions || isLoadingTypes;

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
              {t("working_conditions.actions.create", { defaultValue: "GÁN ĐIỀU KIỆN MỚI" })}
            </Button>
          </div>
        }
        icon={<HeartHandshake size={24} />}
        subtitle={t("working_conditions.subtitle", {
          defaultValue: "Quản lý tập trung các chế độ lao động đặc thù (con nhỏ, thai sản, độc hại, làm việc từ xa) toàn công ty",
        })}
        title={t("working_conditions.title", { defaultValue: "Quản lý Điều kiện Làm việc Đặc thù" })}
      />

      {employeeFilter && (
        <div className="flex items-center gap-2" data-testid="employee-filter-chip">
          <span className="rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs font-semibold text-primary-700 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-300">
            {t("employees.details.filtered_employee", { defaultValue: "Đang lọc theo nhân viên" })}
          </span>
          <Link
            aria-label={t("common.clear_filter", { defaultValue: "Bỏ bộ lọc" })}
            className="rounded-full p-1 text-slate-500 hover:bg-slate-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:hover:bg-slate-800"
            search={{}}
            to="/working-conditions"
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
              {t("working_conditions.metrics.total", { defaultValue: "Tổng hồ sơ áp dụng" })}
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
              {t("working_conditions.metrics.active", { defaultValue: "Đang hiệu lực" })}
            </span>
            <CheckCircle2 size={16} />
          </div>
          <Typography className="text-2xl font-black text-emerald-600 dark:text-emerald-400" variant="h3">
            {metrics.active}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-primary-500 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("working_conditions.metrics.paid", { defaultValue: "Có hưởng lương" })}
            </span>
            <DollarSign size={16} />
          </div>
          <Typography className="text-2xl font-black text-primary-600 dark:text-primary-400" variant="h3">
            {metrics.paid}
          </Typography>
        </div>

        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-5 rounded-3xl shadow-xs">
          <div className="flex items-center justify-between text-slate-400 mb-2">
            <span className="text-[11px] font-bold uppercase tracking-wider">
              {t("working_conditions.metrics.unpaid", { defaultValue: "Không hưởng lương" })}
            </span>
            <XCircle size={16} />
          </div>
          <Typography className="text-2xl font-black text-slate-600 dark:text-slate-300" variant="h3">
            {metrics.unpaid}
          </Typography>
        </div>
      </div>

      {/* ── Filters ──────────────────────────────────────────────────── */}
      <WorkingConditionsFilters
        onClearFilters={() => {
          setSearch("");
          setSelectedType("all");
          setPaymentFilter("all");
        }}
        onPaymentFilterChange={setPaymentFilter}
        onSearchChange={setSearch}
        onTypeChange={setSelectedType}
        paymentFilter={paymentFilter}
        search={search}
        selectedType={selectedType}
        typeDefs={typeDefs}
      />

      {/* ── Table Container ─────────────────────────────────────────── */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] shadow-xs overflow-hidden relative min-h-[400px]">
        <WorkingConditionsTable
          conditions={filteredConditions}
          isLoading={isLoading}
          onClearFilters={() => {
            setSearch("");
            setSelectedType("all");
            setPaymentFilter("all");
          }}
          onDelete={handleDelete}
          paymentFilter={paymentFilter}
          search={search}
          selectedType={selectedType}
          typeDefs={typeDefs}
        />
      </div>

      {/* ── Dual-Mode Working Condition Form Modal ─────────────────── */}
      {isModalOpen && (
        <WorkingConditionFormModal isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false);
        }}  />
      )}

      <ConfirmationDialog />
    </PageContainer>
  );
}
