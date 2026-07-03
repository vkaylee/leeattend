import { Link } from "@tanstack/react-router";
import { Pin, PinOff, Sun, Moon, Bell, LogOut, UserCircle } from "lucide-react";
import React, { useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

import type { NavGroup } from "@/hooks/use-nav-menu-items";

import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useLanguage } from "@/hooks/use-language";
import { useFilteredMenuItems } from "@/hooks/use-nav-menu-items";
import { useNotifications } from "@/hooks/use-notifications";
import { useProfile } from "@/hooks/use-profile";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

// ── Types ──────────────────────────────────────────────────────────

interface IconRailProps {
  activeGroup: NavGroup | null;
  isPinned: boolean;
  onGroupClick: (group: NavGroup) => void;
  onGroupHover: (group: NavGroup | null) => void;
  onShowWorkspaceSwitcher: () => void;
  onTogglePin: () => void;
  workspaceLogoUrl?: string;
  workspaceName?: string;
}

// ── Component ──────────────────────────────────────────────────────

export default function IconRail({
  activeGroup,
  isPinned,
  onGroupHover,
  onGroupClick,
  onTogglePin,
  workspaceName,
  workspaceLogoUrl,
  onShowWorkspaceSwitcher,
}: IconRailProps) {
  const { t } = useTranslation();
  const { dashboard, settings, groups } = useFilteredMenuItems();
  const { user } = useProfile();
  const { mode, setMode } = useDarkModeContext();
  const { currentLanguage, changeLanguage, currentOption } = useLanguage();
  const { unreadCount } = useNotifications();

  const handleLogout = () => {
    localStorage.clear();
    window.location.href = ROUTES.LOGIN;
  };

  // ── Theme cycle: Light → System → Dark ─────────────────────────
  const cycleTheme = () => {
    const modes: ("light" | "system" | "dark")[] = ["light", "system", "dark"];
    const idx = modes.indexOf(mode);
    setMode(modes[(idx + 1) % 3]);
  };
  const ThemeIcon = mode === "dark" ? Moon : Sun;

  // ── Language code label (2-char) ────────────────────────────────
  const langLabel = currentLanguage.toUpperCase(); // "EN", "VI", "ZH"
  const langTooltip = currentOption?.nativeName ?? langLabel;

  // ── Language cycle: EN → VI → ZH ────────────────────────────────
  const cycleLanguage = () => {
    const codes: ("en" | "vi" | "zh")[] = ["vi", "en", "zh"];
    const idx = codes.indexOf(currentLanguage);
    const next = codes[(idx + 1) % codes.length];
    void changeLanguage(next);
  };

  return (
    <nav
      aria-label={t("common.settings.navigation", "Navigation")}
      className="hidden md:flex fixed inset-y-0 left-0 z-50 w-14 flex-col overflow-x-clip bg-white dark:bg-slate-950 border-r border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-200/50 dark:shadow-black/30"
      role="navigation"
    >
      {/* Logo */}
      <button
        aria-label={workspaceName ?? "LeeAttend"}
        className="flex items-center justify-center h-14 w-full shrink-0 hover:bg-slate-50 dark:hover:bg-slate-900 transition-colors group"
        data-testid="nav-rail-logo"
        onClick={onShowWorkspaceSwitcher}
        type="button"
      >
        {workspaceLogoUrl ? (
          <div
            className="w-8 h-8 bg-contain bg-no-repeat bg-center drop-shadow-sm"
            role="img"
            style={{ backgroundImage: `url(${workspaceLogoUrl})` }}
          />
        ) : (
          <div className="h-8 w-8 rounded-xl bg-gradient-to-tr from-primary-600 to-primary-400 shadow-lg shadow-primary-200 dark:shadow-primary-900/20 flex items-center justify-center text-white font-black text-sm tracking-tighter italic">
            {workspaceName?.charAt(0).toUpperCase() ?? "L"}
          </div>
        )}
      </button>

      <div className="h-px bg-slate-100 dark:bg-slate-800 mx-3 shrink-0" />

      {/* Nav Groups */}
      <div className="flex-1 overflow-y-auto custom-scrollbar py-3 space-y-1 px-1.5" role="list">
        <RailIconButton
          group="dashboard"
          icon={dashboard.icon}
          isActive={activeGroup === "dashboard"}
          label={dashboard.label}
          onClick={() => { onGroupClick("dashboard"); }}
          onMouseEnter={() => { onGroupHover("dashboard"); }}
          onMouseLeave={() => { onGroupHover(null); }}
          testId="nav-rail-group-dashboard"
          to={ROUTES.HOME}
        />

        {!!settings && (
          <RailIconButton
            group="settings"
            icon={settings.icon}
            isActive={activeGroup === "settings"}
            label={settings.label}
            onClick={() => { onGroupClick("settings"); }}
            onMouseEnter={() => { onGroupHover("settings"); }}
            onMouseLeave={() => { onGroupHover(null); }}
            testId="nav-rail-group-settings"
            to={settings.to}
          />
        )}

        {groups.map((g) => (
          <RailIconButton
            group={g.section as NavGroup}
            icon={g.sectionIcon}
            isActive={activeGroup === g.section}
            key={g.section}
            label={g.label}
            onClick={() => { onGroupClick(g.section as NavGroup); }}
            onMouseEnter={() => { onGroupHover(g.section as NavGroup); }}
            onMouseLeave={() => { onGroupHover(null); }}
            testId={`nav-rail-group-${g.section}`}
          />
        ))}
      </div>

      {/* Bottom Controls: Pin → Theme → Language → Notifications → Avatar → Logout */}
      <div className="shrink-0 border-t border-slate-100 dark:border-slate-800 py-2 space-y-1 px-1.5">

        {/* Pin toggle */}
        <RailControlButton
          active={isPinned}
          data-testid="nav-rail-pin-toggle"
          icon={isPinned ? PinOff : Pin}
          label={isPinned ? t("common.collapse", "Unpin") : t("common.expand", "Pin")}
          onClick={onTogglePin}
        />

        {/* Dark mode cycle */}
        <RailControlButton
          data-testid="nav-rail-theme-toggle"
          icon={ThemeIcon}
          label={t(`common.theme.${mode}`, mode)}
          onClick={cycleTheme}
        />

        {/* Language cycle */}
        <LanguageButton
          langLabel={langLabel}
          onClick={cycleLanguage}
          tooltip={langTooltip}
        />

        {/* Notifications */}
        <NotificationsButton unreadCount={unreadCount} />

        {/* Avatar */}
        <AvatarButton userName={user?.full_name ?? undefined} />

        {/* Logout */}
        <LogoutButton onLogout={handleLogout} />
      </div>
    </nav>
  );
}

// ── Internal: Rail Control Button ──────────────────────────────────

function RailControlButton({
  icon: Icon,
  label,
  active,
  onClick,
  ...props
}: {
  icon: React.ElementType;
  label: string;
  active?: boolean;
  onClick: () => void;
  "data-testid": string;
}) {
  const ref = useRef<HTMLButtonElement>(null);
  const [hovered, setHovered] = useState(false);

  return (
    <>
      <button
        ref={ref}
        aria-label={label}
        className={cn(
          "flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative",
          active
            ? "text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950"
            : "text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-900",
        )}
        data-testid={props["data-testid"]}
        onBlur={() => { setHovered(false); }}
        onClick={onClick}
        onFocus={() => { setHovered(true); }}
        onMouseEnter={() => { setHovered(true); }}
        onMouseLeave={() => { setHovered(false); }}
        type="button"
      >
        <Icon className="h-4 w-4" />
      </button>
      {/* Tooltip rendered via portal to escape overflow-x-clip on parent */}
      {hovered && ref.current && createPortal(
        <RailTooltipLabel label={label} targetRef={ref} />,
        document.body,
      )}
    </>
  );
}

// ── Internal: Rail Icon Button ─────────────────────────────────────

interface RailIconButtonProps {
  group: NavGroup;
  icon?: React.ElementType;
  isActive: boolean;
  label: string;
  onClick: () => void;
  onMouseEnter: () => void;
  onMouseLeave: () => void;
  testId: string;
  to?: string;
}

function RailIconButton({
  icon: Icon,
  isActive,
  label,
  testId,
  group,
  onMouseEnter,
  onMouseLeave,
  onClick,
  to,
}: RailIconButtonProps) {
  const ref = useRef<HTMLAnchorElement | HTMLButtonElement>(null);
  const [hovered, setHovered] = useState(false);

  const handleEnter = () => {
    setHovered(true);
    onMouseEnter();
  };
  const handleLeave = () => {
    setHovered(false);
    onMouseLeave();
  };

  const sharedClasses = cn(
    "flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative",
    isActive
      ? "text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950"
      : "text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-900"
  );

  const activeIndicator = isActive ? (
    <span className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-6 rounded-r-full bg-primary-500 dark:bg-primary-400" />
  ) : null;

  const content = (
    <>
      {activeIndicator}
      {Icon ? <Icon className="h-4 w-4" /> : null}
    </>
  );

  const tooltipPortal = hovered && ref.current
    ? createPortal(
        <RailTooltipLabel label={label} targetRef={ref} />,
        document.body,
      )
    : null;

  if (to && (group === "dashboard" || group === "settings")) {
    return (
      <>
        <Link
          ref={ref as React.Ref<HTMLAnchorElement>}
          aria-label={label}
          className={sharedClasses}
          data-testid={testId}
          onBlur={() => { setHovered(false); }}
          onFocus={() => { setHovered(true); }}
          onMouseEnter={handleEnter}
          onMouseLeave={handleLeave}
          to={to as "/"}
        >
          {content}
        </Link>
        {tooltipPortal}
      </>
    );
  }

  return (
    <>
      <button
        ref={ref as React.Ref<HTMLButtonElement>}
        aria-label={label}
        className={sharedClasses}
        data-testid={testId}
        onClick={onClick}
        onBlur={() => { setHovered(false); }}
        onFocus={() => { setHovered(true); }}
        onMouseEnter={handleEnter}
        onMouseLeave={handleLeave}
        type="button"
      >
        {content}
      </button>
      {tooltipPortal}
    </>
  );
}

// ── Internal: Tooltip (portal-positioned) ──────────────────────────

function RailTooltipLabel({
  label,
  targetRef,
}: {
  label: string;
  targetRef: React.RefObject<HTMLElement | null>;
}) {
  const rect = targetRef.current?.getBoundingClientRect();
  if (!rect) return null;

  const top = rect.top + rect.height / 2;
  const left = rect.right + 12; // ml-3 (12px)

  return (
    <span
      className="fixed px-3 py-1.5 bg-slate-800 dark:bg-slate-200 text-white dark:text-slate-800 text-xs font-bold rounded-xl whitespace-nowrap pointer-events-none shadow-lg z-[60]"
      role="tooltip"
      style={{ top, left, transform: "translateY(-50%)" }}
    >
      {label}
    </span>
  );
}

// ── Internal: Language Button ──────────────────────────────────────

function LanguageButton({
  langLabel,
  onClick,
  tooltip,
}: {
  langLabel: string;
  onClick: () => void;
  tooltip: string;
}) {
  const { t } = useTranslation();
  const ref = useRef<HTMLButtonElement>(null);
  const [hovered, setHovered] = useState(false);

  return (
    <>
      <button
        ref={ref}
        aria-label={t("appLauncher.language", "Language") + ": " + tooltip}
        className="flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-900"
        data-testid="nav-rail-lang-toggle"
        onBlur={() => { setHovered(false); }}
        onClick={onClick}
        onFocus={() => { setHovered(true); }}
        onMouseEnter={() => { setHovered(true); }}
        onMouseLeave={() => { setHovered(false); }}
        type="button"
      >
        <span className="text-[11px] font-black tracking-tighter">
          {langLabel}
        </span>
      </button>
      {hovered && ref.current && createPortal(
        <RailTooltipLabel label={tooltip} targetRef={ref} />,
        document.body,
      )}
    </>
  );
}

// ── Internal: Notifications Button ────────────────────────────────

function NotificationsButton({ unreadCount }: { unreadCount: number }) {
  const { t } = useTranslation();
  const ref = useRef<HTMLAnchorElement>(null);
  const [hovered, setHovered] = useState(false);

  const label = t("common.notifications") + (unreadCount > 0 ? ` (${unreadCount})` : "");

  return (
    <>
      <Link
        ref={ref}
        aria-label={t("common.notifications")}
        className="flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative text-slate-400 dark:text-slate-500 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-900"
        data-testid="nav-rail-notifications"
        onBlur={() => { setHovered(false); }}
        onFocus={() => { setHovered(true); }}
        onMouseEnter={() => { setHovered(true); }}
        onMouseLeave={() => { setHovered(false); }}
        to={ROUTES.NOTIFICATIONS}
      >
        <div className="relative">
          <Bell className="h-4 w-4" />
          {unreadCount > 0 && (
            <span className="absolute -top-1.5 -right-1.5 min-w-[16px] h-4 px-1 rounded-full bg-rose-500 text-white text-[9px] font-black flex items-center justify-center leading-none">
              {unreadCount > 99 ? "99+" : unreadCount}
            </span>
          )}
        </div>
      </Link>
      {hovered && ref.current && createPortal(
        <RailTooltipLabel label={label} targetRef={ref} />,
        document.body,
      )}
    </>
  );
}

// ── Internal: Avatar Button ────────────────────────────────────────

function AvatarButton({ userName }: { userName?: string }) {
  const { t } = useTranslation();
  const ref = useRef<HTMLAnchorElement>(null);
  const [hovered, setHovered] = useState(false);

  return (
    <>
      <Link
        ref={ref}
        aria-label={t("common.profile")}
        className="flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative hover:bg-slate-50 dark:hover:bg-slate-900"
        data-testid="nav-rail-user-avatar"
        onBlur={() => { setHovered(false); }}
        onFocus={() => { setHovered(true); }}
        onMouseEnter={() => { setHovered(true); }}
        onMouseLeave={() => { setHovered(false); }}
        to={ROUTES.ESS_PROFILE}
      >
        <div className="w-7 h-7 rounded-lg bg-slate-800 dark:bg-slate-200 text-white dark:text-slate-800 flex items-center justify-center font-bold text-[10px] shadow-sm group-hover:bg-primary-600 dark:group-hover:bg-primary-500 group-hover:text-white transition-all duration-300">
          {userName ? userName.charAt(0).toUpperCase() : (
            <UserCircle className="h-4 w-4" />
          )}
        </div>
      </Link>
      {hovered && ref.current && createPortal(
        <RailTooltipLabel label={t("common.profile")} targetRef={ref} />,
        document.body,
      )}
    </>
  );
}

// ── Internal: Logout Button ────────────────────────────────────────

function LogoutButton({ onLogout }: { onLogout: () => void }) {
  const { t } = useTranslation();
  const ref = useRef<HTMLButtonElement>(null);
  const [hovered, setHovered] = useState(false);

  return (
    <>
      <button
        ref={ref}
        aria-label={t("common.logout")}
        className="flex items-center justify-center w-full h-11 rounded-xl transition-all duration-200 group relative text-slate-400 dark:text-slate-500 hover:text-rose-500 dark:hover:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30"
        data-testid="nav-rail-logout"
        onBlur={() => { setHovered(false); }}
        onClick={onLogout}
        onFocus={() => { setHovered(true); }}
        onMouseEnter={() => { setHovered(true); }}
        onMouseLeave={() => { setHovered(false); }}
        type="button"
      >
        <LogOut className="h-4 w-4" />
      </button>
      {hovered && ref.current && createPortal(
        <RailTooltipLabel label={t("common.logout")} targetRef={ref} />,
        document.body,
      )}
    </>
  );
}
