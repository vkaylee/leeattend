/* eslint-disable react/forbid-elements */
import { useLocation } from "@tanstack/react-router";
import { ChevronRight as BreadcrumbSeparator, Search } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { useHeaderActions } from "@/hooks/use-header-actions";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";
interface NavbarProps {
  currentWorkspaceName: string;
  onOpenLauncher?: () => void;
}

export default function Navbar({ currentWorkspaceName, onOpenLauncher }: NavbarProps) {
  const { t } = useTranslation(["common", "dashboard", "employees", "leave", "payroll", "settings"]);
  const location = useLocation();
  const { actions } = useHeaderActions();

  const getPageName = (path: string) => {
    const map: Partial<Record<string, string>> = {
      [ROUTES.HOME]: t("common.dashboard"),
      [ROUTES.DASHBOARD]: t("dashboard.catalog.title"),
      [ROUTES.PEOPLE]: t("common.people"),
      [ROUTES.EMPLOYEES]: t("common.employees"),
      [ROUTES.DEPARTMENTS]: t("common.departments"),
      [ROUTES.ATTENDANCE_LOGS]: t("common.logs"),
      [ROUTES.LEAVE_REQUESTS]: t("leave_requests.title"),
      [ROUTES.LEAVE_BALANCES]: t("common.leave_balances"),
      [ROUTES.ATTENDANCE_ADJUSTMENTS]: t("common.attendance_confirm"),
      [ROUTES.REPORTS]: t("common.reports"),
      [ROUTES.SHIFTS]: t("common.shifts"),
      [ROUTES.PAYROLL]: t("payroll.title"),
      [ROUTES.SETTINGS.ROOT]: t("settings.title"),
      [ROUTES.SETTINGS.GENERAL]: t("settings.categories.general"),
      [ROUTES.SETTINGS.ORGANIZATION]: t("settings.categories.organization", { defaultValue: "Organization" }),
      [ROUTES.SETTINGS.TIME]: t("settings.categories.time"),
      [ROUTES.SETTINGS.PAYROLL]: t("settings.categories.payroll"),
      [ROUTES.SETTINGS.WORKFLOWS]: t("settings.categories.workflows", { defaultValue: "Approval Workflows" }),
      [ROUTES.PROFILE]: t("common.profile"),
      [ROUTES.ESS_PROFILE]: t("common.my_profile"),
      [ROUTES.ESS_ATTENDANCE]: t("ess.attendance.title"),
      [ROUTES.ESS_CLOCK]: t("ess.clock.title"),
    };
    return map[path] ?? path.split("/").pop()?.replace(/-/g, " ") ?? "";
  };

  return (
    <header className="h-14 border-b border-slate-100 dark:border-slate-800 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xl flex items-center px-4 lg:px-6 gap-4 flex-shrink-0 z-30 sticky top-0 shadow-sm">
      {/* Breadcrumbs */}
      <nav className="flex items-center gap-3 flex-1 min-w-0">
        <span className="hidden sm:inline text-slate-400 dark:text-slate-500 font-bold uppercase text-[10px] tracking-[0.25em] opacity-60 truncate">
          {currentWorkspaceName}
        </span>
        <BreadcrumbSeparator className="w-3 h-3 text-slate-300 dark:text-slate-600 hidden sm:inline shrink-0" />
        <span className="text-slate-700 dark:text-slate-200 font-bold uppercase text-[10px] tracking-[0.25em] bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg shadow-inner truncate">
          {getPageName(location.pathname).toUpperCase()}
        </span>
      </nav>

      {/* Quick Feature Search Trigger (DSM 7 Launcher) */}
      {Boolean(onOpenLauncher) && (
        <button
          type="button"
          data-testid="navbar-dsm-launcher-btn"
          onClick={onOpenLauncher}
          aria-label={t("appLauncher.openSearch", "Mở tìm kiếm tính năng DSM 7")}
          className={cn(
            "flex items-center gap-2 px-3 py-1.5 rounded-xl text-xs",
            "bg-slate-100/80 dark:bg-slate-800/80 hover:bg-slate-200/80 dark:hover:bg-slate-700/80",
            "border border-slate-200/60 dark:border-slate-700/60 text-slate-500 dark:text-slate-400",
            "transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500",
          )}
        >
          <Search className="w-3.5 h-3.5 text-slate-400 shrink-0" />
          <span className="hidden md:inline font-normal text-slate-600 dark:text-slate-300">
            {t("appLauncher.quickSearchPlaceholder", "Tìm tính năng, ứng dụng...")}
          </span>
          <kbd className="hidden md:inline-flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] font-semibold text-slate-400 dark:text-slate-400 bg-white dark:bg-slate-900 rounded border border-slate-200 dark:border-slate-700 shadow-xs">
            ⌘K
          </kbd>
        </button>
      )}

      {/* Quick Actions slot (injected by child pages via context) */}
      {!!actions && (
        <div className="flex min-w-0 max-w-[60vw] md:max-w-none overflow-x-auto scrollbar-hide flex-shrink-0 min-h-11 items-center gap-2 py-1">
          {actions}
        </div>
      )}
    </header>
  );
}
