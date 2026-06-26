import {
  BarChart3,
  Building2,
  Briefcase,
  Clock,
  CreditCard,
  Gift,
  PieChart,
  Timer,
  TrendingUp,
  UserMinus,
  Users,
  UserX,
  Zap,
} from "lucide-react";

import type { WidgetId } from "@/hooks/use-dashboard-layout";

export type ChartCategory = "attendance" | "workforce" | "performance" | "summary";

export interface ChartMeta {
  category: ChartCategory;
  icon: React.ElementType;
  descriptionKey: string;
}

export const CHART_METADATA: Record<WidgetId, ChartMeta> = {
  kpi_cards:           { category: "attendance",  icon: BarChart3,   descriptionKey: "dashboard.meta.kpi_cards" },
  donut_chart:         { category: "attendance",  icon: PieChart,    descriptionKey: "dashboard.meta.donut_chart" },
  quick_stats:         { category: "attendance",  icon: Zap,         descriptionKey: "dashboard.meta.quick_stats" },
  attendance_trend:    { category: "attendance",  icon: TrendingUp,  descriptionKey: "dashboard.meta.attendance_trend" },
  department_chart:    { category: "attendance",  icon: Building2,   descriptionKey: "dashboard.meta.department_chart" },
  peak_times:          { category: "attendance",  icon: Clock,       descriptionKey: "dashboard.meta.peak_times" },
  leave_types:         { category: "workforce",   icon: Briefcase,   descriptionKey: "dashboard.meta.leave_types" },
  recruitment_sources: { category: "workforce",   icon: Users,       descriptionKey: "dashboard.meta.recruitment_sources" },
  turnover_analytics:  { category: "workforce",   icon: UserMinus,   descriptionKey: "dashboard.meta.turnover_analytics" },
  unexcused_absences:  { category: "workforce",   icon: UserX,       descriptionKey: "dashboard.meta.unexcused_absences" },
  overtime_trend:      { category: "performance", icon: Timer,       descriptionKey: "dashboard.meta.overtime_trend" },
  lateness_by_day:     { category: "performance", icon: CreditCard,  descriptionKey: "dashboard.meta.lateness_by_day" },
  promo_cards:         { category: "summary",     icon: Gift,        descriptionKey: "dashboard.meta.promo_cards" },
};

export const CATEGORY_ORDER: ChartCategory[] = ["attendance", "workforce", "performance", "summary"];

export const CATEGORY_LABEL_KEYS: Record<ChartCategory, string> = {
  attendance:  "dashboard.categories.attendance",
  workforce:   "dashboard.categories.workforce",
  performance: "dashboard.categories.performance",
  summary:     "dashboard.categories.summary",
};
