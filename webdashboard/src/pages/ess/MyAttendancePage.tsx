import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useMyAttendance } from "@/hooks/use-ess";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { startOfMonth, endOfMonth, format, subMonths, addMonths } from "date-fns";
import {
  CalendarDays, 
  Clock, 
  MapPin, 
  ChevronLeft, 
  ChevronRight,
  Info,
  Calendar,
  Activity,
  History
} from "lucide-react";import { cn } from "@/lib/utils";
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

    <div className="space-y-10 animate-in fade-in duration-700 pb-20">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">

        <div className="space-y-1">

          <h1 className="text-4xl font-black tracking-tighter text-slate-900 uppercase leading-none">{t("ess.attendance.title")}</h1>

          <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("ess.attendance.subtitle")}</p>

        </div>



        <div className="flex items-center gap-4 bg-white p-2.5 rounded-[2.5rem] border border-slate-100 shadow-xl shadow-slate-900/5">

          <div className="flex items-center bg-slate-100/50 rounded-[1.5rem] p-1.5 shadow-inner border border-slate-200/50">

            <button onClick={handlePrevMonth} className="w-12 h-12 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-md rounded-xl transition-all active:scale-90"><ChevronLeft size={24} /></button>

            

            <Popover.Root>

              <Popover.Trigger asChild>

                <button className="px-10 flex flex-col items-center min-w-[200px] cursor-pointer hover:bg-white hover:shadow-md rounded-xl transition-all py-2 group outline-none border border-transparent hover:border-slate-100">

                  <span className="text-[8px] font-black text-slate-400 uppercase tracking-[0.3em] leading-none mb-1.5 group-hover:text-primary-500 transition-colors">{t("payroll.table.period").toUpperCase()}</span>

                  <span className="text-sm font-black text-slate-900 group-hover:text-primary-600 transition-colors uppercase tracking-tight">

                    {format(currentMonth, "MMMM yyyy")}

                  </span>

                </button>

              </Popover.Trigger>

              <Popover.Portal>

                <Popover.Content className="z-50 w-80 p-4 rounded-[2.5rem] shadow-premium-2xl border border-slate-100 bg-white animate-in zoom-in-95 slide-in-from-top-4 origin-top" sideOffset={12} align="center">

                  <MonthYearPicker month={currentMonth.getMonth() + 1} year={currentMonth.getFullYear()} onChange={handleMonthYearChange} />

                </Popover.Content>

              </Popover.Portal>

            </Popover.Root>



            <button onClick={handleNextMonth} className="w-12 h-12 flex items-center justify-center text-slate-400 hover:text-primary-600 hover:bg-white hover:shadow-md rounded-xl transition-all active:scale-90"><ChevronRight size={24} /></button>

          </div>

        </div>

      </div>



      <div className="bg-white border border-slate-100 rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">

        {isLoading ? (

          <div className="p-32 flex flex-col items-center gap-6 animate-pulse">

            <div className="w-12 h-12 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>

            <span className="text-[10px] font-black text-slate-400 uppercase tracking-[0.3em]">{t("common.loading")}</span>

          </div>

        ) : entries.length === 0 ? (

          <div className="py-40 text-center flex flex-col items-center border-4 border-dashed border-slate-50 m-8 rounded-[3rem] bg-slate-50/30">

            <div className="w-24 h-24 bg-white rounded-full flex items-center justify-center shadow-xl border border-slate-100 mb-8"><Calendar size={40} className="text-slate-200" /></div>

            <h3 className="text-xl font-bold text-slate-900 uppercase tracking-tight">{t("ess.attendance.no_data")}</h3>

            <p className="text-slate-400 text-[10px] font-black uppercase tracking-widest mt-2 opacity-50 italic">{t("common.no_records_found")}</p>

          </div>

        ) : (

          <div className="overflow-x-auto">

            <table className="w-full text-left border-collapse">

              <thead>

                <tr className="bg-slate-50/50 border-b border-slate-100">

                  <th className="px-10 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em]">{t("ess.attendance.date")}</th>

                  <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-center">{t("common.in")}</th>

                  <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-center">{t("common.out")}</th>

                  <th className="px-8 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-center">{t("ess.attendance.duration")}</th>

                  <th className="px-10 py-6 text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] text-right">{t("common.status")}</th>

                </tr>

              </thead>

              <tbody className="divide-y divide-slate-50">

                {entries.map((entry) => (

                  <tr key={entry.id} className="hover:bg-slate-50/50 transition-all group">

                    <td className="px-10 py-6">

                      <div className="flex items-center gap-6">

                        <div className="w-14 h-14 rounded-2xl bg-primary-50 text-primary-600 flex items-center justify-center font-black text-2xl shadow-inner border border-primary-100/50 group-hover:scale-110 transition-transform">{format(new Date(entry.punch_in_time), "dd")}</div>

                        <div>

                          <div className="font-black text-slate-900 text-sm tracking-tight uppercase leading-none mb-1.5">{format(new Date(entry.punch_in_time), "EEEE")}</div>

                          <div className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] opacity-70 leading-none">{formatDate(entry.punch_in_time)}</div>

                        </div>

                      </div>

                    </td>

                    <td className="px-8 py-6 text-center">

                      <div className="flex flex-col items-center gap-2">

                        <span className="text-sm font-black font-mono text-slate-700 tracking-tighter">{formatTime(entry.punch_in_time)}</span>

                        <div className="w-1.5 h-1.5 bg-emerald-500 rounded-full shadow-sm shadow-emerald-500/50 animate-pulse"></div>

                      </div>

                    </td>

                    <td className="px-8 py-6 text-center">

                      {entry.punch_out_time ? (

                        <div className="flex flex-col items-center gap-2">

                          <span className="text-sm font-black font-mono text-slate-700 tracking-tighter">{formatTime(entry.punch_out_time)}</span>

                          <div className="w-1.5 h-1.5 bg-rose-500 rounded-full shadow-sm shadow-rose-500/50"></div>

                        </div>

                      ) : <span className="text-slate-300 font-black tracking-widest italic text-[10px] uppercase opacity-50">Active...</span>}

                    </td>

                    <td className="px-8 py-6 text-center">

                      {entry.duration_minutes ? (

                        <span className="inline-flex items-center px-3 py-1 rounded-xl bg-slate-100/50 text-slate-600 text-[10px] font-black uppercase tracking-widest border border-slate-200/50 shadow-inner">{Math.floor(entry.duration_minutes / 60)}h {entry.duration_minutes % 60}m</span>

                      ) : <span className="text-slate-300 font-black">-</span>}

                    </td>

                    <td className="px-10 py-6 text-right">

                      <div className="flex items-center justify-end gap-4">

                        <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm", entry.punch_out_time ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-amber-50 text-amber-600 border-amber-100 animate-pulse")}>{entry.punch_out_time ? t("common.completed").toUpperCase() : t("common.pending").toUpperCase()}</span>

                        {entry.generation_metadata && (

                          <Popover.Root>

                            <Popover.Trigger asChild>

                              <button className="p-2.5 bg-white text-slate-300 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90 opacity-0 group-hover:opacity-100"><Info size={18} /></button>

                            </Popover.Trigger>

                            <Popover.Portal>

                              <Popover.Content className="z-50 w-96 bg-slate-900 rounded-[2.5rem] shadow-premium-2xl border border-white/10 p-8 animate-in fade-in zoom-in-95 duration-500" side="left" align="center" sideOffset={16}>

                                <div className="space-y-6">

                                  <div className="flex items-center gap-4 border-b border-white/10 pb-6">

                                    <div className="w-10 h-10 rounded-xl bg-primary-500/10 flex items-center justify-center text-primary-400 shadow-inner border border-primary-500/20"><Activity size={20} /></div>

                                    <span className="text-[10px] font-black uppercase tracking-[0.3em] text-white opacity-80">{t("ess.attendance.generation_info").toUpperCase()}</span>

                                  </div>

                                  <div className="bg-black/40 p-6 rounded-[1.5rem] border border-white/5 shadow-inner">

                                    <pre className="text-[11px] text-emerald-400 overflow-x-auto whitespace-pre-wrap font-mono leading-relaxed selection:bg-emerald-500/20">{JSON.stringify(entry.generation_metadata, null, 2)}</pre>

                                  </div>

                                </div>

                                <Popover.Arrow className="fill-slate-900" width={20} height={10} />

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
