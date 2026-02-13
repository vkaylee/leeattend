import { FileBarChart, Users2 } from "lucide-react";
import { formatUnitsToHours } from "@/lib/utils/attendance";
import { AttendanceSummary, DailyAttendanceReport } from "@/lib/api";

interface ReportStatsProps {
  reportType: "summary" | "daily";
  filteredData: AttendanceSummary[] | DailyAttendanceReport[];
  workUnitMinutes: number;
  t: (key: string) => string;
}

export default function ReportStats({
  reportType,
  filteredData,
  workUnitMinutes,
  t,
}: ReportStatsProps) {
  const totalHours = formatUnitsToHours(
    reportType === "summary"
      ? filteredData.reduce((acc: number, r: any) => acc + r.total_work_units, 0)
      : filteredData.reduce((acc: number, r: any) => acc + r.std_work_units + r.ot_work_units, 0),
    workUnitMinutes
  );

  const totalEmployees = new Set(filteredData.map((r: any) => r.employee_id)).size;

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 px-2">
      <div className="bg-slate-900 p-10 rounded-[3rem] border border-slate-800 shadow-2xl shadow-slate-900/20 overflow-hidden relative group hover:scale-[1.02] transition-all duration-500">
        <div className="bg-primary-500/20 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 text-primary-400 shadow-inner border border-white/5 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">
          <FileBarChart size={32} />
        </div>
        <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] mb-2 opacity-70">
          {t("reports.quick_stats.total_hours")}
        </p>
        <p className="text-5xl font-bold text-white tracking-tighter">
          {totalHours}{" "}
          <span className="text-xl opacity-30 font-bold uppercase tracking-widest ml-1">HRS</span>
        </p>
        <FileBarChart className="absolute -right-8 -bottom-8 w-40 h-44 text-white opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
      </div>

      <div className="bg-white p-10 rounded-[3rem] border border-slate-100 shadow-xl shadow-slate-900/5 overflow-hidden relative group hover:scale-[1.02] transition-all duration-500">
        <div className="bg-primary-50 w-16 h-16 rounded-[1.5rem] flex items-center justify-center mb-8 text-primary-600 shadow-inner border border-primary-100 group-hover:scale-110 group-hover:rotate-3 transition-transform duration-500">
          <Users2 size={32} />
        </div>
        <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] mb-2 opacity-70">
          {t("common.employees")}
        </p>
        <p className="text-5xl font-bold text-slate-700 tracking-tighter">
          {totalEmployees}{" "}
          <span className="text-xl opacity-20 font-bold uppercase tracking-widest ml-1">STAFF</span>
        </p>
        <Users2 className="absolute -right-8 -bottom-8 w-40 h-44 text-slate-100 opacity-20 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
      </div>
    </div>
  );
}
