import { Link, useLocation } from "@tanstack/react-router";
import {
  Banknote,
  Bell,
  CalendarPlus,
  Clock,
  LayoutDashboard,
  LayoutGrid,
  LogOut,
  Moon,
  Settings,
  Sun,
  UserCircle,
  Users,
} from "lucide-react";
import React, { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LanguageSwitcher } from "@/components/ui/language-switcher";
import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useNotifications } from "@/hooks/use-notifications";
import { useProfile } from "@/hooks/use-profile";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

export interface FloatingDockProps {
  className?: string;
  onOpenLauncher?: () => void;
}

interface DockAppItem {
  defaultLabel: string;
  icon: React.ElementType;
  id: string;
  isActive: (pathname: string) => boolean;
  labelKey: string;
  testId: string;
  to: string;
}

const DOCK_APPS: DockAppItem[] = [
  {
    id: "dashboard",
    labelKey: "common.dashboard",
    defaultLabel: "Trang chủ",
    to: ROUTES.HOME,
    icon: LayoutDashboard,
    testId: "dock-app-dashboard",
    isActive: (pathname) => pathname === "/" || pathname === "/dashboard",
  },
  {
    id: "org",
    labelKey: "common.employees",
    defaultLabel: "Nhân sự",
    to: ROUTES.EMPLOYEES,
    icon: Users,
    testId: "dock-app-org",
    isActive: (pathname) =>
      pathname.startsWith("/employees") || pathname.startsWith("/departments"),
  },
  {
    id: "time",
    labelKey: "common.logs",
    defaultLabel: "Chấm công",
    to: ROUTES.ATTENDANCE_LOGS,
    icon: Clock,
    testId: "dock-app-time",
    isActive: (pathname) =>
      pathname.startsWith("/attendance") || pathname.startsWith("/shifts"),
  },
  {
    id: "leave",
    labelKey: "leave_requests.title",
    defaultLabel: "Nghỉ phép",
    to: ROUTES.LEAVE_REQUESTS,
    icon: CalendarPlus,
    testId: "dock-app-leave",
    isActive: (pathname) => pathname.startsWith("/leave"),
  },
  {
    id: "payroll",
    labelKey: "payroll.title",
    defaultLabel: "Bảng lương",
    to: ROUTES.PAYROLL,
    icon: Banknote,
    testId: "dock-app-payroll",
    isActive: (pathname) => pathname.startsWith("/payroll"),
  },
  {
    id: "settings",
    labelKey: "settings.title",
    defaultLabel: "Cài đặt",
    to: ROUTES.SETTINGS.ROOT,
    icon: Settings,
    testId: "dock-app-settings",
    isActive: (pathname) => pathname.startsWith(ROUTES.SETTINGS.ROOT) || pathname.startsWith("/settings"),
  },
];

export const FloatingDock = memo(function FloatingDock({
  onOpenLauncher,
  className,
}: FloatingDockProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const { user } = useProfile();
  const { mode, setMode } = useDarkModeContext();
  const { unreadCount } = useNotifications();

  const cycleTheme = useCallback(() => {
    const modes: ("light" | "system" | "dark")[] = ["light", "system", "dark"];
    setMode(modes[(modes.indexOf(mode) + 1) % 3]);
  }, [mode, setMode]);

  const handleLogout = useCallback(() => {
    localStorage.clear();
    window.location.href = ROUTES.LOGIN;
  }, []);

  const ThemeIcon = mode === "dark" ? Moon : Sun;

  return (
    <nav
      aria-label={t("common.settings.navigation", "Navigation")}
      className={cn(
        "hidden lg:flex fixed bottom-5 left-1/2 -translate-x-1/2 z-40 items-center gap-1.5 md:gap-2",
        "bg-white/90 dark:bg-slate-950/90 backdrop-blur-2xl",
        "border border-slate-200/80 dark:border-slate-800/80",
        "shadow-2xl shadow-slate-900/10 dark:shadow-black/60",
        "transition-all duration-300",
        className
      )}
      data-testid="floating-dock"
      role="navigation"
    >
      {/* 1. DSM 7 Fullscreen Launcher Trigger */}
      {!!onOpenLauncher && (
        <Button
          aria-label={t("appLauncher.dsmLauncher", "Main Menu (⌘K)")}
          className="relative p-0 h-10 w-10 rounded-2xl bg-gradient-to-br from-primary-500 to-primary-700 text-white hover:from-primary-600 hover:to-primary-800 hover:scale-110 active:scale-95 shadow-md shadow-primary-500/25 transition-all group shrink-0"
          data-testid="dock-dsm-launcher-btn"
          onClick={onOpenLauncher}
          size="sm"
          title={t("appLauncher.dsmLauncher", "Main Menu (⌘K)")}
          type="button"
          variant="ghost"
        >
          <LayoutGrid className="w-5 h-5 group-hover:rotate-12 transition-transform duration-200" />
        </Button>
      )}

      {/* Divider */}
      <div className="h-5 w-px bg-slate-200 dark:bg-slate-800 mx-1 shrink-0" />

      {/* 2. Core App Links */}
      <div className="flex items-center gap-1 shrink-0" role="list">
        {DOCK_APPS.map((item) => {
          const Icon = item.icon;
          const active = item.isActive(location.pathname);
          const label = t(item.labelKey, item.defaultLabel);

          return (
            <Link
              aria-label={label}
              className={cn(
                "relative flex items-center justify-center h-10 w-10 rounded-2xl transition-all duration-200 group shrink-0",
                active
                  ? "bg-primary-50 dark:bg-primary-950/60 text-primary-600 dark:text-primary-400 font-bold shadow-2xs"
                  : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-100/80 dark:hover:bg-slate-800/80 hover:scale-110 active:scale-95"
              )}
              data-testid={item.testId}
              key={item.id}
              title={label}
              to={item.to}
            >
              <Icon className="w-4 h-4 transition-transform duration-200 group-hover:scale-110" />
              {active ? <span className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-1.5 h-1.5 rounded-full bg-primary-500 shadow-xs shadow-primary-500/80" /> : null}
            </Link>
          );
        })}
      </div>

      {/* Divider */}
      <div className="h-5 w-px bg-slate-200 dark:bg-slate-800 mx-1 shrink-0" />

      {/* 3. System Tray & Utilities */}
      <div className="flex items-center gap-1 shrink-0">
        {/* Notifications */}
        <Link
          aria-label={t("common.notifications", "Notifications")}
          className="relative flex items-center justify-center h-10 w-10 rounded-2xl text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-100/80 dark:hover:bg-slate-800/80 hover:scale-110 active:scale-95 transition-all shrink-0"
          data-testid="dock-notifications"
          title={t("common.notifications", "Thông báo")}
          to={ROUTES.NOTIFICATIONS}
        >
          <Bell className="w-4 h-4" />
          {unreadCount > 0 && (
            <span className="absolute -top-1 -right-1 min-w-[16px] h-4 px-1 rounded-full bg-rose-500 text-white text-[9px] font-black flex items-center justify-center leading-none shadow-xs">
              {unreadCount > 99 ? "99+" : unreadCount}
            </span>
          )}
        </Link>

        {/* Theme Toggle */}
        <Button
          aria-label={t(`common.theme.${mode}`, mode)}
          className="h-10 w-10 p-0 rounded-2xl text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-100/80 dark:hover:bg-slate-800/80 hover:scale-110 active:scale-95 transition-all shrink-0"
          data-testid="dock-theme-toggle"
          onClick={cycleTheme}
          size="sm"
          title={t(`common.theme.${mode}`, mode)}
          type="button"
          variant="ghost"
        >
          <ThemeIcon className="w-4 h-4" />
        </Button>

        {/* Language Switcher */}
        <div className="shrink-0" data-testid="dock-lang-toggle">
          <LanguageSwitcher
            className="h-10 min-h-[40px] px-2.5 rounded-2xl border-0 bg-transparent hover:bg-slate-100/80 dark:hover:bg-slate-800/80 text-slate-600 dark:text-slate-400 shadow-none hover:scale-105 active:scale-95"
            compact
          />
        </div>

        {/* Profile Link (Matches accessible name "Profile" for E2E contract) */}
        <Link
          aria-label="Profile"
          className="relative flex items-center justify-center h-10 w-10 rounded-2xl hover:scale-110 active:scale-95 transition-all group shrink-0"
          data-testid="dock-user-avatar"
          title={t("common.profile", "Profile")}
          to={ROUTES.ESS_PROFILE}
        >
          <div className="w-7 h-7 rounded-xl bg-slate-900 dark:bg-slate-100 text-white dark:text-slate-900 flex items-center justify-center font-bold text-[11px] shadow-sm group-hover:bg-primary-600 dark:group-hover:bg-primary-500 group-hover:text-white transition-colors">
            {user?.full_name ? (
              user.full_name.charAt(0).toUpperCase()
            ) : (
              <UserCircle className="w-4 h-4" />
            )}
          </div>
        </Link>

        {/* Logout Button (Matches accessible name "Logout" for E2E contract) */}
        <Button
          aria-label="Logout"
          className="h-10 w-10 p-0 rounded-2xl text-slate-400 hover:text-rose-500 dark:hover:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 hover:scale-110 active:scale-95 transition-all shrink-0"
          data-testid="dock-logout"
          onClick={handleLogout}
          size="sm"
          title={t("common.logout", "Logout")}
          type="button"
          variant="ghost"
        >
          <LogOut className="w-4 h-4" />
        </Button>
      </div>
    </nav>
  );
});

FloatingDock.displayName = "FloatingDock";

export default FloatingDock;
