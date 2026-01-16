import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { FileBarChart, Download, Calendar, Search, List, TableProperties, Clock, User as UserIcon, CalendarDays } from "lucide-react";
import { format } from "date-fns";
import { 
  fetchAttendanceReport, 
  getCurrentWorkspaceId,
  AttendanceSummary,
  DailyAttendanceReport
} from "../lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

export default function ReportsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const { formatDate, formatTime } = useDateTimeFormatter();
  
  // Helper to get month range
  const getMonthRange = (offset: number = 0) => {
    const d = new Date();
    d.setMonth(d.getMonth() + offset);
    const firstDay = format(new Date(d.getFullYear(), d.getMonth(), 1), 'yyyy-MM-dd');
    const lastDay = format(new Date(d.getFullYear(), d.getMonth() + 1, 0), 'yyyy-MM-dd');
    return { start: firstDay, end: lastDay };
  };

  const [filterMode, setFilterMode] = useState<"this_month" | "last_month" | "custom">("this_month");
  const [dateRange, setDateRange] = useState(getMonthRange(0));
  const [searchTerm, setSearchTerm] = useState("");
  const [reportType, setReportType] = useState<"summary" | "daily">("summary");

  const handleFilterModeChange = (mode: "this_month" | "last_month" | "custom") => {
    setFilterMode(mode);
    if (mode === "this_month") {
      setDateRange(getMonthRange(0));
    } else if (mode === "last_month") {
      setDateRange(getMonthRange(-1));
    }
  };

  const { data: reportData = [], isLoading } = useQuery({
    queryKey: ["attendance-report", wsId, dateRange, reportType],
    queryFn: () => fetchAttendanceReport(wsId!, dateRange.start, dateRange.end, reportType),
    enabled: !!wsId,
  });

  const filteredData = Array.isArray(reportData) ? reportData.filter((row: any) => 
    row.employee_name.toLowerCase().includes(searchTerm.toLowerCase())
  ) : [];

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  const getStatusBadge = (status: string) => {
    const styles: Record<string, string> = {
      present: "bg-emerald-100 text-emerald-700",
      late: "bg-amber-100 text-amber-700",
      absent: "bg-rose-100 text-rose-700",
      off: "bg-slate-100 text-slate-600",
    };
    const labels: Record<string, string> = {
      present: t("reports.status.present"),
      late: t("reports.status.late"),
      absent: t("reports.status.absent"),
      off: t("reports.status.off"),
    };
    return (
      <span className={`px-2 py-1 rounded-full text-[10px] font-black uppercase tracking-wider ${styles[status] || styles.off}`}>
        {labels[status] || status}
      </span>
    );
  };

  return (
    <div className="space-y-6 pb-20">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t("reports.title")}</h1>
          <p className="text-slate-500">{t("reports.subtitle")}</p>
        </div>
        <button 
          className="flex items-center gap-2 border border-slate-200 bg-white px-4 py-2 rounded-xl hover:bg-slate-50 transition-colors shadow-sm font-semibold"
          onClick={() => alert(t("common.feature_coming_soon"))}
        >
          <Download className="w-5 h-5 text-slate-600" />
          <span>{t("reports.export_excel")}</span>
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 bg-slate-100 p-1 rounded-2xl w-fit">
        <button
          onClick={() => setReportType("summary")}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold transition-all ${
            reportType === "summary" ? "bg-white shadow-sm text-slate-900" : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <List className="w-4 h-4" />
          {t("reports.tabs.summary")}
        </button>
        <button
          onClick={() => setReportType("daily")}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold transition-all ${
            reportType === "daily" ? "bg-white shadow-sm text-slate-900" : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <TableProperties className="w-4 h-4" />
          {t("reports.tabs.daily")}
        </button>
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl p-6 shadow-sm space-y-6">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="flex items-center gap-2 bg-slate-100 p-1 rounded-2xl">
            <button
              onClick={() => handleFilterModeChange("this_month")}
              className={`px-4 py-2 rounded-xl text-sm font-bold transition-all ${
                filterMode === "this_month" ? "bg-white shadow-sm text-primary-600" : "text-slate-500 hover:text-slate-700"
              }`}
            >
              {t("common.filters.this_month")}
            </button>
            <button
              onClick={() => handleFilterModeChange("last_month")}
              className={`px-4 py-2 rounded-xl text-sm font-bold transition-all ${
                filterMode === "last_month" ? "bg-white shadow-sm text-primary-600" : "text-slate-500 hover:text-slate-700"
              }`}
            >
              {t("common.filters.last_month")}
            </button>
            <button
              onClick={() => handleFilterModeChange("custom")}
              className={`px-4 py-2 rounded-xl text-sm font-bold transition-all ${
                filterMode === "custom" ? "bg-white shadow-sm text-primary-600" : "text-slate-500 hover:text-slate-700"
              }`}
            >
              {t("common.filters.custom")}
            </button>
          </div>

          <div className="flex-1 max-w-md relative">
            <input 
              type="text" 
              placeholder={t("reports.search_placeholder")}
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-indigo-500/20 font-medium"
            />
            <Search className="absolute left-3.5 top-2.5 w-4 h-4 text-slate-400" />
          </div>
        </div>

        {filterMode === "custom" && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-4 border-t border-slate-100 animate-in slide-in-from-top-2 duration-300">
            <div className="space-y-1">
              <label htmlFor="start-date" className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
                <Calendar className="w-3 h-3" /> {t("common.from_date")}
              </label>
              <input 
                id="start-date"
                type="date" 
                value={dateRange.start}
                onChange={(e) => setDateRange(prev => ({ ...prev, start: e.target.value }))}
                className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-indigo-500/20 font-mono"
              />
            </div>
            <div className="space-y-1">
              <label htmlFor="end-date" className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2">
                <Calendar className="w-3 h-3" /> {t("common.to_date")}
              </label>
              <input 
                id="end-date"
                type="date" 
                value={dateRange.end}
                onChange={(e) => setDateRange(prev => ({ ...prev, end: e.target.value }))}
                className="w-full px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl outline-none focus:ring-2 focus:ring-indigo-500/20 font-mono"
              />
            </div>
          </div>
        )}
      </div>

      <div className="bg-white border border-slate-200 rounded-3xl overflow-hidden shadow-sm">
        {reportType === "summary" ? (
          <table className="w-full text-left">
            <thead className="bg-slate-50 border-b border-slate-200">
              <tr>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("common.employee")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">{t("reports.table.total_hours")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">{t("reports.table.leave_days")}</th>
                <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">{t("common.notes")}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {isLoading ? (
                <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-500">{t("reports.messages.generating")}</td></tr>
              ) : filteredData.length === 0 ? (
                <tr><td colSpan={4} className="px-6 py-8 text-center text-slate-500 italic">{t("reports.messages.no_data")}</td></tr>
              ) : filteredData.map((row: AttendanceSummary) => (
                <tr key={row.employee_id} className="hover:bg-slate-50/50 transition-colors">
                  <td className="px-6 py-4 font-medium text-slate-900">{row.employee_name}</td>
                  <td className="px-6 py-4 text-right font-mono text-primary-600 font-bold">
                    {row.total_hours.toFixed(1)}h
                  </td>
                  <td className="px-6 py-4 text-right font-mono text-orange-600 font-bold">
                    {row.total_leave_days} {t("common.days")}
                  </td>
                  <td className="px-6 py-4 text-right text-sm text-slate-400 italic">
                    -
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left min-w-[800px]">
              <thead className="bg-slate-50 border-b border-slate-200">
                <tr>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap">{t("common.date")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600">{t("common.employee")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap">{t("common.shift")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap text-center">Punch In</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 whitespace-nowrap text-center">Punch Out</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-right">{t("reports.table.hours")}</th>
                  <th className="px-6 py-4 text-sm font-semibold text-slate-600 text-center">{t("common.status")}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {isLoading ? (
                  <tr><td colSpan={7} className="px-6 py-8 text-center text-slate-500">{t("common.loading")}</td></tr>
                ) : filteredData.length === 0 ? (
                  <tr><td colSpan={7} className="px-6 py-8 text-center text-slate-500 italic">{t("reports.messages.no_data")}</td></tr>
                ) : filteredData.map((row: DailyAttendanceReport, idx: number) => (
                  <tr key={`${row.employee_id}-${row.date}-${idx}`} className="hover:bg-slate-50/50 transition-colors">
                    <td className="px-6 py-4 text-sm font-medium text-slate-600 whitespace-nowrap font-mono uppercase">
                      {formatDate(row.date)}
                    </td>
                    <td className="px-6 py-4 font-bold text-slate-900">{row.employee_name}</td>
                    <td className="px-6 py-4 text-sm text-slate-500 whitespace-nowrap">
                      {row.scheduled_start ? (
                        <div className="flex flex-col">
                          <span className="font-semibold text-slate-700">{row.scheduled_start.substring(0, 5)} - {row.scheduled_end?.substring(0, 5)}</span>
                          {row.break_duration_minutes !== null && (
                            <span className="text-[10px] text-slate-400">
                              {row.break_start ? (
                                `${t("common.break")}: ${row.break_start.substring(0, 5)} - ${row.break_end?.substring(0, 5)}`
                              ) : (
                                `${t("common.break")}: ${row.break_duration_minutes}m`
                              )}
                            </span>
                          )}
                        </div>
                      ) : <span className="text-slate-400">{t("reports.no_shift")}</span>}
                    </td>
                    <td className="px-6 py-4 text-center">
                      <div className="flex flex-col items-center">
                        <span className={`font-mono font-bold ${row.actual_punch_in ? "text-slate-900" : "text-slate-300"}`}>
                          {formatTime(row.actual_punch_in)}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-center">
                      <div className="flex flex-col items-center">
                        <span className={`font-mono font-bold ${row.actual_punch_out ? "text-slate-900" : "text-slate-300"}`}>
                          {formatTime(row.actual_punch_out)}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-right font-mono text-primary-600 font-black">
                      {(row.total_minutes / 60).toFixed(1)}h
                    </td>
                    <td className="px-6 py-4 text-center">
                      {getStatusBadge(row.status)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-slate-900 rounded-3xl p-6 border border-slate-800 shadow-xl shadow-slate-200">
          <h3 className="text-white font-bold mb-2 flex items-center gap-2">
            <FileBarChart className="w-5 h-5 text-blue-400" /> {t("reports.quick_stats.title")}
          </h3>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="bg-slate-800/50 p-4 rounded-2xl">
              <p className="text-slate-400 text-xs font-black uppercase tracking-widest">{t("common.employees")}</p>
              <p className="text-2xl font-bold text-white mt-1">{new Set(filteredData.map((r: any) => r.employee_id)).size}</p>
            </div>
            <div className="bg-slate-800/50 p-4 rounded-2xl">
              <p className="text-slate-400 text-xs font-black uppercase tracking-widest">{t("reports.quick_stats.total_hours")}</p>
              <p className="text-2xl font-bold text-blue-400 mt-1">
                {reportType === "summary" 
                  ? filteredData.reduce((acc: number, r: any) => acc + r.total_hours, 0).toFixed(1)
                  : (filteredData.reduce((acc: number, r: any) => acc + r.total_minutes, 0) / 60).toFixed(1)}h
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
