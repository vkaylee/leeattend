import { Link } from "@tanstack/react-router";
import { Settings, ShieldCheck } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import StaggerChildren from "@/components/ui/stagger-children";
import { Typography } from "@/components/ui/typography";
import { useMyPermissions } from "@/hooks/use-permissions";
import { cn } from "@/lib/utils";

import { SettingItem, useSettingsNav } from "./hooks/use-settings-nav";

// ── SettingCategoryCard (Apple HIG Inset Card) ─────────────────────────

function SettingCategoryCard({ item }: { item: SettingItem }) {
  return (
    <Link
      className={cn(
        "group flex flex-row items-start gap-3.5 p-4 rounded-2xl transition-all duration-200 h-full",
        "bg-white dark:bg-slate-900/90 border border-slate-200/80 dark:border-slate-800/80 shadow-2xs",
        "hover:border-primary-500/40 hover:shadow-xs",
        "focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-slate-900",
        "outline-none",
      )}
      to={item.path as "/"}
    >
      <div
        className={cn(
          "w-9 h-9 rounded-xl flex items-center justify-center shrink-0 transition-colors duration-200",
          "bg-primary-50 dark:bg-primary-950/70 border border-primary-100/50 dark:border-primary-900/50",
          "text-primary-600 dark:text-primary-400 shadow-2xs",
        )}
      >
        {item.icon}
      </div>
      <div className="flex flex-col gap-0.5 min-w-0 flex-1">
        <span className="text-xs font-semibold text-slate-800 dark:text-slate-100 truncate leading-snug group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
          {item.label}
        </span>
        <span className="text-[11px] text-slate-400 dark:text-slate-500 line-clamp-2 leading-relaxed">
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
    <div className="mb-8 flex flex-col md:flex-row md:items-center md:justify-between gap-4 pb-4 border-b border-slate-200/60 dark:border-slate-800/60">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 border border-primary-100/50 dark:border-primary-900/50 shadow-2xs">
          <Settings className="w-5 h-5" />
        </div>
        <div>
          <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100" variant="h1">
            {t("common.settings.title")}
          </Typography>
          <Typography className="text-xs font-semibold text-slate-400 dark:text-slate-500 uppercase tracking-wider mt-0.5" variant="p">
            {t("settings.layout.control_panel")}
          </Typography>
        </div>
      </div>
      <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-slate-100 dark:bg-slate-800 border border-slate-200/60 dark:border-slate-700/60">
        <ShieldCheck className="w-3.5 h-3.5 text-slate-400 dark:text-slate-500" />
        <span className="text-[10px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
          {t("settings.admin_only")}
        </span>
      </div>
    </div>
  );
}

// ── Skeleton ─────────────────────────────────────────────────────────

function SettingsHubSkeleton() {
  return (
    <div className="space-y-8 animate-in fade-in duration-300" data-testid="settings-hub-skeleton">
      {[4, 2, 7, 4].map((count, i) => (
        <div key={i}>
          <Skeleton className="h-4 w-32 mb-4 rounded-md" />
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {Array.from({ length: count }).map((_, j) => (
              <Skeleton className="h-24 rounded-2xl" key={j} />
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
  const { workspaceGroup, accessGroup, operationsGroup, systemGroup } = useSettingsNav();
  if (permsLoading) return <SettingsHubSkeleton />;

  const totalItems = workspaceGroup.length + accessGroup.length + operationsGroup.length + systemGroup.length;
  if (totalItems === 0) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]" data-testid="settings-hub-empty">
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
    <div className="space-y-8 animate-in fade-in duration-300">
      <SettingsHubHeader />

      {sections.filter((s) => s.items.length > 0).map((section) => (
        <section aria-label={t(section.titleKey)} data-testid={`settings-section-${section.key}`} key={section.key}>
          <Typography
            className="text-xs font-semibold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-3 flex items-center gap-2 select-none"
            variant="p"
          >
            <span className="w-1.5 h-3.5 rounded-full bg-primary-500 dark:bg-primary-400 inline-block shrink-0" />
            {section.emoji} {t(section.titleKey)}
          </Typography>
          <StaggerChildren className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4" delay={30}>
            {section.items.map((item) => (
              <SettingCategoryCard item={item} key={item.path} />
            ))}
          </StaggerChildren>
        </section>
      ))}
    </div>
  );
}
