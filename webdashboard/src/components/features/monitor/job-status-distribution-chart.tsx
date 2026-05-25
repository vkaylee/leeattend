import { useTranslation } from "react-i18next";

import { PieChart, Pie, Cell, Tooltip } from "recharts";

import type { JobStatusCount } from "@/hooks/use-monitor-charts";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface Props {
  data: JobStatusCount[];
  isLoading: boolean;
}

export default function JobStatusDistributionChart({ data, isLoading }: Props) {
  const { t } = useTranslation();

  return (
    <div className="bg-white/70 dark:bg-slate-900/70 backdrop-blur-xl border border-slate-200/60 dark:border-slate-700/60 rounded-[2rem] p-6 shadow-lg dark:shadow-black/20 flex flex-col items-center gap-4 relative overflow-hidden group">
      {/* Hover glow */}
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-40 h-40 rounded-full bg-gradient-to-br from-blue-400/15 to-blue-400/10 blur-2xl" />
      </div>

      <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] w-full text-center" variant="h5">
        {t("monitor.job_distribution")}
      </Typography>

      {isLoading ? (
        <div className="w-[160px] h-[160px] flex items-center justify-center">
          <div className="w-20 h-20 rounded-full border-4 border-slate-100 dark:border-slate-700 border-t-blue-400 animate-spin" />
        </div>
      ) : data.length === 0 ? (
        <div className="h-[160px] flex flex-col items-center justify-center gap-2">
          <div className="w-12 h-12 rounded-2xl bg-slate-50 dark:bg-slate-800/50 flex items-center justify-center">
            <span className="text-xl">📊</span>
          </div>
          <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
            {t("monitor.no_data", { defaultValue: "No data" })}
          </Typography>
        </div>
      ) : (
        <PieChart height={160} width={160}>
          <Pie
            cornerRadius={4}
            cx="50%"
            cy="50%"
            data={data}
            dataKey="count"
            innerRadius={45}
            nameKey="status"
            outerRadius={70}
            paddingAngle={3}
            strokeWidth={0}
          >
            {data.map((entry, index) => {
              // eslint-disable-next-line @typescript-eslint/no-deprecated
              return <Cell fill={entry.color} key={index} />;
            })}
          </Pie>
          <Tooltip
            contentStyle={{
              borderRadius: "1rem",
              border: "1px solid rgba(148,163,184,0.2)",
              background: "rgba(255,255,255,0.95)",
              backdropFilter: "blur(8px)",
              fontSize: "12px",
              fontWeight: 600,
            }}
          />
        </PieChart>
      )}

      {/* Percentage legend pills */}
      <div className="flex flex-wrap items-center justify-center gap-2">
        {data.map((entry) => (
          <span
            className={cn(
              "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider",
              "bg-slate-50 dark:bg-slate-800/60 text-slate-600 dark:text-slate-300 border border-slate-100 dark:border-slate-700/50",
              "hover:scale-105 transition-transform duration-200 cursor-default"
            )}
            data-testid={`legend-${entry.status}`}
            key={entry.status}
          >
            <span
              className="w-2 h-2 rounded-full flex-shrink-0"
              style={{ backgroundColor: entry.color }}
            />
            {entry.status} {entry.percent}%
          </span>
        ))}
      </div>
    </div>
  );
}
