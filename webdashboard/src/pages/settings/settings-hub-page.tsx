import { Link } from "@tanstack/react-router";
import { Settings, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";

import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import StaggerChildren from "@/components/ui/stagger-children";
import { Typography } from "@/components/ui/typography";
import { useMyPermissions } from "@/hooks/use-permissions";
import { cn } from "@/lib/utils";

import { SettingItem, useSettingsNav } from "./hooks/use-settings-nav";

// ── SettingCategoryCard ─────────────────────────────────────────────

function SettingCategoryCard({ item }: { item: SettingItem }) {
  return (
    <Link
      className={cn(
        "group flex flex-row items-start gap-4 p-4 rounded-2xl transition-all duration-300 h-full",
        "glass-premium",
        "hover:scale-[1.03] hover:shadow-2xl hover:shadow-primary-100/10 dark:hover:shadow-primary-900/10",
        "focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900",
        "outline-none",
      )}
      to={item.path as "/"}
    >
      <div
        className={cn(
          "w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition-all duration-300",
          "bg-primary-50 dark:bg-primary-950",
          "group-hover:bg-primary-100 dark:group-hover:bg-primary-900",
          "group-hover:shadow-md group-hover:shadow-primary-200/50 dark:group-hover:shadow-primary-900/30",
          "text-primary-600 dark:text-primary-400",
        )}
      >
        {item.icon}
      </div>
      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <span className="text-sm font-bold text-slate-700 dark:text-slate-200 truncate leading-snug group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
          {item.label}
        </span>
        <span className="text-[11px] text-slate-500 dark:text-slate-400 line-clamp-2 leading-relaxed">
          {item.description}
        </span>
      </div>
    </Link>
  );
}

// ── DangerCard ───────────────────────────────────────────────────────

function DangerCard({ item }: { item: SettingItem }) {
  return (
    <Link
      className={cn(
        "group flex flex-row items-start gap-4 p-4 rounded-2xl transition-all duration-300 h-full",
        "bg-rose-50/70 dark:bg-rose-950/40 border border-rose-100 dark:border-rose-900/50",
        "hover:bg-rose-100/80 dark:hover:bg-rose-950/60 hover:scale-[1.03] hover:shadow-xl",
        "focus-visible:ring-2 focus-visible:ring-rose-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900",
        "outline-none",
      )}
      to={item.path as "/"}
    >
      <div
        className={cn(
          "w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition-all duration-300",
          "bg-rose-100 dark:bg-rose-900/30 text-rose-500 dark:text-rose-400",
          "group-hover:shadow-md group-hover:shadow-rose-200/50 dark:group-hover:shadow-rose-900/30",
        )}
      >
        {item.icon}
      </div>
      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <span className="text-sm font-bold text-rose-700 dark:text-rose-300 truncate leading-snug">
          {item.label}
        </span>
        <span className="text-[11px] text-rose-500/80 dark:text-rose-400/80 line-clamp-2 leading-relaxed">
          {item.description}
        </span>
      </div>
    </Link>
  );
}

// ── SettingsHubHeader ────────────────────────────────────────────────

function SettingsHubHeader() {
  const { t } = useTranslation();
  return (
    <div className="mb-10 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <div className="flex items-center gap-4">
        <div className="p-3 bg-primary-50 dark:bg-primary-950 rounded-2xl border border-primary-100 dark:border-primary-900/50">
          <Settings className="w-7 h-7 text-primary-600 dark:text-primary-400" />
        </div>
        <div>
          <Typography className="text-2xl font-black text-slate-800 dark:text-slate-100" variant="h1">
            {t("common.settings.title")}
          </Typography>
          <Typography className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mt-0.5" variant="p">
            {t("settings.layout.control_panel")}
          </Typography>
        </div>
      </div>
      <div className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700">
        <ShieldCheck className="w-4 h-4 text-slate-400 dark:text-slate-500" />
        <span className="text-[11px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest">
          {t("settings.admin_only")}
        </span>
      </div>
    </div>
  );
}

// ── Skeleton ─────────────────────────────────────────────────────────

function SettingsHubSkeleton() {
  return (
    <div className="space-y-8 animate-in fade-in duration-500" data-testid="settings-hub-skeleton">
      {[4, 2, 7, 4].map((count, i) => (
        <div key={i}>
          <Skeleton className="h-4 w-32 mb-4" />
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {Array.from({ length: count }).map((_, j) => (
              <Skeleton className="h-28 rounded-2xl" key={j} />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

// ── Main Component ───────────────────────────────────────────────────

export default function SettingsHubPage() {
  const { t } = useTranslation();
  const { isLoading: permsLoading } = useMyPermissions();
  const { workspaceGroup, accessGroup, operationsGroup, systemGroup, dangerItem } = useSettingsNav();

  if (permsLoading) return <SettingsHubSkeleton />;

  const totalItems = workspaceGroup.length + accessGroup.length + operationsGroup.length + systemGroup.length;
  if (totalItems === 0 && !dangerItem) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]" data-testid="settings-hub-empty">
        <EmptyState
          description={t("settings.hub.empty.desc", "You don't have permission to access any settings. Contact your administrator.")}
          icon={<Settings className="h-10 w-10" />}
          title={t("settings.hub.empty.title", "No Settings Available")}
        />
      </div>
    );
  }

  const sections = [
    { key: "workspace", titleKey: "settings.layout.workspace", emoji: "🏢", items: workspaceGroup },
    { key: "access", titleKey: "settings.layout.access", emoji: "🔐", items: accessGroup },
    { key: "operations", titleKey: "settings.layout.operations", emoji: "⚙️", items: operationsGroup },
    { key: "system", titleKey: "settings.layout.security_system", emoji: "🖥️", items: systemGroup },
  ];

  return (
    <div className="space-y-10 animate-in fade-in duration-700">
      <SettingsHubHeader />

      {sections.filter((s) => s.items.length > 0).map((section) => (
        <section aria-label={t(section.titleKey)} data-testid={`settings-section-${section.key}`} key={section.key}>
          <Typography
            className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mb-4 flex items-center gap-2"
            variant="p"
          >
            <span className="w-1.5 h-4 rounded-full bg-primary-500 dark:bg-primary-400 inline-block shrink-0" />
            {section.emoji} {t(section.titleKey)}
          </Typography>
          <StaggerChildren className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4" delay={40}>
            {section.items.map((item) => (
              <SettingCategoryCard item={item} key={item.path} />
            ))}
          </StaggerChildren>
        </section>
      ))}

      {!!dangerItem && (
        <section aria-label={dangerItem.label} data-testid="settings-section-danger">
          <Typography
            className="text-xs font-bold text-rose-400 dark:text-rose-500 uppercase tracking-[0.2em] mb-4 flex items-center gap-2"
            variant="p"
          >
            <span className="w-1.5 h-4 rounded-full bg-rose-500 inline-block shrink-0" />
            {"⚠️"} {dangerItem.label}
          </Typography>
          <div className="max-w-sm">
            <DangerCard item={dangerItem} />
          </div>
        </section>
      )}
    </div>
  );
}
