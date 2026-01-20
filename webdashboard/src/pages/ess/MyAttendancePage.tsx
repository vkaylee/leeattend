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

  const handleMonthYearChange = (month: number, year: number) => { setCurrentMonth(new Date(year, month - 1, 1)); };
  const handlePrevMonth = () => setCurrentMonth(subMonths(currentMonth, 1));
  const handleNextMonth = () => setCurrentMonth(addMonths(currentMonth, 1));

  return (
    <div className="space-y-8 animate-in fade-in duration-500 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900">{t("ess.attendance.title")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("ess.attendance.subtitle")}</p>
        </div>

        <div className="flex items-center gap-4 bg-white p-2 rounded-[2rem] border border-slate-200 shadow-sm">
          <div className="flex items-center bg-slate-100/50 rounded-2xl p-1 shadow-inner border border-slate-200/50">
            <button onClick={handlePrevMonth} className="w-10 h-10 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-sm rounded-xl transition-all"><ChevronLeft size={20} /></button>
            
            <Popover.Root>
              <Popover.Trigger asChild>
                <button className="px-6 flex flex-col items-center min-w-[160px] cursor-pointer hover:bg-white hover:shadow-sm rounded-xl transition-all py-1.5 group outline-none border border-transparent hover:border-slate-100">
                  <span className="text-[9px] font-black text-slate-400 uppercase tracking-widest leading-none mb-1 group-hover:text-primary-500 transition-colors uppercase">{t("payroll.table.period")}</span>
                  <span className="text-sm font-black text-slate-900 group-hover:text-primary-600 transition-colors uppercase tracking-tighter">
                    {format(currentMonth, "MMMM yyyy")}
                  </span>
                </button>
              </Popover.Trigger>
              <Popover.Portal>
                <Popover.Content className="z-50 w-80 p-4 rounded-[2rem] shadow-2xl border border-slate-100 bg-white animate-in zoom-in-95 slide-in-from-top-2 origin-top" sideOffset={8} align="center">
                  <MonthYearPicker month={currentMonth.getMonth() + 1} year={currentMonth.getFullYear()} onChange={handleMonthYearChange} />
                </Popover.Content>
              </Popover.Portal>
            </Popover.Root>

            <button onClick={handleNextMonth} className="w-10 h-10 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-sm rounded-xl transition-all"><ChevronRight size={20} /></button>
          </div>
        </div>
      </div>

      <div className="bg-white border border-slate-100 rounded-[2.5rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-4 duration-500">
        {isLoading ? (
          <div className="p-20 flex flex-col items-center gap-4"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div><span className="text-sm font-black text-slate-400 uppercase tracking-widest">{t("common.loading")}</span></div>
        ) : entries.length === 0 ? (
          <div className="p-24 text-center flex flex-col items-center border-2 border-dashed border-slate-100 m-4 rounded-[2rem] bg-slate-50/30">
            <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4"><Calendar size={32} className="text-slate-200" /></div>
            <h3 className="text-lg font-bold text-slate-900">{t("ess.attendance.no_data")}</h3>
            <p className="text-slate-400 text-sm mt-1 uppercase font-black tracking-widest opacity-50">{t("common.no_records_found")}</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="bg-slate-50/50 border-b border-slate-100">
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("ess.attendance.date")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.in")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("common.out")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-center">{t("ess.attendance.duration")}</th>
                  <th className="px-8 py-4 text-[10px] font-black text-slate-400 uppercase tracking-widest text-right">{t("common.status")}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-50">
                {entries.map((entry) => (
                  <tr key={entry.id} className="hover:bg-slate-50/50 transition-colors group">
                    <td className="px-8 py-5">
                      <div className="flex items-center gap-4">
                        <div className="w-12 h-12 rounded-2xl bg-primary-50 text-primary-600 flex items-center justify-center font-black text-lg shadow-inner border border-primary-100/50">{format(new Date(entry.punch_in_time), "dd")}</div>
                        <div>
                          <div className="font-black text-slate-900 text-sm tracking-tight uppercase">{format(new Date(entry.punch_in_time), "EEEE")}</div>
                          <div className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-0.5">{formatDate(entry.punch_in_time)}</div>
                        </div>
                      </div>
                    </td>
                    <td className="px-8 py-5 text-center">
                      <div className="flex flex-col items-center gap-1">
                        <span className="text-xs font-black font-mono text-slate-700">{formatTime(entry.punch_in_time)}</span>
                        <div className="w-1 h-1 bg-emerald-500 rounded-full"></div>
                      </div>
                    </td>
                    <td className="px-8 py-5 text-center">
                      {entry.punch_out_time ? (
                        <div className="flex flex-col items-center gap-1">
                          <span className="text-xs font-black font-mono text-slate-700">{formatTime(entry.punch_out_time)}</span>
                          <div className="w-1 h-1 bg-rose-500 rounded-full"></div>
                        </div>
                      ) : <span className="text-slate-300 font-black tracking-widest italic text-[10px]">--:--</span>}
                    </td>
                    <td className="px-8 py-5 text-center">
                      {entry.duration_minutes ? (
                        <span className="inline-flex items-center px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 text-[10px] font-black uppercase tracking-tighter border border-slate-200">{Math.floor(entry.duration_minutes / 60)}h {entry.duration_minutes % 60}m</span>
                      ) : <span className="text-slate-300 font-black">-</span>}
                    </td>
                    <td className="px-8 py-5 text-right">
                      <div className="flex items-center justify-end gap-3">
                        <span className={cn("px-3 py-1 rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm", entry.punch_out_time ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-amber-50 text-amber-600 border-amber-100 animate-pulse")}>{entry.punch_out_time ? t("common.completed") : t("common.pending")}</span>
                        {entry.generation_metadata && (
                          <Popover.Root>
                            <Popover.Trigger asChild>
                              <button className="p-2 text-slate-300 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all active:scale-90 opacity-0 group-hover:opacity-100"><Info size={16} /></button>
                            </Popover.Trigger>
                            <Popover.Portal>
                              <Popover.Content className="z-50 w-80 bg-slate-900 rounded-[2rem] shadow-2xl border border-white/5 p-6 animate-in fade-in zoom-in-95" side="left" align="center" sideOffset={12}>
                                <div className="space-y-4">
                                  <div className="flex items-center gap-3 border-b border-white/10 pb-4">
                                    <AlertCircle size={16} className="text-primary-400" />
                                    <span className="text-[10px] font-black uppercase tracking-[0.2em] text-white opacity-80">{t("ess.attendance.generation_info")}</span>
                                  </div>
                                  <div className="bg-white/5 p-4 rounded-2xl border border-white/5">
                                    <pre className="text-[10px] text-emerald-400 overflow-x-auto whitespace-pre-wrap font-mono leading-relaxed">{JSON.stringify(entry.generation_metadata, null, 2)}</pre>
                                  </div>
                                </div>
                                <Popover.Arrow className="fill-slate-900" />
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
}    </div>
  );
}
