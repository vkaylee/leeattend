import React from "react";

import { TFunction } from "i18next";

import type { MonitorDependencyHealth, MonitorDependencyHealthSet } from "@/lib/api/types-monitoring";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

function dependencyStatusClasses(status: MonitorDependencyHealth["status"]): string {
  switch (status) {
    case "available":
      return "border-emerald-200 bg-emerald-50/70 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/30 dark:text-emerald-300";
    case "degraded":
      return "border-amber-200 bg-amber-50/70 text-amber-700 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-300";
    default:
      return "border-slate-200 bg-slate-50 text-slate-600 dark:border-slate-700 dark:bg-slate-800/60 dark:text-slate-300";
  }
}

export function DependencyHealthSection({ dependencies, t }: { dependencies?: MonitorDependencyHealthSet; t: TFunction }) {
  if (!dependencies) return null;

  const formatObservedAt = (value: string): string => {
    if (!Number.isFinite(Date.parse(value))) return t("monitor.metric_unavailable", { defaultValue: "Unavailable" });
    return new Intl.DateTimeFormat(undefined, { dateStyle: "short", timeStyle: "short" }).format(new Date(value));
  };

  const items: [keyof MonitorDependencyHealthSet, string][] = [
    ["redis", t("monitor.dependency_redis", { defaultValue: "Redis" })],
    ["postgres", t("monitor.dependency_postgres", { defaultValue: "PostgreSQL" })],
    ["database_pool", t("monitor.dependency_database_pool", { defaultValue: "Database pool" })],
    ["outbox_sweeper", t("monitor.dependency_outbox_sweeper", { defaultValue: "Outbox sweeper" })],
    ["worker_process", t("monitor.dependency_worker_process", { defaultValue: "Worker process" })],
    ["cpu", t("monitor.dependency_cpu", { defaultValue: "CPU" })],
    ["memory", t("monitor.dependency_memory", { defaultValue: "Memory" })],
    ["disk", t("monitor.dependency_disk", { defaultValue: "Disk" })],
  ];

  return (
    <section aria-label={t("monitor.dependency_health", { defaultValue: "Dependency health" })} className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-6">
      <div className="mb-4 flex flex-wrap items-end justify-between gap-3">
        <div>
          <Typography className="text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" variant="h4">
            {t("monitor.dependency_health", { defaultValue: "Dependency health" })}
          </Typography>
          <Typography className="mt-1 text-xs text-slate-500 dark:text-slate-400" variant="p">
            {t("monitor.dependency_health_subtitle", { defaultValue: "Measured sources only; unsupported metrics remain unavailable." })}
          </Typography>
        </div>
      </div>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {items.map(([key, label]) => {
          const dependency = dependencies[key];
          const statusLabel = t(`monitor.source_status.${dependency.status}`, { defaultValue: t("monitor.unknown_status", { defaultValue: "Status unavailable" }) });
          const sourceLabel = dependency.source === "unavailable"
            ? t("monitor.unknown_source", { defaultValue: "Source unavailable" })
            : t(`monitor.sources.${dependency.source.toLowerCase().replace(/[^a-z0-9]+/g, "_")}`, { defaultValue: dependency.source });
          const measurement = dependency.pool_utilization_percent !== null && dependency.pool_utilization_percent !== undefined
            ? `${dependency.pool_utilization_percent.toFixed(1)}%`
            : dependency.latency_ms !== null && dependency.latency_ms !== undefined
              ? `${dependency.latency_ms.toFixed(1)} ms`
              : dependency.availability === true
                ? t("monitor.available", { defaultValue: "Available" })
                : t("monitor.metric_unavailable", { defaultValue: "Unavailable" });
          const observedAt = formatObservedAt(dependency.observed_at);
          return (
            <article className={cn("rounded-2xl border p-4", dependencyStatusClasses(dependency.status))} key={key}>
              <div className="flex items-center justify-between gap-2">
                <span className="text-xs font-bold uppercase tracking-wider">{label}</span>
                <span className="text-[10px] font-bold uppercase tracking-wider">{statusLabel}</span>
              </div>
              <div className="mt-3 text-xl font-black tabular-nums">{measurement}</div>
              {dependency.limitation ? <div className="mt-2 text-xs opacity-80">{dependency.source === "unavailable" ? t("monitor.limitation_unavailable", { defaultValue: "Measurement limitation details are unavailable." }) : t("monitor.limitation_details", { defaultValue: "Measurement limitation is provided by the monitoring source." })}</div> : null}
              <div className="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-[10px] opacity-70">
                <span>{sourceLabel}</span>
                <span>{t("monitor.observed_at", { defaultValue: "Observed at" })}: {observedAt}</span>
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}
