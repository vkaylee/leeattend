import React from 'react';
import { FileText } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { HierarchyLegend, HierarchyStep } from '../common/HierarchyLegend';

interface PayrollPriorityLegendProps {
  activeSource?: string; // "individual", "department", "workspace", "system"
}

export const PayrollPriorityLegend: React.FC<PayrollPriorityLegendProps> = ({ activeSource }) => {
  const { t } = useTranslation();

  const getSteps = (): HierarchyStep[] => [
    { 
      label: t("payroll.traceability.individual_policy", "Individual Policy"), 
      priority: 100, 
      color: "bg-purple-500",
      isActive: activeSource === 'individual'
    },
    { 
      label: t("payroll.traceability.department_policy", "Department Policy"), 
      priority: 50, 
      color: "bg-blue-500",
      isActive: activeSource === 'department'
    },
    { 
      label: t("payroll.traceability.workspace_default", "Workspace Default"), 
      priority: 10, 
      color: "bg-slate-400",
      isActive: activeSource === 'workspace' || activeSource === 'system'
    },
  ];

  return (
    <HierarchyLegend
      steps={getSteps()}
      title={t("payroll.traceability.hierarchy_title", "Policy Hierarchy")}
      icon={<FileText size={14} />}
      explanation={t("payroll.traceability.hierarchy_explanation", "Payroll settings are resolved from Individual > Department > Workspace.")}
    />
  );
};
