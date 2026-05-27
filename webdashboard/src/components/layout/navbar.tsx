import { useLocation, Link } from "@tanstack/react-router";
import { Menu, ChevronRight as BreadcrumbSeparator, ChevronLeft, ChevronRight, Sun, Moon, Monitor, LogOut, Settings as SettingsIcon, UserCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { ThemeMode } from "@/hooks/use-dark-mode";

import Button from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { LanguageSwitcher } from "@/components/ui/language-switcher";
import { Typography } from "@/components/ui/typography";
import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useProfile } from "@/hooks/use-profile";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import { NotificationsPopover } from "./notifications-popover";

interface NavbarProps {
  currentWorkspaceName: string;
  isCollapsed: boolean;
  onOpenSidebar: () => void;
  onToggleCollapsed: () => void;
}

export default function Navbar({ currentWorkspaceName, onOpenSidebar, isCollapsed, onToggleCollapsed }: NavbarProps) {
  const { t } = useTranslation(['common', 'employees', 'leave', 'payroll', 'settings']);
  const location = useLocation();

  const getPageName = (path: string) => {
    const map: Partial<Record<string, string>> = {
      [ROUTES.HOME]: t("common.dashboard"), [ROUTES.PEOPLE]: t("common.people"), [ROUTES.EMPLOYEES]: t("common.employees"), [ROUTES.DEPARTMENTS]: t("common.departments"),
      [ROUTES.ATTENDANCE_LOGS]: t("common.logs"), [ROUTES.LEAVE_REQUESTS]: t("leave_requests.title"),
      [ROUTES.LEAVE_BALANCES]: t("common.leave_balances"), [ROUTES.ATTENDANCE_ADJUSTMENTS]: t("common.attendance_confirm"), [ROUTES.REPORTS]: t("common.reports"),
      [ROUTES.SHIFTS]: t("common.shifts"), [ROUTES.PAYROLL]: t("payroll.title"),
      [ROUTES.AGENTS]: t("agents.title"), [ROUTES.DEVICES]: t("common.devices"), [ROUTES.SETTINGS.ROOT]: t("settings.title"), [ROUTES.SETTINGS.GENERAL]: t("settings.categories.general"),
      [ROUTES.SETTINGS.ORGANIZATION]: t("settings.categories.organization", { defaultValue: "Organization" }), [ROUTES.SETTINGS.TIME]: t("settings.categories.time"),
      [ROUTES.SETTINGS.PAYROLL]: t("settings.categories.payroll"), [ROUTES.SETTINGS.CONNECTIVITY]: t("settings.categories.connectivity"),
      [ROUTES.PROFILE]: t("common.profile"), [ROUTES.ESS_PROFILE]: t("common.my_profile"), [ROUTES.ESS_ATTENDANCE]: t("ess.attendance.title"),
      [ROUTES.ESS_CLOCK]: t("ess.clock.title"),
    };
    return map[path] ?? path.split('/').pop()?.replace(/-/g, ' ') ?? "";
  };

  return (
    <header className="h-20 border-b border-slate-100 dark:border-slate-800 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xl flex items-center justify-between px-6 lg:px-10 gap-6 flex-shrink-0 z-30 sticky top-0 shadow-2xl shadow-slate-900/5 dark:shadow-black/20">
      <div className="flex items-center gap-6">
        <Button className="lg:hidden p-3 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-xl active:scale-90 transition-all border border-slate-100 dark:border-slate-700 shadow-sm" onClick={onOpenSidebar} variant="ghost"><Menu size={20} /></Button>

        {/* Desktop Collapse Toggle */}
        <Button className="hidden lg:flex p-3 bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-primary-50 dark:hover:bg-primary-950 hover:text-primary-600 dark:hover:text-primary-400 rounded-xl active:scale-90 transition-all border border-slate-100 dark:border-slate-700 shadow-sm hover:shadow-md hover:border-primary-100 dark:hover:border-primary-800"
          onClick={onToggleCollapsed}
          title={isCollapsed ? t("common.expand_sidebar") : t("common.collapse_sidebar")}
          variant="ghost"
        >
          {isCollapsed ? <ChevronRight size={20} /> : <ChevronLeft size={20} />}
        </Button>

        {/* Premium Breadcrumbs */}
        <nav className="flex items-center gap-3">
          <span className="hidden sm:inline text-slate-500 dark:text-slate-400 font-bold uppercase text-[10px] tracking-[0.25em] opacity-60">{currentWorkspaceName}</span>
          <BreadcrumbSeparator className="w-3 h-3 text-slate-300 dark:text-slate-600" />
          <span className="text-slate-700 dark:text-slate-200 font-bold uppercase text-[10px] tracking-[0.25em] bg-slate-100 dark:bg-slate-800 px-3 py-1 rounded-lg shadow-inner">
            {getPageName(location.pathname).toUpperCase()}
          </span>
        </nav>
      </div>

      <div className="flex items-center gap-6 ml-auto">
        {/* Theme Toggle */}
        <ThemeToggle />

        {/* Language Switcher */}
        <LanguageSwitcher />

        <NotificationsPopover />

        <div className="h-10 w-px bg-slate-100 dark:bg-slate-800 mx-1 hidden sm:block opacity-50"></div>

        <UserProfileMenu />
      </div>
    </header>
  );
}

/** Segmented theme toggle: Light / System / Dark */
function ThemeToggle() {
  const { mode, setMode } = useDarkModeContext();
  const { t } = useTranslation(['common', 'employees', 'leave', 'payroll', 'settings']);

  const options: { value: ThemeMode; icon: React.ReactNode; label: string }[] = [
    { value: 'light', icon: <Sun size={14} />, label: t('common.theme.light', 'Light') },
    { value: 'system', icon: <Monitor size={14} />, label: t('common.theme.system', 'System') },
    { value: 'dark', icon: <Moon size={14} />, label: t('common.theme.dark', 'Dark') },
  ];

  return (
    <div className="flex bg-slate-100 dark:bg-slate-800 p-1 rounded-xl border border-slate-200 dark:border-slate-700 shadow-inner" data-testid="theme-toggle">
      {options.map((opt) => (
        <Button className={cn(
            "p-2 rounded-lg transition-all flex items-center justify-center gap-1.5 min-w-[44px] min-h-[44px]",
            mode === opt.value
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-700"
              : "text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300"
          )}
          data-testid={`theme-toggle-${opt.value}`}
          key={opt.value}
          onClick={() => { setMode(opt.value); }}
          title={opt.label}
          variant="ghost"
        >
          {opt.icon}
        </Button>
      ))}
    </div>
  );
}

function UserProfileMenu() {
  const { t } = useTranslation(['common', 'employees', 'leave', 'payroll', 'settings']);
  const { roles, managedDepartmentIds } = useMyPermissions();
  const { user } = useProfile();

  const handleLogout = () => { localStorage.clear(); window.location.href = ROUTES.LOGIN; };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <div className="flex items-center gap-4 pl-2 group cursor-pointer active:scale-95 transition-all outline-none" data-testid="navbar-user-menu-trigger">
          <div className="hidden sm:block text-right">
            <div className="flex items-center justify-end gap-2">
              <Typography className="text-xs font-bold text-slate-700 dark:text-slate-200 leading-none group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors uppercase tracking-tight" variant="p">
                {user?.full_name ?? (roles[0] ?? t("common.admin"))}
              </Typography>
              {roles.includes('TeamLead') && managedDepartmentIds !== null && (
                <span className="inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400 border border-amber-200 dark:border-amber-800 uppercase tracking-wider">
                  Department-scoped
                </span>
              )}
            </div>
            <Typography className="text-[9px] font-bold text-slate-500 dark:text-slate-400 mt-1.5 uppercase tracking-[0.2em] opacity-60 leading-none" variant="p">{t("common.workspace_owner")}</Typography>
          </div>
          <div className="w-12 h-12 rounded-[1.25rem] bg-slate-900 text-white flex items-center justify-center font-bold text-sm shadow-2xl shadow-slate-400 dark:shadow-black border border-slate-800 group-hover:bg-primary-600 group-hover:border-primary-500 transition-all duration-500 group-hover:scale-110 group-hover:rotate-3">
            {user?.full_name ? user.full_name.charAt(0).toUpperCase() : (roles[0]?.substring(0, 2).toUpperCase() ?? "AD")}
          </div>
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56 mt-2" data-testid="user-profile-content">
        <DropdownMenuLabel>{t("common.my_account", { defaultValue: "My Account" })}</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild data-testid="user-menu-profile-btn">
          <Link className="cursor-pointer flex items-center w-full" to={ROUTES.ESS_PROFILE}>
            <UserCircle className="mr-2 h-4 w-4" />
            <span>{t("common.profile")}</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem asChild data-testid="user-menu-settings-btn">
          <Link className="cursor-pointer flex items-center w-full" to={ROUTES.SETTINGS.ROOT}>
            <SettingsIcon className="mr-2 h-4 w-4" />
            <span>{t("settings.title")}</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem className="text-rose-600 dark:text-rose-400 focus:bg-rose-50 dark:focus:bg-rose-950/30 focus:text-rose-600 dark:focus:text-rose-400 cursor-pointer" data-testid="user-menu-logout-btn" onClick={handleLogout}>
          <LogOut className="mr-2 h-4 w-4" />
          <span>{t("common.logout")}</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
