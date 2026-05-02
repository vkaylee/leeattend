import { FileBarChart, Users2, Clock } from "lucide-react";

import { AttendanceSummary, DailyAttendanceReport } from "@/lib/api";
import { formatUnitsToHours } from "@/lib/utils/attendance";

interface ReportStatsProps {
  filteredData: AttendanceSummary[] | DailyAttendanceReport[];
  reportType: "summary" | "daily";
  t: (key: string) => string;
  workUnitMinutes: number;
}

export default function ReportStats({
  reportType,
  filteredData,
  workUnitMinutes,
  t,
}: ReportStatsProps) {
  const stdHours = formatUnitsToHours(
    reportType === "summary"
 
      ? (filteredData as AttendanceSummary[]).reduce((accumulator: number, r: AttendanceSummary) => accumulator + (r.total_std_units || 0), 0)
 
      : (filteredData as DailyAttendanceReport[]).reduce((accumulator: number, r: DailyAttendanceReport) => accumulator + r.std_work_units, 0),
    workUnitMinutes
  );

  const otHours = formatUnitsToHours(
    reportType === "summary"
 
      ? (filteredData as AttendanceSummary[]).reduce((accumulator: number, r: AttendanceSummary) => accumulator + (r.total_ot_units || 0), 0)
 
      : (filteredData as DailyAttendanceReport[]).reduce((accumulator: number, r: DailyAttendanceReport) => accumulator + (r.ot_work_units || 0), 0),
    workUnitMinutes
  );

 
  const totalEmployees = new Set(filteredData.map((r) => r.employee_id)).size;

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 px-2">
      <div className="bg-slate-900 p-10 rounded-[3rem] border border-slate-800 shadow-2xl shadow-slate-900/20 overflow-hidden relative group hover:scale-[1.02] transition-all duration-500">
        <div className="bg-primary-50 dark:bg-primary-950/20 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 text-primary-400 shadow-inner border border-white/5 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">
          <FileBarChart size={32} />
        </div>
        <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-2 opacity-70">
          {t("reports.quick_stats.std_hours")}
        </p>
        <p className="text-5xl font-bold text-white tracking-tighter">
          {stdHours}{" "}
          <span className="text-xl opacity-30 font-bold uppercase tracking-widest ml-1">HRS</span>
        </p>
        <FileBarChart className="absolute -right-8 -bottom-8 w-40 h-44 text-white opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
      </div>

      <div className="bg-gradient-to-br from-orange-900 to-orange-950 p-10 rounded-[3rem] border border-orange-800 shadow-2xl shadow-orange-900/20 overflow-hidden relative group hover:scale-[1.02] transition-all duration-500">
        <div className="bg-orange-800/30 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 text-orange-400 shadow-inner border border-orange-700/30 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">
          <Clock size={32} />
        </div>
        <p className="text-[10px] font-bold text-orange-400/70 uppercase tracking-[0.25em] mb-2">
          {t("reports.quick_stats.ot_hours")}
        </p>
        <p className="text-5xl font-bold text-orange-100 tracking-tighter">
          {otHours}{" "}
          <span className="text-xl opacity-30 font-bold uppercase tracking-widest ml-1">OT</span>
        </p>
        <Clock className="absolute -right-8 -bottom-8 w-40 h-44 text-orange-400 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
      </div>

      <div className="bg-white dark:bg-slate-900 p-10 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-xl shadow-slate-900/5 overflow-hidden relative group hover:scale-[1.02] transition-all duration-500">
        <div className="bg-primary-50 dark:bg-primary-950 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">
          <Users2 size={32} />
        </div>
        <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-2 opacity-70">
          {t("common.employees")}
        </p>
        <p className="text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">
          {totalEmployees}{" "}
          <span className="text-xl opacity-20 font-bold uppercase tracking-widest ml-1">STAFF</span>
        </p>
        <Users2 className="absolute -right-8 -bottom-8 w-40 h-44 text-slate-100 opacity-20 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
      </div>
    </div>
  );
}
