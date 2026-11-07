import { Link, useLocation } from "@tanstack/react-router";
import {
  Bell,
  ChevronRight,
  Menu,
  Moon,
  Search,
  Sun,
  UserCircle,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LanguageSwitcher } from "@/components/ui/language-switcher";
import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useNotifications } from "@/hooks/use-notifications";
import { useProfile } from "@/hooks/use-profile";
import { ROUTES } from "@/lib/routes";

export interface TopbarProps {
  onOpenMobileMenu?: () => void;
  onOpenSearch?: () => void;
  workspaceName?: string;
}

export function Topbar({
  onOpenMobileMenu,
  onOpenSearch,
  workspaceName = "LeeAttend",
}: TopbarProps) {
  const { t } = useTranslation([
    "common",
    "dashboard",
    "employees",
    "leave",
    "payroll",
    "settings",
  ]);
  const location = useLocation();
  const { isDark, setMode } = useDarkModeContext();
  const { user } = useProfile();
  const { unreadCount } = useNotifications();

  const getPageTitle = (path: string): string => {
    const titles: Record<string, string> = {
      [ROUTES.HOME]: t("common.dashboard"),
      [ROUTES.DASHBOARD]: t("dashboard.catalog.title", "Tổng quan"),
      [ROUTES.PEOPLE]: t("common.people", "Nhân sự"),
      [ROUTES.EMPLOYEES]: t("common.employees", "Nhân viên"),
      [ROUTES.DEPARTMENTS]: t("common.departments", "Phòng ban"),
      [ROUTES.ORG_CHART]: t("common.org_chart", "Sơ đồ tổ chức"),
      [ROUTES.ATTENDANCE_LOGS]: t("common.logs", "Bảng chấm công"),
      [ROUTES.LEAVE_REQUESTS]: t("leave_requests.title", "Yêu cầu nghỉ phép"),
      [ROUTES.LEAVE_BALANCES]: t("common.leave_balances", "Quỹ phép"),
      [ROUTES.ATTENDANCE_ADJUSTMENTS]: t("common.attendance_confirm", "Xác nhận công"),
      [ROUTES.SHIFTS]: t("common.shifts", "Ca làm việc"),
      [ROUTES.SHIFTS_RULES]: t("common.sections.shifts_rules", "Quy tắc xếp ca"),
      [ROUTES.SHIFT_SWAPS]: t("shift_swaps.title", "Đổi ca"),
      [ROUTES.PAYROLL]: t("payroll.title", "Tính lương"),
      [ROUTES.REPORTS]: t("common.reports", "Báo cáo"),
      [ROUTES.APPROVALS]: t("ot_approvals.title", "Phê duyệt"),
      [ROUTES.APPROVALS_OT]: t("ot_approvals.title", "Duyệt tăng ca"),
      [ROUTES.AUDIT_LOGS]: t("audit_logs.title", "Audit Logs"),
      [ROUTES.SETTINGS.ROOT]: t("settings.title", "Cài đặt"),
      [ROUTES.SETTINGS.GENERAL]: t("settings.categories.general", "Cài đặt chung"),
      [ROUTES.SETTINGS.ORGANIZATION]: t("settings.categories.organization", "Tổ chức"),
      [ROUTES.SETTINGS.TIME]: t("settings.categories.time", "Thời gian & Chấm công"),
      [ROUTES.SETTINGS.PAYROLL]: t("settings.categories.payroll", "Lương & Thưởng"),
      [ROUTES.SETTINGS.WORKFLOWS]: t("settings.categories.workflows", "Quy trình duyệt"),
      [ROUTES.NOTIFICATIONS]: t("common.notifications", "Thông báo"),
      [ROUTES.PROFILE]: t("common.profile", "Hồ sơ"),
      [ROUTES.ESS_PROFILE]: t("common.my_profile", "Hồ sơ của tôi"),
    };

    if (titles[path]) {
      return titles[path];
    }

    // Match prefix route if exact match not found
    for (const [routePath, title] of Object.entries(titles)) {
      if (routePath !== "/" && path.startsWith(`${routePath}/`)) {
        return title;
      }
    }

    const segments = path.split("/").filter(Boolean);
    const last = segments[segments.length - 1];
    return last ? last.replace(/-/g, " ") : t("common.dashboard");
  };

  const pageTitle = getPageTitle(location.pathname);

  return (
    <header
      aria-label="Top Navigation"
      className="flex h-14 shrink-0 items-center justify-between border-b border-border bg-card/80 px-3 md:px-5 backdrop-blur-md shadow-2xs z-20"
      data-testid="topbar-container"
    >
      {/* ── Left: Mobile Hamburger & Breadcrumb ── */}
      <div className="flex items-center gap-2 md:gap-3 min-w-0">
        {onOpenMobileMenu && (
          <Button
            aria-label="Open navigation menu"
            className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground md:hidden"
            data-testid="topbar-mobile-menu-btn"
            onClick={onOpenMobileMenu}
            size="sm"
            type="button"
            variant="ghost"
          >
            <Menu className="h-4 w-4" />
          </Button>
        )}

        <nav
          aria-label="Breadcrumbs"
          className="flex items-center gap-1.5 text-xs md:text-sm font-medium truncate"
          data-testid="topbar-breadcrumb"
        >
          <Link
            className="text-muted-foreground hover:text-foreground transition-colors truncate max-w-[120px] md:max-w-none"
            to={ROUTES.HOME}
          >
            {workspaceName}
          </Link>
          <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
          <span className="text-foreground font-semibold truncate capitalize">
            {pageTitle}
          </span>
        </nav>
      </div>

      {/* ── Center: Search Bar Trigger (Cmd+K) ── */}
      <div className="flex-1 max-w-md mx-3 hidden sm:flex">
        <Button
          aria-label="Open command palette (⌘K)"
          className="flex h-9 w-full items-center justify-between rounded-lg border border-border bg-muted/50 px-3 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors shadow-2xs outline-none focus-visible:ring-2 focus-visible:ring-primary"
          data-testid="topbar-search-trigger"
          onClick={onOpenSearch}
          type="button"
          variant="ghost"
        >
          <span className="flex items-center gap-2">
            <Search className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="truncate">
              {t("common.search_placeholder", "Tìm kiếm tính năng, nhân sự...")}
            </span>
          </span>
          <kbd className="rounded border border-border bg-background px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground">
            ⌘K
          </kbd>
        </Button>
      </div>

      {/* ── Right: Quick Actions ── */}
      <div className="flex items-center gap-1.5 shrink-0">
        {/* Mobile search trigger icon */}
        <Button
          aria-label="Open search"
          className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground transition-colors focus-visible:ring-2 focus-visible:ring-primary sm:hidden"
          data-testid="topbar-search-trigger-mobile"
          onClick={onOpenSearch}
          size="sm"
          type="button"
          variant="ghost"
        >
          <Search className="h-4 w-4" />
        </Button>

        {/* Notifications */}
        <Link
          aria-label={t("common.notifications", "Notifications")}
          className="relative flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary"
          data-testid="topbar-notifications"
          title={t("common.notifications", "Thông báo")}
          to={ROUTES.NOTIFICATIONS}
        >
          <Bell className="h-4 w-4" />
          {unreadCount > 0 && (
            <span className="absolute top-1 right-1 flex h-3.5 min-w-[14px] items-center justify-center rounded-full bg-destructive px-1 text-[9px] font-bold text-destructive-foreground leading-none">
              {unreadCount > 99 ? "99+" : unreadCount}
            </span>
          )}
        </Link>

        {/* Theme Toggle */}
        <Button
          aria-label="Toggle theme"
          className="h-8 w-8 p-0 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors focus-visible:ring-2 focus-visible:ring-primary"
          data-testid="topbar-theme-toggle"
          onClick={() => {
            setMode(isDark ? "light" : "dark");
          }}
          size="sm"
          title={isDark ? "Light mode" : "Dark mode"}
          type="button"
          variant="ghost"
        >
          {isDark ? (
            <Sun className="h-4 w-4 text-amber-400" />
          ) : (
            <Moon className="h-4 w-4" />
          )}
        </Button>

        {/* Language Switcher */}
        <div data-testid="topbar-lang-toggle">
          <LanguageSwitcher
            className="h-8 min-h-[32px] px-2 rounded-lg border-0 bg-transparent hover:bg-muted text-muted-foreground hover:text-foreground shadow-none text-xs"
            compact
          />
        </div>

        {/* User Profile Avatar Link */}
        <Link
          aria-label="Profile"
          className="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary overflow-hidden ml-1"
          data-testid="topbar-profile-link"
          title={user?.full_name ?? user?.email ?? "Profile"}
          to={ROUTES.ESS_PROFILE}
        >
          {user?.avatar_url ? (
            <img
              alt={user.full_name ?? user?.email ?? "Profile"}
              className="h-6 w-6 rounded-md object-cover border border-border"
              src={user.avatar_url}
            />
          ) : (
            <UserCircle className="h-5 w-5 text-primary" />
          )}
        </Link>
      </div>
    </header>
  );
}
export default Topbar;
