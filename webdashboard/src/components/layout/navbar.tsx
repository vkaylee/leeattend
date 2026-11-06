import { Link, useLocation, useNavigate } from "@tanstack/react-router";
import {
  ChevronDown,
  ChevronRight as BreadcrumbSeparator,
  Search,
} from "lucide-react";
import React, { memo } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useHeaderActions } from "@/hooks/use-header-actions";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

interface NavbarProps {
  currentWorkspaceName: string;
  onOpenLauncher?: () => void;
}

interface SiblingApp {
  labelKey: string;
  defaultLabel: string;
  to: string;
}

interface NavCategory {
  key: string;
  labelKey: string;
  defaultLabel: string;
  siblings: SiblingApp[];
}

const CATEGORIES: NavCategory[] = [
  {
    key: "time",
    labelKey: "common.sections.time_leave",
    defaultLabel: "Chấm công & Ca",
    siblings: [
      { labelKey: "common.logs", defaultLabel: "Bảng chấm công", to: ROUTES.ATTENDANCE_LOGS },
      { labelKey: "leave_requests.title", defaultLabel: "Yêu cầu nghỉ phép", to: ROUTES.LEAVE_REQUESTS },
      { labelKey: "common.leave_balances", defaultLabel: "Quỹ phép", to: ROUTES.LEAVE_BALANCES },
      { labelKey: "common.attendance_confirm", defaultLabel: "Xác nhận công", to: ROUTES.ATTENDANCE_ADJUSTMENTS },
      { labelKey: "common.shifts", defaultLabel: "Ca làm việc", to: ROUTES.SHIFTS },
    ],
  },
  {
    key: "org",
    labelKey: "common.people",
    defaultLabel: "Nhân sự & Tổ chức",
    siblings: [
      { labelKey: "common.employees", defaultLabel: "Danh sách nhân viên", to: ROUTES.EMPLOYEES },
      { labelKey: "common.departments", defaultLabel: "Phòng ban", to: ROUTES.DEPARTMENTS },
      { labelKey: "common.people", defaultLabel: "Sơ đồ tổ chức", to: ROUTES.PEOPLE },
    ],
  },
  {
    key: "settings",
    labelKey: "settings.title",
    defaultLabel: "Cài đặt",
    siblings: [
      { labelKey: "settings.categories.general", defaultLabel: "Chung", to: ROUTES.SETTINGS.GENERAL },
      { labelKey: "settings.categories.organization", defaultLabel: "Tổ chức", to: ROUTES.SETTINGS.ORGANIZATION },
      { labelKey: "settings.categories.time", defaultLabel: "Chấm công & Nghỉ", to: ROUTES.SETTINGS.TIME },
      { labelKey: "settings.categories.payroll", defaultLabel: "Lương & Thưởng", to: ROUTES.SETTINGS.PAYROLL },
      { labelKey: "settings.categories.workflows", defaultLabel: "Quy trình duyệt", to: ROUTES.SETTINGS.WORKFLOWS },
    ],
  },
  {
    key: "payroll",
    labelKey: "payroll.title",
    defaultLabel: "Bảng lương",
    siblings: [
      { labelKey: "payroll.title", defaultLabel: "Tính lương", to: ROUTES.PAYROLL },
      { labelKey: "common.reports", defaultLabel: "Báo cáo chi phí", to: ROUTES.REPORTS },
    ],
  },
];

export default memo(function Navbar({
  currentWorkspaceName,
  onOpenLauncher,
}: NavbarProps) {
  const { t } = useTranslation([
    "common",
    "dashboard",
    "employees",
    "leave",
    "payroll",
    "settings",
  ]);
  const location = useLocation();
  const navigate = useNavigate();
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
      [ROUTES.SETTINGS.ORGANIZATION]: t("settings.categories.organization", {
        defaultValue: "Organization",
      }),
      [ROUTES.SETTINGS.TIME]: t("settings.categories.time"),
      [ROUTES.SETTINGS.PAYROLL]: t("settings.categories.payroll"),
      [ROUTES.SETTINGS.WORKFLOWS]: t("settings.categories.workflows", {
        defaultValue: "Approval Workflows",
      }),
      [ROUTES.PROFILE]: t("common.profile"),
      [ROUTES.ESS_PROFILE]: t("common.my_profile"),
      [ROUTES.ESS_ATTENDANCE]: t("ess.attendance.title"),
      [ROUTES.ESS_CLOCK]: t("ess.clock.title"),
    };
    return map[path] ?? path.split("/").pop()?.replace(/-/g, " ") ?? "";
  };

  // Find matching category for in-app sibling switching
  const currentCategory = CATEGORIES.find((cat) =>
    cat.siblings.some((sib) => location.pathname.startsWith(sib.to))
  );

  return (
    <header className="h-14 border-b border-slate-100 dark:border-slate-800 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xl flex items-center px-4 lg:px-6 gap-4 flex-shrink-0 z-30 sticky top-0 shadow-xs">
      {/* Breadcrumbs & App Switcher */}
      <nav
        aria-label="Breadcrumb"
        className="flex items-center gap-2 md:gap-3 flex-1 min-w-0"
      >
        <Link
          to={ROUTES.HOME}
          className="flex items-center gap-2 hover:opacity-80 transition-opacity truncate"
        >
          <span className="hidden sm:inline text-slate-400 dark:text-slate-500 font-bold uppercase text-[10px] tracking-[0.25em] opacity-80 truncate">
            {currentWorkspaceName}
          </span>
        </Link>

        <BreadcrumbSeparator className="w-3 h-3 text-slate-300 dark:text-slate-600 hidden sm:inline shrink-0" />

        {/* Sibling Category Dropdown if inside a feature module */}
        {currentCategory && (
          <>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="sm"
                  data-testid="navbar-category-switcher"
                  className="hidden md:flex items-center gap-1 h-7 px-2 text-xs font-semibold text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg"
                >
                  <span>
                    {t(currentCategory.labelKey, currentCategory.defaultLabel)}
                  </span>
                  <ChevronDown className="w-3 h-3 text-slate-400" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-56 p-1 rounded-2xl">
                <DropdownMenuLabel className="text-[11px] font-bold text-slate-400 px-2 py-1 uppercase tracking-wider">
                  {t(currentCategory.labelKey, currentCategory.defaultLabel)}
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                {currentCategory.siblings.map((sibling) => (
                  <DropdownMenuItem
                    key={sibling.to}
                    onClick={() => {
                      void navigate({ to: sibling.to });
                    }}
                    className={cn(
                      "text-xs px-2.5 py-1.5 rounded-xl cursor-pointer",
                      location.pathname.startsWith(sibling.to)
                        ? "bg-primary-50 dark:bg-primary-950/60 text-primary-600 dark:text-primary-400 font-bold"
                        : "text-slate-700 dark:text-slate-200"
                    )}
                  >
                    {t(sibling.labelKey, sibling.defaultLabel)}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
            <BreadcrumbSeparator className="w-3 h-3 text-slate-300 dark:text-slate-600 hidden md:inline shrink-0" />
          </>
        )}

        {/* Current Active Page Title Badge */}
        <span
          data-testid="navbar-active-page"
          className="text-slate-700 dark:text-slate-200 font-bold uppercase text-[10px] tracking-[0.25em] bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg shadow-inner truncate"
        >
          {getPageName(location.pathname).toUpperCase()}
        </span>
      </nav>

      {/* Quick Feature Search Trigger (DSM 7 Launcher) */}
      {Boolean(onOpenLauncher) && (
        <Button
          type="button"
          variant="outline"
          size="sm"
          data-testid="navbar-dsm-launcher-btn"
          onClick={onOpenLauncher}
          aria-label={t(
            "appLauncher.openSearch",
            "Mở tìm kiếm tính năng DSM 7"
          )}
          className={cn(
            "flex items-center gap-2 px-3 h-9 rounded-xl text-xs",
            "bg-slate-100/80 dark:bg-slate-800/80 hover:bg-slate-200/80 dark:hover:bg-slate-700/80",
            "border border-slate-200/60 dark:border-slate-700/60 text-slate-600 dark:text-slate-300",
            "transition-all duration-150 shadow-2xs group"
          )}
        >
          <Search className="w-3.5 h-3.5 text-slate-400 group-hover:text-primary-500 transition-colors shrink-0" />
          <span className="hidden md:inline font-normal text-slate-600 dark:text-slate-300">
            {t(
              "appLauncher.quickSearchPlaceholder",
              "Tìm tính năng, ứng dụng..."
            )}
          </span>
          <kbd className="hidden md:inline-flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] font-semibold text-slate-400 dark:text-slate-400 bg-white dark:bg-slate-900 rounded border border-slate-200 dark:border-slate-700 shadow-2xs">
            ⌘K
          </kbd>
        </Button>
      )}

      {/* Quick Actions slot (injected by child pages via context) */}
      {!!actions && (
        <div className="flex min-w-0 max-w-[60vw] md:max-w-none overflow-x-auto scrollbar-hide flex-shrink-0 min-h-11 items-center gap-2 py-1">
          {actions}
        </div>
      )}
    </header>
  );
});
