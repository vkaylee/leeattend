import type { TFunction } from "i18next";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Calculator } from "lucide-react";
import React from "react";

import type { Department } from "@/lib/api";

import TemporalConfigTable from "@/components/common/temporal-config-table";
import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { fetchDepartmentPayrollConfigs, setDepartmentPayrollConfig } from "@/lib/api";

export interface DepartmentConfigModalProps {
  editingDept: Department | null;
  isOpen: boolean;
  onClose: () => void;
  t: TFunction;
  wsId: string;
}

// Internal type matching what the component expects
interface TemporalConfigRecord {
  [key: string]: unknown;
  effective_date: string;
  hours_per_day?: number;
  id: string;
  std_working_days?: number;
}

export function DepartmentConfigModal({ editingDept, isOpen, onClose, wsId, t }: DepartmentConfigModalProps) {
  const queryClient = useQueryClient();

  const { data: deptConfigs = [], isLoading: isConfigsLoading } = useQuery({ 
    queryKey: ["dept-payroll-configs", editingDept?.id], 
    queryFn: () => fetchDepartmentPayrollConfigs(wsId, editingDept!.id), 
    enabled: isOpen && !!editingDept?.id 
  });

  const saveConfigMutation = useMutation({
    mutationFn: (data: { std_working_days: number; hours_per_day: number; effective_date: string }) => setDepartmentPayrollConfig(wsId, editingDept!.id, data),
    onSuccess: () => { void queryClient.invalidateQueries({ queryKey: ["dept-payroll-configs", editingDept?.id] }); },
  });

  return (
    <Modal icon={<Calculator size={24} />} isOpen={isOpen && Boolean(editingDept)} maxWidth="3xl" onClose={onClose} subtitle={editingDept?.name.toUpperCase()} title={t("payroll.settings.dept_title")}>
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <TemporalConfigTable 
          isLoading={isConfigsLoading} 
          onSave={async (data) => { await saveConfigMutation.mutateAsync(data); }} 
          records={deptConfigs as TemporalConfigRecord[]} 
          title={t("payroll.settings.std_days_history")} 
        />
        <div className="pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end"><Button onClick={onClose} size="lg" variant="secondary">{t("common.close").toUpperCase()}</Button></div>
      </div>
    </Modal>
  );
}
