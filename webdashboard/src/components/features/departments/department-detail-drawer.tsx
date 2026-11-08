import type { TFunction } from "i18next";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Pencil, Trash2, Calculator, UserCircle, Users } from "lucide-react";
import React, { useState } from "react";

import type { Department, Employee } from "@/lib/api";

import TemporalConfigTable from "@/components/common/temporal-config-table";
import { CustomFieldSection } from "@/components/custom-fields/custom-field-section";
import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";
import { fetchDepartmentPayrollConfigs, setDepartmentPayrollConfig } from "@/lib/api";
import { cn } from "@/lib/utils";

export interface DepartmentDetailDrawerProps {
  dept: Department | null;
  employees: Employee[];
  isOpen: boolean;
  onClose: () => void;
  onConfig: (dept: Department) => void;
  onDelete: (dept: Department) => void;
  onEdit: (dept: Department) => void;
  t: TFunction;
}

interface TemporalConfigRecord {
  effective_date: string;
  hours_per_day: number;
  id: string;
  is_virtual?: boolean;
  max_ot_hours_per_day?: number | null;
  max_ot_hours_per_month?: number | null;
  std_working_days: number;
}

interface EmployeeGridProps {
  deptEmployees: Employee[];
  t: TFunction;
}

function EmployeeGrid({ deptEmployees, t }: EmployeeGridProps) {
  if (deptEmployees.length === 0) {
    return (
      <div className="py-20 text-center text-slate-400 dark:text-slate-500 font-bold uppercase tracking-widest text-[10px]">
        {t("departments.drawer.empty_employees")}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      {deptEmployees.map((emp) => (
        <div
          className="p-6 bg-slate-50 dark:bg-slate-800/20 border border-slate-100 dark:border-slate-800/80 rounded-[2rem] flex items-center gap-4 group hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-primary-100/10 transition-all duration-300 relative overflow-hidden"
          key={emp.id}
        >
          <div className="w-12 h-12 rounded-2xl bg-primary-50 dark:bg-primary-950/50 flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-base border border-primary-100 dark:border-primary-800/30">
            {emp.full_name[0].toUpperCase()}
          </div>
          <div>
            <div className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight leading-tight">
              {emp.full_name}
            </div>
            <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold mt-1 tracking-widest uppercase opacity-75">
              {emp.employee_id_number}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

interface SettingsSectionProps {
  deptConfigs: TemporalConfigRecord[];
  deptId: string;
  isConfigsLoading: boolean;
  onSave: (data: { effective_date: string; hours_per_day: number; std_working_days: number }) => Promise<void>;
  t: TFunction;
}

function SettingsSection({ deptId, deptConfigs, isConfigsLoading, onSave, t }: SettingsSectionProps) {
  return (
    <div className="space-y-10">
      <TemporalConfigTable
        isLoading={isConfigsLoading}
        onSave={onSave}
        records={deptConfigs}
        title={t("payroll.settings.std_days_history")}
      />
      <CustomFieldSection entityId={deptId} entityType="department" />
    </div>
  );
}

interface MetadataSectionProps {
  activeEmployeeCount: number;
  managerName: string;
  t: TFunction;
}

function MetadataSection({ activeEmployeeCount, managerName, t }: MetadataSectionProps) {
  return (
    <div className="grid grid-cols-2 gap-6 pb-6 border-b border-slate-100 dark:border-slate-800">
      <div className="flex items-center gap-4">
        <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-400 dark:text-slate-500">
          <UserCircle size={20} />
        </div>
        <div>
          <span className="block text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("departments.manager_label")}</span>
          <span className="font-bold text-slate-700 dark:text-slate-200 text-xs uppercase">{managerName}</span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <div className="w-10 h-10 rounded-xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-400 dark:text-slate-500">
          <Users size={20} />
        </div>
        <div>
          <span className="block text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("departments.active_employees_label")}</span>
          <span className="font-bold text-slate-700 dark:text-slate-200 text-xs">{activeEmployeeCount}</span>
        </div>
      </div>
    </div>
  );
}

export function DepartmentDetailDrawer({
  isOpen,
  onClose,
  dept,
  employees,
  onEdit,
  onConfig,
  onDelete,
  t,}: DepartmentDetailDrawerProps) {
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<"employees" | "settings">("employees");

  // Fetch standard days history for settings tab
  const { data: deptConfigs = [], isLoading: isConfigsLoading } = useQuery({
    queryKey: ["dept-payroll-configs", dept?.id],
    queryFn: () => fetchDepartmentPayrollConfigs(dept?.id ?? ""),
    enabled: isOpen && !!dept?.id,
  });

  const saveConfigMutation = useMutation({
    mutationFn: (data: { effective_date: string; hours_per_day: number; std_working_days: number }) =>
      setDepartmentPayrollConfig(dept?.id ?? "", {
        effective_date: data.effective_date,
        hours_per_day: data.hours_per_day.toString(),
        std_working_days: data.std_working_days.toString(),
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["dept-payroll-configs", dept?.id] });
    },
  });

  if (!dept) return null;

  // Filter employees belonging to this department
  const deptEmployees = employees.filter((e) => e.department_name === dept.name);
  const manager = employees.find((e) => e.id === dept.manager_id);
  const managerName = manager ? manager.full_name : t("departments.not_assigned").toUpperCase();

  const getTabClass = (tab: "employees" | "settings") => cn(
    "px-6 py-2.5 rounded-[18px] text-[10px] font-bold uppercase tracking-widest transition-all shadow-none border border-transparent",
    activeTab === tab
      ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
      : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent"
  );

  const headerExtra = (
    <div className="flex gap-2">
      <Button
        className="hover:text-emerald-600 dark:hover:text-emerald-400"
        icon={<Calculator size={16} />}
        onClick={() => { onConfig(dept); }}
        size="sm"
        title={t("payroll.settings.title")}
        variant="outline"
      />
      <Button
        className="hover:text-primary-600 dark:hover:text-primary-400"
        icon={<Pencil size={16} />}
        onClick={() => { onEdit(dept); }}
        size="sm"
        title={t("common.edit")}
        variant="outline"
      />
      <Button
        className="hover:text-rose-600 dark:hover:text-rose-455"
        icon={<Trash2 size={16} />}
        onClick={() => { onDelete(dept); }}
        size="sm"
        title={t("common.delete")}
        variant="outline"
      />
    </div>
  );

  return (
    <Sheet
      className="!max-w-4xl"
      headerExtra={headerExtra}
      isOpen={isOpen}
      onClose={onClose}
      subtitle={dept.description ?? t("common.no_description")}
      title={dept.name}
    >
      <div className="space-y-8 h-full flex flex-col">
        <MetadataSection
          activeEmployeeCount={dept.active_employee_count}
          managerName={managerName}
          t={t}
        />

        {/* Tab Toggle buttons */}
        <div className="flex bg-slate-50 dark:bg-slate-900/50 p-1.5 rounded-[22px] border border-slate-200 dark:border-slate-700/50 shadow-inner w-fit">
          <Button
            className={getTabClass("employees")}
            onClick={() => { setActiveTab("employees"); }}
            variant="ghost"
          >
            {t("departments.drawer.tab_employees")}
          </Button>
          <Button
            className={getTabClass("settings")}
            onClick={() => { setActiveTab("settings"); }}
            variant="ghost"
          >
            {t("departments.drawer.tab_settings")}
          </Button>
        </div>

        {/* Tab Contents */}
        <div className="flex-1">
          {activeTab === "employees" ? (
            <EmployeeGrid deptEmployees={deptEmployees} t={t} />
          ) : (
            <SettingsSection
              deptConfigs={deptConfigs as TemporalConfigRecord[]}
              deptId={dept.id}
              isConfigsLoading={isConfigsLoading}
              onSave={async (data) => {
                await saveConfigMutation.mutateAsync(data);
              }}
              t={t}
            />
          )}
        </div>
      </div>
    </Sheet>
  );
}
