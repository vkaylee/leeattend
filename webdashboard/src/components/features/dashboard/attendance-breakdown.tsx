import { Users2 } from "lucide-react";

import { type TFunction } from "i18next";
import { PieChart, Pie, ResponsiveContainer, Tooltip } from "recharts";

import { Typography } from "@/components/ui/typography";
import { type OrganizationStats } from "@/hooks/use-dashboard";

interface DonutData {
  fill: string;
  name: string;
  value: number;
}

interface AttendanceBreakdownProps {
  donutData: DonutData[];
  hasDonutData: boolean;
  hrStats?: OrganizationStats;
  t: TFunction;
}

function rateLabel(rate: OrganizationStats["attendance_rate"] | undefined, fallback: string, t: TFunction): string {
  if (rate?.value == null) return "—";
  return t("dashboard.hr.rate_value", { value: rate.value });
}

export function AttendanceBreakdown({ donutData, hasDonutData, hrStats, t }: AttendanceBreakdownProps) {
  const summary = t("dashboard.hr.attendance_breakdown_summary", {
    present: hrStats?.present_today ?? 0,
    late: hrStats?.late_today ?? 0,
    absent: hrStats?.absent_today ?? 0,
    onLeave: hrStats?.on_leave_today ?? 0,
  });

  return (
    <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] p-6 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
      <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-8" variant="h2">
        {t("dashboard.hr.attendance_breakdown")}
      </Typography>

      {hrStats ? (
        <div aria-label={t("dashboard.hr.rate_summary_label")} className="flex flex-wrap items-center justify-center gap-x-6 gap-y-2 mb-4">
          <RateBadge label={t("dashboard.hr.attendance_rate")} tone="emerald" value={rateLabel(hrStats.attendance_rate, "—", t)} />
          <RateBadge label={t("dashboard.hr.absence_rate")} tone="rose" value={rateLabel(hrStats.absence_rate, "—", t)} />
          <RateBadge label={t("dashboard.hr.late_rate")} tone="amber" value={rateLabel(hrStats.late_rate, "—", t)} />
        </div>
      ) : null}

      {hasDonutData ? (
        <div className="flex flex-col items-center">
          <div aria-label={summary} role="img">
            <ResponsiveContainer height={260} width="100%">
              <PieChart>
                <Pie
                  cx="50%"
                  cy="50%"
                  data={donutData}
                  dataKey="value"
                  innerRadius={70}
                  outerRadius={110}
                  paddingAngle={3}
                  stroke="none"
                />
                <Tooltip
                  contentStyle={{
                    borderRadius: "1rem",
                    border: "none",
                    boxShadow: "0 8px 30px rgba(0,0,0,.1)",
                    fontSize: "12px",
                    fontWeight: 600,
                  }}
                />
              </PieChart>
            </ResponsiveContainer>
          </div>
          <div className="flex flex-wrap justify-center gap-4 mt-4">
            {donutData.map((entry, index) => (
              <div className="flex items-center gap-2" key={index}>
                <div className="w-3 h-3 rounded-full" style={{ backgroundColor: entry.fill }} />
                <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {entry.name}: {entry.value}
                </span>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center h-[260px]">
          <Users2 className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
          <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
            {t("common.no_data")}
          </Typography>
        </div>
      )}
    </div>
  );
}

function RateBadge({ label, value, tone }: { label: string; value: string; tone: "emerald" | "rose" | "amber" }) {
  const tones: Record<typeof tone, string> = {
    emerald: "text-emerald-600 dark:text-emerald-400",
    rose: "text-rose-500 dark:text-rose-400",
    amber: "text-amber-600 dark:text-amber-400",
  };
  return (
    <span className="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">
      {label}: <span className={tones[tone]}>{value}</span>
    </span>
  );
}
