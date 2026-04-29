import React from 'react';

import { Layers } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { HierarchyLegend } from '../common/HierarchyLegend';

export const PriorityLegend: React.FC = () => {
  const { t } = useTranslation();

  const priorities = [
    { label: t("shifts.traceability.p100_manual"), priority: 100, color: "bg-teal-50 dark:bg-teal-950", isActive: true },
    { label: t("shifts.traceability.p80_pattern"), priority: 80, color: "bg-blue-50 dark:bg-blue-950", isActive: true },
    { label: t("shifts.traceability.p60_rules"), priority: 60, color: "bg-indigo-50 dark:bg-indigo-950", isActive: true },
    { label: t("shifts.traceability.p40_dept_pattern"), priority: 40, color: "bg-slate-50 dark:bg-slate-8000", isActive: true },
    { label: t("shifts.traceability.p20_fixed"), priority: 20, color: "bg-emerald-50 dark:bg-emerald-950", isActive: true },
    { label: t("shifts.traceability.p10_dept_fixed"), priority: 10, color: "bg-slate-400", isActive: true },
  ];

  return (
    <HierarchyLegend
      steps={priorities}
      title={t("shifts.traceability.priority_hierarchy")}
      icon={<Layers size={14} />}
      explanation={t("shifts.traceability.hierarchy_explanation", "Rules are evaluated from top to bottom. Higher nodes override lower nodes.")}
    />
  );
};
