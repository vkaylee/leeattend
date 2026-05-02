import { FileText } from 'lucide-react';
import React from 'react';
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
      color: "bg-teal-50 dark:bg-teal-950",
      isActive: activeSource === 'individual'
    },
    { 
      label: t("payroll.traceability.department_policy", "Department Policy"), 
      priority: 50, 
      color: "bg-blue-50 dark:bg-blue-950",
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
      explanation={t("payroll.traceability.hierarchy_explanation", "Payroll settings are resolved from Individual > Department > Workspace.")}
      icon={<FileText size={14} />}
      steps={getSteps()}
      title={t("payroll.traceability.hierarchy_title", "Policy Hierarchy")}
    />
  );
};
