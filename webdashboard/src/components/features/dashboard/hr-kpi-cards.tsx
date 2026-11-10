import { Users2, UserCheck, Clock, UserX, CalendarClock, AlertTriangle } from "lucide-react";

import { type TFunction } from "i18next";

import CollapsibleSection from "@/components/features/dashboard/collapsible-section";
import StaggerChildren from "@/components/ui/stagger-children";
import StatCard from "@/components/ui/stat-card";
import { type OrganizationStats } from "@/hooks/use-dashboard";

interface HrKpiCardsProps {
  hrStats: OrganizationStats | undefined;
  isCollapsed: boolean;
  isLoading: boolean;
  onToggle: (id: string) => void;
  t: TFunction;
}

export function HrKpiCards({ hrStats, isLoading, isCollapsed, onToggle, t }: HrKpiCardsProps) {
  const stats = [
    { label: "total_employees", value: hrStats?.total_employees, icon: Users2, color: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800" },
    { label: "present_today", value: hrStats?.present_today, icon: UserCheck, color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" },
    { label: "late_today", value: hrStats?.late_today, icon: Clock, color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" },
    { label: "absent_today", value: hrStats?.absent_today, icon: UserX, color: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800" },
    { label: "on_leave_today", value: hrStats?.on_leave_today, icon: CalendarClock, color: "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800" },
    { label: "pending_requests", value: hrStats?.pending_leave_requests, icon: AlertTriangle, color: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-100 dark:border-orange-800" },
  ];

  return (
    <CollapsibleSection
      id="summary"
      isCollapsed={isCollapsed}
      onToggle={onToggle}
      title={t("dashboard.sections.summary")}
    >
      <StaggerChildren className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4 sm:gap-6" delay={80}>
        {stats.map((stat, index) => (
          <StatCard
            colorClasses={stat.color}
            icon={stat.icon}
            isLoading={isLoading}
            key={index}
            label={t(`dashboard.hr.${stat.label}`)}
            size="sm"
            value={stat.value ?? 0}
          />
        ))}
      </StaggerChildren>
    </CollapsibleSection>
  );
}
