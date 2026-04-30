import {
    BarChart3,
    LineChart,
    LayoutGrid,
    Plus,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

import type { DashboardView } from "@/hooks/use-dashboard-layout";

interface DashboardViewTabsProps {
    activeView: DashboardView;
    onViewChange: (view: DashboardView) => void;
}

const TABS: {
    id: DashboardView;
    icon: React.ElementType;
    labelKey: string;
}[] = [
        { id: "overview", icon: LayoutGrid, labelKey: "dashboard.views.overview" },
        { id: "attendance", icon: BarChart3, labelKey: "dashboard.views.attendance" },
        { id: "trends", icon: LineChart, labelKey: "dashboard.views.trends" },
        { id: "custom", icon: Plus, labelKey: "dashboard.views.custom" },
    ];

export default function DashboardViewTabs({
    activeView,
    onViewChange,
}: DashboardViewTabsProps) {
    const { t } = useTranslation();

    return (
        <div
            className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner overflow-x-auto"
 
            role="tablist"
            aria-label={t("dashboard.views.label")}
        >
            {TABS.map((tab) => {
                const isActive = activeView === tab.id;
                const Icon = tab.icon;
                return (
                    <button
                        key={tab.id}
                        role="tab"
                        aria-selected={isActive}
                        onClick={() => onViewChange(tab.id)}
                        className={cn(
                            "flex items-center gap-2 px-5 sm:px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all whitespace-nowrap",
                            isActive
                                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                        )}
                    >
                        <Icon size={14} />
                        <span className="hidden sm:inline">{t(tab.labelKey)}</span>
                    </button>
                );
            })}
        </div>
    );
}
