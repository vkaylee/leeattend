import { Link } from "@tanstack/react-router";
import { Server, Database, Cpu, HardDrive, CheckCircle, ExternalLink, Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { useMyPermissions } from "@/hooks/use-permissions";
import { ROUTES } from "@/lib/routes";

interface ServiceItem {
  icon: React.ElementType;
  id: string;
  latency: string;
  name: string;
  status: "healthy" | "degraded" | "down";
}

const SERVICES: ServiceItem[] = [
  { id: "api", name: "Core API (Go 1.26+)", icon: Server, status: "healthy", latency: "14ms" },
  { id: "db", name: "PostgreSQL 18", icon: Database, status: "healthy", latency: "4ms" },
  { id: "cache", name: "Redis In-Memory", icon: Zap, status: "healthy", latency: "1ms" },
  { id: "storage", name: "MinIO S3 Storage", icon: HardDrive, status: "healthy", latency: "8ms" },
  { id: "workers", name: "Async Job Workers", icon: Cpu, status: "healthy", latency: "Active" },
];

export function SystemHealthWidget() {
  const { t } = useTranslation();
  const { hasAnyPermission } = useMyPermissions();
  const canViewMonitor = hasAnyPermission("infra:monitor", "system:manage");
  return (
    <div className="flex flex-col justify-between h-full space-y-4">
      {/* Top Overview Banner */}
      <div className="flex items-center justify-between p-3 rounded-2xl bg-emerald-50/60 dark:bg-emerald-950/20 border border-emerald-100 dark:border-emerald-900/40">
        <div className="flex items-center gap-2.5">
          <span className="relative flex h-3 w-3">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
            <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500" />
          </span>
          <span className="text-xs font-bold text-emerald-800 dark:text-emerald-300">
            {t("desktop.systemNormal", "Tất cả dịch vụ hoạt động bình thường")}
          </span>
        </div>
        <span className="text-[11px] font-semibold text-emerald-600 dark:text-emerald-400 bg-emerald-100/60 dark:bg-emerald-900/40 px-2 py-0.5 rounded-full">
          99.98%
        </span>
      </div>

      {/* Services Grid */}
      <div className="space-y-2">
        {SERVICES.map((srv) => {
          const Icon = srv.icon;
          return (
            <div
              className="flex items-center justify-between p-2.5 rounded-xl bg-slate-50/60 dark:bg-slate-900/40 border border-slate-100 dark:border-slate-800/60 text-xs"
              key={srv.id}
            >
              <div className="flex items-center gap-2.5">
                <div className="w-6 h-6 rounded-lg bg-white dark:bg-slate-800 flex items-center justify-center text-slate-500 shadow-2xs">
                  <Icon className="w-3.5 h-3.5 text-primary-500" />
                </div>
                <span className="font-medium text-slate-700 dark:text-slate-300">{srv.name}</span>
              </div>

              <div className="flex items-center gap-2 text-[11px]">
                <span className="text-slate-400 font-mono">{srv.latency}</span>
                <CheckCircle className="w-3.5 h-3.5 text-emerald-500" />
              </div>
            </div>
          );
        })}
      </div>

      {/* Footer Details */}
      {canViewMonitor ? (
        <div className="pt-2 border-t border-slate-100 dark:border-slate-800 flex items-center justify-between text-xs text-slate-400">
          <span>LeeAttend v1.26 Enterprise</span>
          <Link
            className="flex items-center gap-1 text-primary-500 hover:text-primary-600 font-medium"
            to={ROUTES.INFRA_MONITOR}
          >
            <span>{t("desktop.openMonitor", "Chi tiết giám sát")}</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </Link>
        </div>
      ) : null}
    </div>
  );
}
