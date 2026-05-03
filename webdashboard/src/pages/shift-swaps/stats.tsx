import React from "react";

import { TFunction } from "i18next";

import { cn } from "@/lib/utils";

interface ShiftSwapStatsProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  stats: {
    approved: number;
    pending_admin: number;
    pending_target: number;
    rejected: number;
  };
  t: TFunction;
}

export function ShiftSwapStats({
  activeTab,
  onTabChange,
  stats,
  t,
}: ShiftSwapStatsProps) {
  const statItems = [
    {
      color: "amber",
      key: "pending_admin",
      label: t("shift_swaps.status.pending_admin"),
      value: stats.pending_admin,
    },
    {
      color: "blue",
      key: "pending_target",
      label: t("shift_swaps.status.pending_target"),
      value: stats.pending_target,
      value_display: stats.pending_target,
    },
    {
      color: "emerald",
      key: "approved",
      label: t("shift_swaps.status.approved"),
      value: stats.approved,
    },
    {
      color: "rose",
      key: "rejected",
      label: t("shift_swaps.status.rejected"),
      value: stats.rejected,
    },
  ];

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 animate-in slide-in-from-bottom-4 duration-500 delay-100">
      {statItems.map((stat) => (
        <StatCard
          activeTab={activeTab}
          color={stat.color}
          key={stat.key}
          label={stat.label}
          onClick={() => {
            onTabChange(stat.key);
          }}
          statKey={stat.key}
          value={stat.value}
        />
      ))}
    </div>
  );
}

function StatCard({
  activeTab,
  color,
  label,
  onClick,
  statKey,
  value,
}: {
  activeTab: string;
  color: string;
  label: string;
  onClick: () => void;
  statKey: string;
  value: number;
}) {
  const isActive = activeTab === statKey;
  const colorBase = color === "blue" ? "primary" : color;

  return (
    <div
      className={cn(
        "bg-white dark:bg-slate-900 p-5 rounded-[2rem] border shadow-sm cursor-pointer hover:shadow-md transition-all group",
        isActive
          ? `border-${colorBase}-300 dark:border-${colorBase}-700 ring-2 ring-${colorBase}-100 dark:ring-${colorBase}-900`
          : "border-slate-100 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700"
      )}
      onClick={() => {
        onClick();
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick();
        }
      }}
      role="button"
      tabIndex={0}
    >
      <div className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-2 group-hover:text-slate-600 dark:group-hover:text-slate-300 transition-colors">
        {label}
      </div>
      <div
        className={cn(
          "text-3xl font-black tracking-tighter font-mono",
          `text-${colorBase}-600 dark:text-${colorBase}-400`
        )}
      >
        {value}
      </div>
    </div>
  );
}
