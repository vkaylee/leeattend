import type { LucideIcon } from "lucide-react";

import { Link, useLocation } from "@tanstack/react-router";
import { Folder, LayoutDashboard, MoreHorizontal, Timer, User } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { MenuGroupType, NavGroup } from "@/hooks/use-nav-menu-items";

import Button from "@/components/ui/button";
import { getActiveNavGroup, useFilteredMenuItems } from "@/hooks/use-nav-menu-items";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

interface BottomNavProps {
  activeGroup: NavGroup | null;
  onMoreClick: () => void;
  unreadCount?: number;
}

interface BottomNavItemProps {
  ariaCurrent?: "page";
  "data-testid": string;
  icon: LucideIcon;
  isActive: boolean;
  label: string;
  to: string;
}

interface PrimaryNavItem {
  group: NavGroup;
  icon: LucideIcon;
  label: string;
  testId: string;
  to: string;
}

interface PrimaryNavConfig {
  group: Exclude<NavGroup, "dashboard">;
  icon: LucideIcon;
  label: string;
  section: string;
  testId: string;
}

interface MoreNavItemProps {
  label: string;
  onClick: () => void;
  unreadCount: number;
}

function getFirstGroupItem(groups: MenuGroupType[], section: string) {
  return groups.find((group) => group.section === section)?.items[0];
}

function MoreNavItem({ onClick, unreadCount, label }: MoreNavItemProps) {
  return (
    <Button
      aria-label={label}
      className="flex flex-col items-center justify-center flex-1 h-full relative transition-transform active:scale-95 focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus:outline-none"
      data-testid="bottom-nav-more"
      onClick={onClick}
      type="button"
      variant="ghost"
    >
      <div className="relative">
        <MoreHorizontal className="w-6 h-6 transition-colors text-slate-400 dark:text-slate-500" />
        {unreadCount > 0 ? (
          <span className="absolute -top-1.5 -right-2 min-w-4 h-4 px-1 rounded-full bg-rose-500 text-white text-[9px] font-black flex items-center justify-center leading-none">
            {unreadCount > 99 ? "99+" : unreadCount}
          </span>
        ) : null}
      </div>
      <span className="text-[10px] font-bold mt-1 text-slate-400 dark:text-slate-500">{label}</span>
    </Button>
  );
}

function BottomNavItem({
  ariaCurrent,
  icon: Icon,
  isActive,
  label,
  to,
  "data-testid": testId,
}: BottomNavItemProps) {
  const iconClassName = cn(
    "w-6 h-6 transition-colors",
    isActive ? "text-primary-600 dark:text-primary-400" : "text-slate-400 dark:text-slate-500",
  );
  const labelClassName = cn(
    "text-[10px] font-bold mt-1 transition-colors",
    isActive ? "text-primary-600 dark:text-primary-400" : "text-slate-400 dark:text-slate-500",
  );

  return (
    <Link
      aria-current={ariaCurrent}
      className="flex flex-col items-center justify-center flex-1 h-full relative transition-transform active:scale-95"
      data-testid={testId}
      to={to as "/"}
    >
      <div className="relative">
        <Icon className={iconClassName} />
      </div>
      <span className={labelClassName}>{label}</span>
      {isActive ? <span className="absolute top-0 left-1/2 -translate-x-1/2 w-8 h-1 rounded-b-full bg-primary-500 dark:bg-primary-400" /> : null}
    </Link>
  );
}

export default function BottomNav({ activeGroup, onMoreClick, unreadCount = 0 }: BottomNavProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const { groups } = useFilteredMenuItems();
  const currentGroup = activeGroup ?? getActiveNavGroup(location.pathname);
  const primaryConfigs: PrimaryNavConfig[] = [
    {
      group: "time",
      icon: Timer,
      label: t("common.sections.time_leave"),
      section: "time",
      testId: "bottom-nav-time",
    },
    {
      group: "org",
      icon: Folder,
      label: t("common.sections.organization"),
      section: "org",
      testId: "bottom-nav-org",
    },
    {
      group: "ess",
      icon: User,
      label: t("common.sections.self_service"),
      section: "ess",
      testId: "bottom-nav-ess",
    },
  ];
  const primaryItems: PrimaryNavItem[] = [
    {
      group: "dashboard",
      icon: LayoutDashboard,
      label: t("common.dashboard"),
      testId: "bottom-nav-dashboard",
      to: ROUTES.HOME,
    },
    ...primaryConfigs.flatMap((config) => {
      const item = getFirstGroupItem(groups, config.section);
      return item ? [{ ...config, to: item.to }] : [];
    }),
  ];

  return (
    <nav
      aria-label={t("common.settings.navigation", "Navigation")}
      className="lg:hidden fixed bottom-0 left-0 right-0 z-40 bg-white/90 dark:bg-slate-950/90 backdrop-blur-xl border-t border-slate-100 dark:border-slate-800 shadow-lg shadow-slate-200/50 dark:shadow-black/30 pb-safe pl-safe pr-safe"
      role="navigation"
    >
      <div className="flex items-center justify-around h-16 px-2">
        {primaryItems.map((item) => (
          <BottomNavItem
            ariaCurrent={currentGroup === item.group ? "page" : undefined}
            data-testid={item.testId}
            icon={item.icon}
            isActive={currentGroup === item.group}
            key={item.group}
            label={item.label}
            to={item.to}
          />
        ))}
        <MoreNavItem label={t("common.more", "More")} onClick={onMoreClick} unreadCount={unreadCount} />
      </div>
    </nav>
  );
}
