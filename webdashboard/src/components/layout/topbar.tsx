import { Link, useLocation } from "@tanstack/react-router";
import {
  ArrowLeft,
  Bell,
  ChevronRight,
  Menu,
  Search,
  UserCircle,
} from "lucide-react";
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

interface MobileNavigationProps {
  backAriaLabel: string;
  backLabel: string;
  isSettings: boolean;
  onOpenMobileMenu?: () => void;
}

interface BreadcrumbProps {
  organizationName: string;
  pageTitle: string;
}

interface NotificationLinkProps {
  ariaLabel: string;
  title: string;
  unreadCount: number;
}

interface ProfileLinkProps {
  altText: string;
  avatarUrl?: string | null;
  title: string;
}

function MobileNavigation({
  backAriaLabel,
  backLabel,
  isSettings,
  onOpenMobileMenu,
}: MobileNavigationProps) {
  if (isSettings) {
    return (
      <Link
        aria-label={backAriaLabel}
        className="flex xl:hidden items-center justify-center h-8 px-2.5 gap-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted text-xs font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary shrink-0"
        data-testid="topbar-back-btn"
        to={ROUTES.HOME}
      >
        <ArrowLeft className="h-4 w-4" />
        <span className="hidden sm:inline">{backLabel}</span>
      </Link>
    );
  }

  if (!onOpenMobileMenu) {
    return null;
  }

  return (
    <Button
      aria-label="Open navigation menu"
      className="p-0 text-muted-foreground hover:text-foreground xl:hidden"
      data-testid="topbar-mobile-menu-btn"
      onClick={onOpenMobileMenu}
      size="md"
      type="button"
      variant="ghost"
    >
      <Menu className="h-4 w-4" />
    </Button>
  );
}

function Breadcrumb({ organizationName, pageTitle }: BreadcrumbProps) {
  return (
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
      <span className="text-foreground font-semibold truncate capitalize">{pageTitle}</span>
    </nav>
  );
}

function NotificationLink({ ariaLabel, title, unreadCount }: NotificationLinkProps) {
  return (
    <Link
      aria-label={ariaLabel}
      className="relative flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary"
      data-testid="topbar-notifications"
      title={title}
      to={ROUTES.NOTIFICATIONS}
    >
      <Bell className="h-4 w-4" />
      {unreadCount > 0 ? (
        <span className="absolute top-1 right-1 flex h-3.5 min-w-[14px] items-center justify-center rounded-full bg-destructive px-1 text-[9px] font-bold text-destructive-foreground leading-none">
          {unreadCount > 99 ? "99+" : unreadCount}
        </span>
      ) : null}
    </Link>
  );
}

function ProfileLink({ altText, avatarUrl, title }: ProfileLinkProps) {
  return (
    <Link
      aria-label="Profile"
      className="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary overflow-hidden ml-1"
      data-testid="topbar-profile-link"
      title={title}
      to={ROUTES.ESS_PROFILE}
    >
      {avatarUrl ? (
        <img
          alt={altText}
          className="h-6 w-6 rounded-md object-cover border border-border"
          src={avatarUrl}
        />
      ) : (
        <UserCircle className="h-5 w-5 text-primary" />
      )}
    </Link>
  );
}

interface TopbarActionsProps {
  onOpenSearch?: () => void;
  profile: ProfileLinkProps;
  unreadCount: number;
}

function TopbarActions({ onOpenSearch, unreadCount, profile }: TopbarActionsProps) {
  const { t } = useTranslation("common");
  const notificationsLabel = t("common.notifications", { defaultValue: "Notifications" });

  return (
    <div className="flex items-center gap-1.5 shrink-0">
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
      <NotificationLink
        ariaLabel={notificationsLabel}
        title={t("common.notifications", { defaultValue: "Thông báo" })}
        unreadCount={unreadCount}
      />
      <div className="flex items-center px-1">
        <ThemeSwitch data-testid="topbar-theme-toggle" />
      </div>
      <div data-testid="topbar-lang-toggle">
        <LanguageSwitcher
          className="h-8 min-h-[32px] px-2 rounded-lg border-0 bg-transparent hover:bg-muted text-muted-foreground hover:text-foreground shadow-none text-xs"
          compact
        />
      </div>
      <ProfileLink {...profile} />
    </div>
  );
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
  const pageTitle = t(routeMeta.breadcrumbKey, routeMeta.defaultBreadcrumb);
  const profileTitle = user?.full_name ?? user?.email ?? "Profile";
  const profileAltText = profileTitle;

  return (
    <header
      aria-label="Top Navigation"
      className="flex h-14 shrink-0 items-center justify-between border-b border-border bg-card/80 px-3 md:px-5 backdrop-blur-md shadow-2xs z-20"
      data-testid="topbar-container"
    >
      <div className="flex items-center gap-2 md:gap-3 min-w-0">
        <MobileNavigation
          backAriaLabel={t("common.back_to_organization", "Quay lại bảng làm việc")}
          backLabel={t("common.back", "Quay lại")}
          isSettings={isSettings}
          onOpenMobileMenu={onOpenMobileMenu}
        />
        <Breadcrumb organizationName={organizationName} pageTitle={pageTitle} />
      </div>
      <div className="flex-1 max-w-md mx-3 hidden sm:flex">
        <SearchTriggerButton
          aria-label="Open command palette (⌘K)"
          data-testid="topbar-search-trigger"
          onClick={onOpenSearch}
          placeholder={t("common.search_placeholder", "Tìm kiếm tính năng, nhân sự...")}
          shortcut="⌘K"
        />
      </div>
      <TopbarActions
        onOpenSearch={onOpenSearch}
        profile={{
          altText: profileAltText,
          avatarUrl: user?.avatar_url,
          title: profileTitle,
        }}
        unreadCount={unreadCount}
      />
    </header>
  );
}

export default Topbar;
