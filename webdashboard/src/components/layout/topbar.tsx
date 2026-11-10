import { Link, useLocation } from "@tanstack/react-router";
import {
  ArrowLeft,
  Bell,
  ChevronRight,
  Menu,
  Search,
  UserCircle,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LanguageSwitcher } from "@/components/ui/language-switcher";
import { SearchTriggerButton } from "@/components/ui/search-input";
import { ThemeSwitch } from "@/components/ui/theme-switch";
import { useNotifications } from "@/hooks/use-notifications";
import { useProfile } from "@/hooks/use-profile";
import { getRouteMetadata } from "@/lib/route-metadata";
import { ROUTES } from "@/lib/routes";
export interface TopbarProps {
  isSettings?: boolean;
  onOpenMobileMenu?: () => void;
  onOpenSearch?: () => void;
  organizationName?: string;
}

export function Topbar({
  isSettings = false,
  onOpenMobileMenu,
  onOpenSearch,
  organizationName = "LeeAttend",
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
  const { user } = useProfile();
  const { unreadCount } = useNotifications();

  const routeMeta = getRouteMetadata(location.pathname);
  const pageTitle = routeMeta.breadcrumbKey
    ? t(routeMeta.breadcrumbKey, routeMeta.defaultBreadcrumb)
    : routeMeta.defaultBreadcrumb;

  return (
    <header
      aria-label="Top Navigation"
      className="flex h-14 shrink-0 items-center justify-between border-b border-border bg-card/80 px-3 md:px-5 backdrop-blur-md shadow-2xs z-20"
      data-testid="topbar-container"
    >
      {/* ── Left: Mobile Hamburger / Back to App & Breadcrumb ── */}
      <div className="flex items-center gap-2 md:gap-3 min-w-0">
        {isSettings ? (
          <Link
            aria-label={t("common.back_to_workspace", "Quay lại bảng làm việc")}
            className="flex md:hidden items-center justify-center h-8 px-2.5 gap-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted text-xs font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary shrink-0"
            data-testid="topbar-back-btn"
            to={ROUTES.HOME}
          >
            <ArrowLeft className="h-4 w-4" />
            <span className="hidden sm:inline">{t("common.back", "Quay lại")}</span>
          </Link>
        ) : onOpenMobileMenu ? (
          <Button
            aria-label="Open navigation menu"
            className="p-0 text-muted-foreground hover:text-foreground md:hidden"
            data-testid="topbar-mobile-menu-btn"
            onClick={onOpenMobileMenu}
            size="md"
            type="button"
            variant="ghost"
          >
            <Menu className="h-4 w-4" />
          </Button>
        ) : null}

        <nav
          aria-label="Breadcrumbs"
          className="flex items-center gap-1.5 text-xs md:text-sm font-medium truncate"
          data-testid="topbar-breadcrumb"
        >
          <Link
            className="text-muted-foreground hover:text-foreground transition-colors truncate max-w-[120px] md:max-w-none"
            to={ROUTES.HOME}
          >
            {organizationName}
          </Link>
          <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
          <span className="text-foreground font-semibold truncate capitalize">
            {pageTitle}
          </span>
        </nav>
      </div>

      {/* ── Center: Search Bar Trigger (Cmd+K) ── */}
      <div className="flex-1 max-w-md mx-3 hidden sm:flex">
        <SearchTriggerButton
          aria-label="Open command palette (⌘K)"
          data-testid="topbar-search-trigger"
          onClick={onOpenSearch}
          placeholder={t("common.search_placeholder", "Tìm kiếm tính năng, nhân sự...")}
          shortcut="⌘K"
        />
      </div>

      {/* ── Right: Quick Actions ── */}
      <div className="flex items-center gap-1.5 shrink-0">
        {/* Mobile search trigger icon */}
        <Button
          aria-label="Open search"
          className="p-0 text-muted-foreground hover:text-foreground transition-colors focus-visible:ring-2 focus-visible:ring-primary sm:hidden"
          data-testid="topbar-search-trigger-mobile"
          onClick={onOpenSearch}
          size="md"
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

        {/* Theme Toggle (VitePress appearance switch) */}
        <div className="flex items-center px-1">
          <ThemeSwitch data-testid="topbar-theme-toggle" />
        </div>

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
