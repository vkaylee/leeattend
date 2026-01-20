import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useMyAttendance } from "@/hooks/use-ess";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { startOfMonth, endOfMonth, format, subMonths, addMonths } from "date-fns";
import { 
  Calendar, 
  ChevronLeft, 
  ChevronRight, 
  Clock, 
  Info, 
  AlertCircle,
  ChevronDown
} from "lucide-react";
import { cn } from "@/lib/utils";
import * as Popover from "@radix-ui/react-popover";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";

export default function MyAttendancePage() {
  const { t } = useTranslation();
  const { formatTime, formatDate } = useDateTimeFormatter();
  const [currentMonth, setCurrentMonth] = useState(new Date());

  const { entries, isLoading } = useMyAttendance({
    start_date: format(startOfMonth(currentMonth), "yyyy-MM-dd"),
    end_date: format(endOfMonth(currentMonth), "yyyy-MM-dd"),
  });

  const handleMonthYearChange = (month: number, year: number) => {
    setCurrentMonth(new Date(year, month - 1, 1));
  };

  const handlePrevMonth = () => setCurrentMonth(subMonths(currentMonth, 1));
  const handleNextMonth = () => setCurrentMonth(addMonths(currentMonth, 1));

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-black text-slate-900 tracking-tight">
            {t("ess.attendance.title")}
          </h1>
          <p className="text-slate-500 font-medium">
            {t("ess.attendance.subtitle")}
          </p>
        </div>

        <div className="flex items-center gap-1 bg-white p-1 rounded-2xl border border-slate-200 shadow-sm">
          <button 
            onClick={handlePrevMonth}
            className="p-2 hover:bg-slate-50 rounded-xl transition-colors text-slate-500"
          >
            <ChevronLeft size={20} />
          </button>
          
          <Popover.Root>
            <Popover.Trigger asChild>
              <button className="px-4 py-2 font-bold text-slate-700 hover:bg-slate-50 rounded-xl transition-colors flex items-center gap-2 min-w-[140px] justify-center">
                {format(currentMonth, "MMMM yyyy")}
                <ChevronDown size={14} className="text-slate-400" />
              </button>
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content 
                className="z-50 animate-in zoom-in-95 slide-in-from-top-2"
                sideOffset={5}
                align="center"
              >
                <MonthYearPicker 
                  month={currentMonth.getMonth() + 1}
                  year={currentMonth.getFullYear()}
                  onChange={handleMonthYearChange}
                />
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>

          <button 
            onClick={handleNextMonth}
            className="p-2 hover:bg-slate-50 rounded-xl transition-colors text-slate-500"
          >
            <ChevronRight size={20} />
          </button>
        </div>
      </div>
      {/* ... rest of the file remains same */}

      <div className="bg-white border border-slate-200 rounded-[32px] overflow-hidden shadow-sm">
        {isLoading ? (
          <div className="p-12 flex justify-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
          </div>
        ) : entries.length === 0 ? (
          <div className="p-12 text-center flex flex-col items-center">
            <div className="w-16 h-16 bg-slate-50 rounded-full flex items-center justify-center mb-4">
              <Calendar className="w-8 h-8 text-slate-300" />
            </div>
            <h3 className="text-lg font-bold text-slate-900">{t("ess.attendance.no_data")}</h3>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-slate-100">
                  <th className="p-6 text-xs font-black text-slate-400 uppercase tracking-widest">
                    {t("ess.attendance.date")}
                  </th>
                  <th className="p-6 text-xs font-black text-slate-400 uppercase tracking-widest">
                    {t("ess.attendance.check_in")}
                  </th>
                  <th className="p-6 text-xs font-black text-slate-400 uppercase tracking-widest">
                    {t("ess.attendance.check_out")}
                  </th>
                  <th className="p-6 text-xs font-black text-slate-400 uppercase tracking-widest">
                    {t("ess.attendance.duration")}
                  </th>
                  <th className="p-6 text-xs font-black text-slate-400 uppercase tracking-widest text-right">
                    {t("common.status")}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {entries.map((entry) => (
                  <tr key={entry.id} className="hover:bg-slate-50/50 transition-colors group">
                    <td className="p-6">
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-xl bg-blue-50 text-blue-600 flex items-center justify-center font-black text-sm">
                          {format(new Date(entry.punch_in_time), "dd")}
                        </div>
                        <div>
                          <div className="font-bold text-slate-900">
                            {format(new Date(entry.punch_in_time), "EEEE")}
                          </div>
                          <div className="text-xs text-slate-400 font-medium">
                            {formatDate(entry.punch_in_time)}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="p-6">
                      <div className="flex items-center gap-2 font-bold text-slate-700">
                        <Clock size={16} className="text-emerald-500" />
                        {formatTime(entry.punch_in_time)}
                      </div>
                    </td>
                    <td className="p-6">
                      {entry.punch_out_time ? (
                        <div className="flex items-center gap-2 font-bold text-slate-700">
                          <Clock size={16} className="text-rose-500" />
                          {formatTime(entry.punch_out_time)}
                        </div>
                      ) : (
                        <span className="text-slate-400 italic text-sm">--:--</span>
                      )}
                    </td>
                    <td className="p-6">
                      {entry.duration_minutes ? (
                        <span className="inline-flex items-center px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 text-xs font-bold">
                          {Math.floor(entry.duration_minutes / 60)}h {entry.duration_minutes % 60}m
                        </span>
                      ) : (
                        <span className="text-slate-400">-</span>
                      )}
                    </td>
                    <td className="p-6 text-right">
                      <div className="flex items-center justify-end gap-2">
                        {/* Status Badge */}
                        <span className={cn(
                          "px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest border",
                          entry.punch_out_time 
                            ? "bg-emerald-50 text-emerald-600 border-emerald-100" 
                            : "bg-amber-50 text-amber-600 border-amber-100"
                        )}>
                          {entry.punch_out_time ? t("common.completed") : t("common.pending")}
                        </span>

                        {/* Metadata Popover for Traceability */}
                        {entry.generation_metadata && (
                          <Popover.Root>
                            <Popover.Trigger asChild>
                              <button className="p-2 text-slate-300 hover:text-blue-500 transition-colors">
                                <Info size={16} />
                              </button>
                            </Popover.Trigger>
                            <Popover.Portal>
                              <Popover.Content 
                                className="z-50 w-72 bg-white rounded-xl shadow-xl border border-slate-100 p-4 animate-in fade-in zoom-in-95"
                                side="left"
                                align="center"
                              >
                                <div className="space-y-3">
                                  <div className="flex items-center gap-2 border-b border-slate-50 pb-2">
                                    <AlertCircle size={14} className="text-blue-500" />
                                    <span className="text-xs font-black uppercase text-slate-700">
                                      {t("ess.attendance.generation_info")}
                                    </span>
                                  </div>
                                  <div className="bg-slate-50 p-2 rounded-lg border border-slate-100">
                                    <pre className="text-[10px] text-slate-600 overflow-x-auto whitespace-pre-wrap font-mono">
                                      {JSON.stringify(entry.generation_metadata, null, 2)}
                                    </pre>
                                  </div>
                                </div>
                                <Popover.Arrow className="fill-white" />
                              </Popover.Content>
                            </Popover.Portal>
                          </Popover.Root>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
