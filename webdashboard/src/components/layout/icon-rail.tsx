/* eslint-disable react/forbid-elements -- internal rail icon buttons are
 * interactive primitives (with state-driven custom tooltip portals and
 * absolute-positioned active indicators) that the design-system <Button>
 * does not yet support. */
import { Link } from "@tanstack/react-router";
import { Bell, LayoutGrid, LogOut, Moon, Pin, PinOff, Sun, UserCircle } from "lucide-react";
import React, { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { NavGroup } from "@/hooks/use-nav-menu-items";

import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { isHoverNavigationAvailable } from "@/hooks/use-nav-sidebar";
import { useLanguage } from "@/hooks/use-language";
import { useFilteredMenuItems } from "@/hooks/use-nav-menu-items";
import { useNotifications } from "@/hooks/use-notifications";
import { useProfile } from "@/hooks/use-profile";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import { TooltipPortal, useTooltipTarget } from "./icon-rail-hooks";
import { IconLanguageButton } from "./icon-rail-language-button";

interface IconRailProps {
  activeGroup: NavGroup | null;
  isPinned: boolean;
  onClickGroup: (g: NavGroup) => void;
  onHoverGroup: (g: NavGroup | null) => void;
  onOpenLauncher?: () => void;
  onTogglePin: () => void;
  workspaceLogoUrl?: string;
  workspaceName?: string;
  workspaceTagline?: string;
}

const SHARED_ICON_CLASSES = "flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative border-0";
const ACTIVE_CLASSES = "text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950";
const IDLE_CLASSES = "text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-900";

// eslint-disable-next-line complexity
const IconRail = memo(function IconRail({
  activeGroup,
  isPinned,
  onClickGroup,
  onHoverGroup,
  onOpenLauncher,
  onTogglePin,
  workspaceName,
  workspaceLogoUrl,
  workspaceTagline,
}: IconRailProps) {
  const { t } = useTranslation();
  const { dashboard, settings, groups } = useFilteredMenuItems();
  const { user } = useProfile();
  const { mode, setMode } = useDarkModeContext();
  const { currentLanguage, changeLanguage, languages } = useLanguage();
  const { unreadCount } = useNotifications();

  const cycleTheme = useCallback(() => {
    const modes: ("light" | "system" | "dark")[] = ["light", "system", "dark"];
    setMode(modes[(modes.indexOf(mode) + 1) % 3]);
  }, [mode, setMode]);

  const ThemeIcon = mode === "dark" ? Moon : Sun;

  const handleDashboardClick = useCallback(() => { onClickGroup("dashboard"); }, [onClickGroup]);
  const handleDashboardHover = useCallback((h: boolean) => { onHoverGroup(h ? "dashboard" : null); }, [onHoverGroup]);
  const handleSettingsClick = useCallback(() => { onClickGroup("settings"); }, [onClickGroup]);
  const handleSettingsHover = useCallback((h: boolean) => { onHoverGroup(h ? "settings" : null); }, [onHoverGroup]);
  const handleLogout = useCallback(() => { localStorage.clear(); window.location.href = ROUTES.LOGIN; }, []);

  return (
    <nav
      aria-label={t("common.settings.navigation", "Navigation")}
      className="hidden lg:flex fixed inset-y-0 left-0 z-50 w-[var(--nav-rail-width)] flex-col overflow-x-clip bg-white dark:bg-slate-950 border-r border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-200/50 dark:shadow-black/30"
      role="navigation"
    >
      <button
        aria-label={workspaceName ?? "LeeAttend"}
        className="flex items-center justify-center h-14 w-full shrink-0 hover:bg-slate-50 dark:hover:bg-slate-900 transition-colors group"
        data-testid="nav-rail-logo"
        onClick={() => { window.location.href = ROUTES.HOME; }}
        type="button"
      >
        {workspaceLogoUrl ? (
          <div
            aria-label={`${workspaceName ?? "LeeAttend"} logo`}
            className="w-8 h-8 bg-contain bg-no-repeat bg-center drop-shadow-sm"
            data-testid="sidebar-workspace-logo"
            role="img"
            style={{ backgroundImage: `url(${workspaceLogoUrl})` }}
          />
        ) : (
          <div className="h-8 w-8 rounded-xl bg-gradient-to-tr from-primary-600 to-primary-400 shadow-lg shadow-primary-200 dark:shadow-primary-900/20 flex items-center justify-center text-white font-black text-sm tracking-tighter italic">
            {workspaceName?.charAt(0).toUpperCase() ?? "L"}
          </div>
        )}
        {workspaceTagline ? (
          <span className="sr-only" data-testid="sidebar-workspace-tagline">
            {workspaceTagline}
          </span>
        ) : null}
      </button>

      {onOpenLauncher ? (
        <div className="px-1.5 py-1 shrink-0">
          <IconTooltipButton
            data-testid="nav-rail-dsm-launcher-btn"
            icon={LayoutGrid}
            label={t("appLauncher.dsmLauncher", "Main Menu (⌘K)")}
            onClick={onOpenLauncher}
          />
        </div>
      ) : null}

      <div className="h-px bg-slate-100 dark:bg-slate-800 mx-3 shrink-0" />

      <div className="flex-1 overflow-y-auto overscroll-contain custom-scrollbar py-3 space-y-1 px-1.5" role="list">
        <MenuLink group="dashboard" icon={dashboard.icon} isActive={activeGroup === "dashboard"} label={dashboard.label}
          onClick={handleDashboardClick} onHover={handleDashboardHover}
          testId="nav-rail-group-dashboard" to={ROUTES.HOME} />
        <MenuLink group="settings" icon={settings.icon} isActive={activeGroup === "settings"} label={settings.label}
          onClick={handleSettingsClick} onHover={handleSettingsHover}
          testId="nav-rail-group-settings" to={settings.to} />
        {groups.map((g) => (
          <MenuLink group={g.section as NavGroup} icon={g.sectionIcon} isActive={activeGroup === g.section} key={g.section} label={g.label}
            onClick={() => { onClickGroup(g.section as NavGroup); }} onHover={(h) => { onHoverGroup(h ? (g.section as NavGroup) : null); }}
            testId={`nav-rail-group-${g.section}`} />
        ))}
      </div>

      <div className="shrink-0 border-t border-slate-100 dark:border-slate-800 py-2 space-y-1 px-1.5">
        <IconTooltipButton active={isPinned} data-testid="nav-rail-pin-toggle" icon={isPinned ? PinOff : Pin}
          label={isPinned ? t("common.collapse", "Unpin") : t("common.expand", "Pin")} onClick={onTogglePin} />
        <IconTooltipButton data-testid="nav-rail-theme-toggle" icon={ThemeIcon} label={t(`common.theme.${mode}`, mode)} onClick={cycleTheme} />
        <IconLanguageButton changeLanguage={changeLanguage} currentLanguage={currentLanguage} languages={languages} />
        <IconTooltipLink aria-label={t("common.notifications")} data-testid="nav-rail-notifications" to={ROUTES.NOTIFICATIONS}>
          <div className="relative border-0">
            <Bell className="h-4 w-4" />
            {unreadCount > 0 && (
              <span className="absolute -top-1.5 -right-1.5 min-w-[16px] h-4 px-1 rounded-full bg-rose-500 text-white text-[9px] font-black flex items-center justify-center leading-none">
                {unreadCount > 99 ? "99+" : unreadCount}
              </span>
            )}
          </div>
        </IconTooltipLink>
        <IconTooltipLink aria-label={t("common.profile")} data-testid="nav-rail-user-avatar" to={ROUTES.ESS_PROFILE}>
          <div className="w-7 h-7 rounded-lg border-0 bg-slate-800 dark:bg-slate-200 text-white dark:text-slate-800 flex items-center justify-center font-bold text-[10px] shadow-sm group-hover:bg-primary-600 dark:group-hover:bg-primary-500 group-hover:text-white transition-all duration-300">
            {user?.full_name ? user.full_name.charAt(0).toUpperCase() : <UserCircle className="h-4 w-4" />}
          </div>
        </IconTooltipLink>
        <IconLogoutButton onLogout={handleLogout} />
      </div>
    </nav>
  );
});

IconRail.displayName = "IconRail";

export default IconRail;

// ── Menu link (router Link or plain button depending on group) ──────────

interface MenuLinkProps {
  group: NavGroup;
  icon?: React.ElementType;
  isActive: boolean;
  label: string;
  onClick: () => void;
  onHover: (hovered: boolean) => void;
  testId: string;
  to?: string;
}

const MenuLink = memo(function MenuLink({ group, icon: Icon, isActive, label, onClick, onHover, testId, to }: MenuLinkProps) {
  const { el, hovered, setHovered, refCb } = useTooltipTarget<HTMLAnchorElement | HTMLButtonElement>();
  const cls = cn(SHARED_ICON_CLASSES, isActive ? ACTIVE_CLASSES : IDLE_CLASSES);
  const indicator = isActive ? <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-6 rounded-r-full bg-primary-500 dark:bg-primary-400" /> : null;
  const inner = <>{indicator}{Icon ? <Icon className="h-4 w-4" /> : null}</>;

  const handleBlur = useCallback(() => { setHovered(false); }, [setHovered]);
  const handleFocus = useCallback(() => { setHovered(true); }, [setHovered]);
  const handleMouseEnter = useCallback(() => {
    if (!isHoverNavigationAvailable()) return;
    setHovered(true);
    onHover(true);
  }, [setHovered, onHover]);
  const handleMouseLeave = useCallback(() => {
    if (!isHoverNavigationAvailable()) return;
    setHovered(false);
    onHover(false);
  }, [setHovered, onHover]);

  if (to && (group === "dashboard" || group === "settings")) {
    return (
      <Link aria-label={label} className={cls} data-testid={testId} onBlur={handleBlur} onFocus={handleFocus}
        onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave} ref={refCb} to={to as "/"}>
        {inner}
        <TooltipPortal el={el} hovered={hovered} label={label} />
      </Link>
    );
  }

  return (
    <button aria-label={label} className={cls} data-testid={testId} onBlur={handleBlur} onClick={onClick} onFocus={handleFocus}
      onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave} ref={refCb} type="button">
      {inner}
      <TooltipPortal el={el} hovered={hovered} label={label} />
    </button>
  );
});

// ── Icon tooltip button pattern ─────────────────────────────────────────

const IconTooltipButton = memo(function IconTooltipButton({
  icon: Icon, active, onClick, label, "data-testid": testId,
}: {
  active?: boolean;
  "data-testid"?: string;
  icon: React.ElementType;
  label: string;
  onClick: () => void;
}) {
  const { el, hovered, setHovered, refCb } = useTooltipTarget<HTMLButtonElement>();
  const handleBlur = useCallback(() => { setHovered(false); }, [setHovered]);
  const handleFocus = useCallback(() => { setHovered(true); }, [setHovered]);
  const handleMouseEnter = useCallback(() => {
    if (isHoverNavigationAvailable()) setHovered(true);
  }, [setHovered]);
  const handleMouseLeave = useCallback(() => {
    if (isHoverNavigationAvailable()) setHovered(false);
  }, [setHovered]);
  return (
    <>
      <button
        aria-label={label}
        className={cn(SHARED_ICON_CLASSES, active ? ACTIVE_CLASSES : IDLE_CLASSES)}
        data-testid={testId}
        onBlur={handleBlur}
        onClick={onClick}
        onFocus={handleFocus}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        ref={refCb}
        type="button"
      >
        <Icon className="h-4 w-4" />
      </button>
      <TooltipPortal el={el} hovered={hovered} label={label} />
    </>
  );
});

const IconTooltipLink = memo(function IconTooltipLink({
  to, "aria-label": ariaLabel, children, "data-testid": testId,
}: {
  "aria-label": string;
  children: React.ReactNode;
  "data-testid"?: string;
  to: string;
}) {
  const { el, hovered, setHovered, refCb } = useTooltipTarget<HTMLAnchorElement>();
  const handleBlur = useCallback(() => { setHovered(false); }, [setHovered]);
  const handleFocus = useCallback(() => { setHovered(true); }, [setHovered]);
  const handleMouseEnter = useCallback(() => {
    if (isHoverNavigationAvailable()) setHovered(true);
  }, [setHovered]);
  const handleMouseLeave = useCallback(() => {
    if (isHoverNavigationAvailable()) setHovered(false);
  }, [setHovered]);
  return (
    <Link
      aria-label={ariaLabel}
      className={cn(SHARED_ICON_CLASSES, IDLE_CLASSES)}
      data-testid={testId}
      onBlur={handleBlur}
      onFocus={handleFocus}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      ref={refCb}
      to={to}
    >
      {children}
      <TooltipPortal el={el} hovered={hovered} label={ariaLabel} />
    </Link>
  );
});

const IconLogoutButton = memo(function IconLogoutButton({ onLogout }: { onLogout: () => void }) {
  const { t } = useTranslation();
  const { el, hovered, setHovered, refCb } = useTooltipTarget<HTMLButtonElement>();
  const handleBlur = useCallback(() => { setHovered(false); }, [setHovered]);
  const handleFocus = useCallback(() => { setHovered(true); }, [setHovered]);
  const handleMouseEnter = useCallback(() => {
    if (isHoverNavigationAvailable()) setHovered(true);
  }, [setHovered]);
  const handleMouseLeave = useCallback(() => {
    if (isHoverNavigationAvailable()) setHovered(false);
  }, [setHovered]);
  return (
    <>
      <button
        aria-label={t("common.logout")}
        className="flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative border-0 text-slate-400 dark:text-slate-500 hover:text-rose-500 dark:hover:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30"
        data-testid="nav-rail-logout"
        onBlur={handleBlur}
        onClick={onLogout}
        onFocus={handleFocus}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        ref={refCb}
        type="button"
      >
        <LogOut className="h-4 w-4" />
      </button>
      <TooltipPortal el={el} hovered={hovered} label={t("common.logout")} />
    </>
  );
});
