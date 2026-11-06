import { Layers, Search } from "lucide-react";
import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import type { NavGroup } from "@/hooks/use-nav-menu-items";

import AppCard from "@/components/features/app-launcher/app-card";
import QuickStatsBar from "@/components/features/app-launcher/quick-stats-bar";
import { openDsmLauncher } from "@/components/features/dsm-launcher";
import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import StaggerChildren from "@/components/ui/stagger-children";
import { Typography } from "@/components/ui/typography";
import { useMyPermissions } from "@/hooks/use-permissions";
import { useProfile } from "@/hooks/use-profile";
import { useQuickAccess } from "@/hooks/use-quick-access";
import { APP_REGISTRY, GROUP_LABEL_KEYS, type AppDefinition } from "@/lib/app-registry";
import { isFeatureEnabled } from "@/lib/feature-flags";

// ── Helpers ────────────────────────────────────────────────────────

function filterApps(
  apps: AppDefinition[],
  hasAnyPermission: (...perms: string[]) => boolean,
  systemRoles: string[],
): AppDefinition[] {
  return apps.filter((app) => {
    if (app.featureFlag && !isFeatureEnabled(app.featureFlag)) {
      return false;
    }
    if (app.systemRoles && app.systemRoles.length > 0) {
      return app.systemRoles.some((role) => systemRoles.includes(role));
    }
    if (app.permissions && app.permissions.length > 0) {
      return hasAnyPermission(...app.permissions);
    }
    return true;
  });
}

// Group ordering for display
const GROUP_ORDER: NavGroup[] = ["dashboard", "org", "time", "shifts", "payroll", "system", "ess"];

// ── Component ──────────────────────────────────────────────────────

export default function AppLauncherPage() {
  const { t } = useTranslation();
  const { hasAnyPermission, isLoading: permsLoading } = useMyPermissions();
  const { user } = useProfile();
  const { top3, recordAccess } = useQuickAccess();

  // Filter apps by permissions
  const visibleApps = useMemo(() => {
    if (permsLoading) return [];
    return filterApps(APP_REGISTRY, hasAnyPermission, user?.system_roles ?? []);
  }, [permsLoading, hasAnyPermission, user?.system_roles]);

  // Quick Access apps (respect permission filtering)
  const quickAccessApps = useMemo(() => {
    return top3
      .map((id) => visibleApps.find((app) => app.id === id))
      .filter(Boolean) as AppDefinition[];
  }, [top3, visibleApps]);

  // Group apps for "All Apps" section
  const groupedApps = useMemo(() => {
    const map = new Map<NavGroup, AppDefinition[]>();
    for (const group of GROUP_ORDER) {
      const apps = visibleApps.filter((app) => app.group === group);
      if (apps.length > 0) {
        map.set(group, apps);
      }
    }
    return map;
  }, [visibleApps]);

  // Loading state
  if (permsLoading) {
    return (
      <div className="space-y-8 animate-in fade-in duration-500" data-testid="launcher-skeleton">
        {/* Stats skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div className="flex items-center gap-3 p-4 rounded-2xl bg-card border border-slate-100 dark:border-slate-800" key={i}>
              <Skeleton className="w-10 h-10 rounded-xl shrink-0" />
              <div className="space-y-1.5 flex-1">
                <Skeleton className="h-3 w-16" />
                <Skeleton className="h-5 w-12" />
              </div>
            </div>
          ))}
        </div>
        {/* Cards skeleton */}
        <div>
          <Skeleton className="h-5 w-32 mb-4" />
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {Array.from({ length: 8 }).map((_, i) => (
              <Skeleton className="h-32 rounded-2xl" key={i} />
            ))}
          </div>
        </div>
      </div>
    );
  }

  // Empty state
  if (visibleApps.length === 0) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <EmptyState
          data-testid="launcher-empty"
          description={t("appLauncher.empty.desc", "You don't have access to any applications. Contact your administrator.")}
          icon={<Layers className="h-10 w-10" />}
          title={t("appLauncher.empty.title", "No Apps Available")}
        />
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      {/* Quick Stats Bar */}
      <section aria-label={t("dashboard.hr.quick_stats", "Quick Stats")} data-testid="launcher-quick-stats">
        <QuickStatsBar />
      </section>

      {/* DSM 7 Feature Search Trigger Bar */}
      <section aria-label="Tìm kiếm tính năng" data-testid="launcher-hero-search">
        <button
          type="button"
          onClick={openDsmLauncher}
          className="w-full flex items-center justify-between p-4 rounded-2xl glass-premium hover:shadow-xl transition-all duration-300 border border-slate-200/80 dark:border-slate-800/80 group cursor-pointer text-left"
        >
          <div className="flex items-center gap-3 text-slate-500 dark:text-slate-400">
            <Search className="w-5 h-5 text-primary-500 group-hover:scale-110 transition-transform" />
            <span className="text-sm font-medium text-slate-600 dark:text-slate-300">
              {t("appLauncher.searchHeroPlaceholder", "Tìm nhanh ứng dụng, chức năng hoặc cài đặt hệ thống...")}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-slate-400 hidden sm:inline">Phím tắt</span>
            <kbd className="px-2.5 py-1 text-xs font-semibold text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 rounded-lg border border-slate-200 dark:border-slate-700 shadow-xs">
              ⌘K / Ctrl+K
            </kbd>
          </div>
        </button>
      </section>

      {/* Quick Access */}
      {quickAccessApps.length > 0 && (
        <section aria-label={t("appLauncher.quickAccess", "Quick Access")} data-testid="launcher-quick-access">
          <Typography className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mb-4" variant="p">
            {t("appLauncher.quickAccess", "📌 Quick Access")}
          </Typography>
          <StaggerChildren className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4" delay={50}>
            {quickAccessApps.map((app) => (
              <AppCard app={app} key={app.id} onNavigate={recordAccess} />
            ))}
          </StaggerChildren>
        </section>
      )}

      {/* All Apps by Group */}
      <section aria-label={t("appLauncher.allApps", "All Apps")} data-testid="launcher-all-apps">
        <Typography className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mb-6" variant="p">
          {t("appLauncher.allApps", "🗂️ All Apps")}
        </Typography>

        <div className="space-y-8">
          {Array.from(groupedApps.entries()).map(([group, apps]) => (
            <div data-testid={`launcher-group-${group}`} key={group}>
              {/* Group header */}
              <Typography
                className="text-sm font-bold text-slate-600 dark:text-slate-300 mb-3 flex items-center gap-2"
                variant="p"
              >
                <span className="w-1.5 h-4 rounded-full bg-primary-500 dark:bg-primary-400 inline-block shrink-0" />
                {t(GROUP_LABEL_KEYS[group], group)}
              </Typography>

              {/* App grid */}
              <StaggerChildren
                className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4"
                delay={30}
              >
                {apps.map((app) => (
                  <AppCard app={app} key={app.id} onNavigate={recordAccess} />
                ))}
              </StaggerChildren>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
