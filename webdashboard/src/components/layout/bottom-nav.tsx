import Button from "@/components/ui/button";
import { Link, useLocation } from "@tanstack/react-router";
import { LayoutDashboard, Timer, Folder, User, MoreHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { NavGroup } from "@/hooks/use-nav-menu-items";

import { useFilteredMenuItems, getActiveNavGroup } from "@/hooks/use-nav-menu-items";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

interface BottomNavProps {
  activeGroup: NavGroup | null;
  onMoreClick: () => void;
  unreadCount?: number;
}

export default function BottomNav({ activeGroup, onMoreClick, unreadCount = 0 }: BottomNavProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const { groups } = useFilteredMenuItems();

  // Determine active group from path if not explicitly set
  const pathGroup = getActiveNavGroup(location.pathname);
  const currentGroup = activeGroup ?? pathGroup;

  // Find first visible item for each primary group
  const findFirstItem = (section: string) => {
    const group = groups.find((g) => g.section === section);
    return group?.items[0];
  };

  const timeItem = findFirstItem("time");
  const orgItem = findFirstItem("org");
  const essItem = findFirstItem("ess");

  return (
    <nav
      aria-label={t("common.settings.navigation", "Navigation")}
      className="lg:hidden fixed bottom-0 left-0 right-0 z-40 bg-white/90 dark:bg-slate-950/90 backdrop-blur-xl border-t border-slate-100 dark:border-slate-800 shadow-lg shadow-slate-200/50 dark:shadow-black/30 pb-safe pl-safe pr-safe"
      role="navigation"
    >
      <div className="flex items-center justify-around h-16 px-2">
        {/* Dashboard */}
        <BottomNavItem
          ariaCurrent={currentGroup === "dashboard" ? "page" : undefined}
          data-testid="bottom-nav-dashboard"
          icon={LayoutDashboard}
          isActive={currentGroup === "dashboard"}
          label={t("common.dashboard")}
          to={ROUTES.HOME}
        />

        {/* Time & Leave */}
        {timeItem ? (
          <BottomNavItem
            ariaCurrent={currentGroup === "time" ? "page" : undefined}
            data-testid="bottom-nav-time"
            icon={Timer}
            isActive={currentGroup === "time"}
            label={t("common.sections.time_leave")}
            to={timeItem.to}
          />
        ) : null}

        {/* Organization */}
        {orgItem ? (
          <BottomNavItem
            ariaCurrent={currentGroup === "org" ? "page" : undefined}
            data-testid="bottom-nav-org"
            icon={Folder}
            isActive={currentGroup === "org"}
            label={t("common.sections.organization")}
            to={orgItem.to}
          />
        ) : null}

        {/* Self-Service */}
        {essItem ? (
          <BottomNavItem
            ariaCurrent={currentGroup === "ess" ? "page" : undefined}
            data-testid="bottom-nav-ess"
            icon={User}
            isActive={currentGroup === "ess"}
            label={t("common.sections.self_service")}
            to={essItem.to}
          />
        ) : null}

        {/* More */}
        <Button
          aria-label={t("common.more", "More")}
          className="flex flex-col items-center justify-center flex-1 !h-full !min-h-0 !px-0 !py-0 relative transition-transform active:scale-95 focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus:outline-none"
          data-testid="bottom-nav-more"
          onClick={onMoreClick}
          type="button"
          size="sm"
          variant="ghost"
        >
          <div className="relative">
            <MoreHorizontal className={cn(
              "w-6 h-6 transition-colors",
              "text-slate-400 dark:text-slate-500"
            )} />
            {unreadCount > 0 ? (
              <span className="absolute -top-1.5 -right-2 min-w-4 h-4 px-1 rounded-full bg-rose-500 text-white text-[9px] font-black flex items-center justify-center leading-none">
                {unreadCount > 99 ? "99+" : unreadCount}
              </span>
            ) : null}
          </div>
          <span className="text-[10px] font-bold mt-1 text-slate-400 dark:text-slate-500">
            {t("common.more", "More")}
          </span>
        </Button>
      </div>
    </nav>
  );
}

// ── Internal: Bottom Nav Item ────────────────────────────────────────

function BottomNavItem({
  ariaCurrent,
  icon: Icon,
  isActive,
  label,
  to,
  "data-testid": testId,
}: {
  ariaCurrent?: "page";
  "data-testid": string;
  icon: React.ElementType;
  isActive: boolean;
  label: string;
  to: string;
}) {
  return (
    <Link
      aria-current={ariaCurrent}
      className="flex flex-col items-center justify-center flex-1 h-full relative transition-transform active:scale-95"
      data-testid={testId}
      to={to as "/"}
    >
      <div className="relative">
        <Icon className={cn(
          "w-6 h-6 transition-colors",
          isActive
            ? "text-primary-600 dark:text-primary-400"
            : "text-slate-400 dark:text-slate-500"
        )} />
      </div>
      <span className={cn(
        "text-[10px] font-bold mt-1 transition-colors",
        isActive
          ? "text-primary-600 dark:text-primary-400"
          : "text-slate-400 dark:text-slate-500"
      )}>
        {label}
      </span>
      {isActive ? <span className="absolute top-0 left-1/2 -translate-x-1/2 w-8 h-1 rounded-b-full bg-primary-500 dark:bg-primary-400" /> : null}
    </Link>
  );
}
