import { Users2 } from "lucide-react";

import { type TFunction } from "i18next";
import { PieChart, Pie, ResponsiveContainer, Tooltip } from "recharts";

import { Typography } from "@/components/ui/typography";

interface DonutData {
  fill: string;
  name: string;
  value: number;
}

interface AttendanceBreakdownProps {
  donutData: DonutData[];
  hasDonutData: boolean;
  t: TFunction;
}

export function AttendanceBreakdown({ donutData, hasDonutData, t }: AttendanceBreakdownProps) {
  return (
    <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-8 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
      <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-8" variant="h2">
        {t("dashboard.hr.attendance_breakdown")}
      </Typography>
      {hasDonutData ? (
        <div className="flex flex-col items-center">
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
