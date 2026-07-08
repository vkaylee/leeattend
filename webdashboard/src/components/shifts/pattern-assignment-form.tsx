import { useQueries } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { CalendarRange, Calendar, Save, ChevronDown, ChevronUp } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { Employee, fetchPatternDetails, getCurrentWorkspaceId } from "@/lib/api";
import { Shift, WorkingPattern } from "@/lib/api/types";

interface PatternAssignmentFormProps {
  endDate: string;
  isForever: boolean;
  isLoading: boolean;
  onAssign: () => void;
  patterns?: WorkingPattern[];
  selectedEmployee: Employee;
  selectedPatternId: string;
  setEndDate: (date: string) => void;
  setIsForever: (forever: boolean) => void;
  setSelectedPatternId: (id: string) => void;
  setStartDate: (date: string) => void;
  shifts?: Shift[];
  startDate: string;
}

function PatternSelector({
  patterns,
  selectedPatternId,
  setSelectedPatternId,
  shifts
}: {
  patterns?: WorkingPattern[];
  selectedPatternId: string;
  setSelectedPatternId: (id: string) => void;
  shifts?: Shift[];
}) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const [expandedPatternId, setExpandedPatternId] = useState<string | null>(null);

  const patternDetailsQueries = useQueries({
    queries: (patterns ?? []).map((pattern) => ({
      queryKey: ["pattern-details", wsId, pattern.id],
      queryFn: () => fetchPatternDetails(wsId, pattern.id),
      enabled: !!wsId && !!pattern.id,
      staleTime: 5 * 60 * 1000,
    })),
  });

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between ml-1 mr-2">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-2">
          <CalendarRange size={12} /> {t("patterns.select_pattern")}
        </label>
        <Link
          className="text-[10px] font-extrabold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 hover:underline flex items-center gap-1 transition-all"
          search={{ tab: "patterns" }}
          to="/shifts/rules"
        >
          {t("patterns.manage_btn", "Manage Patterns")} ↗
        </Link>
      </div>
      <div className="max-h-[45vh] lg:max-h-[50vh] min-h-[240px] overflow-y-auto pr-1.5 flex flex-col gap-3.5 custom-scrollbar">
        {patterns?.map((pattern, index) => {
          const detailsQuery = patternDetailsQueries[index];
          const details = detailsQuery?.data;
          const isLoadingDetails = detailsQuery?.isLoading;

          const isSelected = selectedPatternId === pattern.id;
          const isRotation = pattern.pattern_type === "rotation";
          const cycleLength = pattern.cycle_length_days ?? 7;
          const isLongRotation = isRotation && cycleLength > 7;
          const isExpanded = expandedPatternId === pattern.id;

          const getDayShiftInfo = (dayIdx: number) => {
            if (!details) return null;
            const dayDetail = details.find((d) => d.schedule_day === dayIdx && d.shift_id);
            if (!dayDetail) return null;
            return shifts?.find((s) => s.id === dayDetail.shift_id);
          };

          return (
            <div
              className={`px-5 py-4 rounded-[24px] border-2 text-left transition-all relative flex flex-col gap-2.5 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-primary-500 ${
                isSelected
                  ? "border-primary-600 bg-primary-50 dark:bg-primary-950/40 shadow-sm"
                  : "border-slate-150 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700 bg-slate-50 dark:bg-slate-800/30 shadow-sm"
              }`}
              key={pattern.id}
              onClick={() => { setSelectedPatternId(pattern.id); }}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  setSelectedPatternId(pattern.id);
                }
              }}
              role="button"
              tabIndex={0}
            >
              {/* Pulsing Green Dot in top right when selected */}
              {!!isSelected && (
                <div className="absolute top-4 right-4 w-2 h-2 bg-primary-600 dark:bg-primary-400 rounded-full flex items-center justify-center">
                  <span className="absolute inline-flex h-full w-full rounded-full bg-primary-400 opacity-75 animate-ping" />
                  <span className="relative inline-flex rounded-full h-1 w-1 bg-primary-600" />
                </div>
              )}

              {/* HÀNG TRÊN: Metadata xếp ngang */}
              <div className="flex items-start justify-between gap-4 w-full">
                
                {/* Left Side: Info */}
                <div className="space-y-1.5 max-w-[75%] sm:max-w-[80%]">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className={`font-extrabold text-sm uppercase tracking-tight ${
                      isSelected ? "text-primary-700 dark:text-primary-300" : "text-slate-700 dark:text-slate-200"
                    }`}>
                      {pattern.name}
                    </span>
                    <StatusBadge variant={isRotation ? "primary" : "success"}>
                      {isRotation 
                        ? `${t("patterns.form.pattern_type_rotation") || "Rotation"} (${cycleLength}d)`
                        : t("patterns.form.pattern_type_weekly") || "Weekly"
                      }
                    </StatusBadge>
                  </div>
                  <Typography className="text-[10px] text-slate-500 dark:text-slate-400 font-medium leading-normal line-clamp-2" variant="p">
                    {pattern.description ?? t("common.no_description")}
                  </Typography>
                </div>

                {/* Right Side: Chevron & Summary / Loading */}
                <div className="flex items-center gap-2 flex-shrink-0">
                  {isLoadingDetails ? (
                    <div className="flex items-center gap-2 text-[10px] text-slate-400 dark:text-slate-500 font-semibold italic">
                      <div className="w-3 h-3 border-2 border-slate-350 dark:border-slate-650 border-t-transparent rounded-full animate-spin" />
                      {t("common.loading", "Đang tải...")}
                    </div>
                  ) : isLongRotation ? (
                    // Option A: Long Rotation - Summary & Chevron
                    <div className="flex items-center gap-2">
                      {(() => {
                        const activeDaysCount = details?.filter((d) => d.shift_id).length ?? 0;
                        const offDaysCount = cycleLength - activeDaysCount;
                        return (
                          <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 px-3 py-1.5 rounded-xl border border-slate-200/30 dark:border-slate-700/30">
                            {t("patterns.cycle_summary", { 
                              defaultValue: "Chu kỳ {{total}} ngày: {{active}} làm / {{off}} nghỉ",
                              total: cycleLength,
                              active: activeDaysCount,
                              off: offDaysCount
                            })}
                          </div>
                        );
                      })()}
                      <div
                        className="p-1.5 rounded-lg text-slate-450 hover:text-slate-650 dark:text-slate-550 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-primary-500"
                        onClick={(e) => {
                          e.stopPropagation();
                          setExpandedPatternId(isExpanded ? null : pattern.id);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.stopPropagation();
                            e.preventDefault();
                            setExpandedPatternId(isExpanded ? null : pattern.id);
                          }
                        }}
                        role="button"
                        tabIndex={0}
                        title={isExpanded ? t("hide_details") : t("show_details")}
                      >
                        {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
                      </div>
                    </div>
                  ) : null}
                </div>
              </div>

              {/* HÀNG DƯỚI: Ưu tiên hiển thị lịch ca làm việc */}
              {!isLoadingDetails && !isLongRotation && (
                <div className="w-full pt-1">
                  <div className="flex flex-wrap items-center gap-2">
                    {isRotation ? (
                      // Short Rotation (<= 7 days)
                      Array.from({ length: cycleLength }).map((_, dayIdx) => {
                        const shiftInfo = getDayShiftInfo(dayIdx);
                        const isOff = !shiftInfo;
                        const dayLabel = `D${dayIdx + 1}`;
                        const badgeText = isOff ? "OFF" : (shiftInfo.report_symbol ?? shiftInfo.name.substring(0, 2).toUpperCase());
                        const tooltipTitle = isOff 
                          ? t("status_off") 
                          : `${shiftInfo.name} (${(shiftInfo as any).intervals?.[0]?.start_time ?? '--'} - ${(shiftInfo as any).intervals?.[0]?.end_time ?? '--'})`;
                        const pillClass = isOff
                          ? "bg-slate-100 dark:bg-slate-800/60 text-slate-400 dark:text-slate-500 border-slate-200/55 dark:border-slate-700/55 opacity-75"
                          : "bg-primary-50 dark:bg-primary-950/40 text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/80";
                        
                        return (
                          <div 
                            className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[9px] font-bold border transition-all ${pillClass}`} 
                            key={dayIdx}
                            title={tooltipTitle}
                          >
                            <span>{dayLabel}</span>
                            <span className="opacity-30">|</span>
                            <span>{badgeText}</span>
                          </div>
                        );
                      })

                    ) : (
                      // Weekly pattern (7 days T2-CN)
                      [1, 2, 3, 4, 5, 6, 0].map((dayIdx) => {
                        const daysOfWeekShort = [
                          t("days.sunday_short"),
                          t("days.monday_short"),
                          t("days.tuesday_short"),
                          t("days.wednesday_short"),
                          t("days.thursday_short"),
                          t("days.friday_short"),
                          t("days.saturday_short"),
                        ];
                        const shiftInfo = getDayShiftInfo(dayIdx);
                        const isOff = !shiftInfo;
                        const dayLabel = daysOfWeekShort[dayIdx];
                        const badgeText = isOff ? "OFF" : (shiftInfo.report_symbol ?? shiftInfo.name.substring(0, 2).toUpperCase());
                        const tooltipTitle = isOff 
                          ? t("status_off") 
                          : `${shiftInfo.name} (${(shiftInfo as any).intervals?.[0]?.start_time ?? '--'} - ${(shiftInfo as any).intervals?.[0]?.end_time ?? '--'})`;
                        const pillClass = isOff
                          ? "bg-slate-100 dark:bg-slate-800/60 text-slate-400 dark:text-slate-500 border-slate-200/55 dark:border-slate-700/55 opacity-75"
                          : "bg-primary-50 dark:bg-primary-950/40 text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/80";

                        return (
                          <div 
                            className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[9px] font-bold border transition-all ${pillClass}`} 
                            key={dayIdx}
                            title={tooltipTitle}
                          >
                            <span>{dayLabel}</span>
                            <span className="opacity-30">|</span>
                            <span>{badgeText}</span>
                          </div>
                        );
                      })
                    )}
                  </div>
                </div>
              )}

              {/* Accordion content for long rotation patterns */}
              {!!isLongRotation && !!isExpanded && (
                <div className="pt-4 border-t border-slate-150 dark:border-slate-850/80 animate-in slide-in-from-top-2 duration-300 w-full" onClick={(e) => { e.stopPropagation(); }}>
                  <div className="flex flex-wrap gap-2 max-h-48 overflow-y-auto pr-1">
                    {Array.from({ length: cycleLength }).map((_, dayIdx) => {
                      const shiftInfo = getDayShiftInfo(dayIdx);
                      const isOff = !shiftInfo;
                      const dayLabel = `D${dayIdx + 1}`;
                      const badgeText = isOff ? "OFF" : (shiftInfo.report_symbol ?? shiftInfo.name.substring(0, 2).toUpperCase());
                      const tooltipTitle = isOff 
                        ? t("status_off") 
                        : `${shiftInfo.name} (${(shiftInfo as any).intervals?.[0]?.start_time ?? '--'} - ${(shiftInfo as any).intervals?.[0]?.end_time ?? '--'})`;
                      const pillClass = isOff
                        ? "bg-slate-100 dark:bg-slate-800/60 text-slate-400 dark:text-slate-500 border-slate-200/55 dark:border-slate-700/55 opacity-75"
                        : "bg-primary-50 dark:bg-primary-950/40 text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/80";

                      return (
                        <div 
                          className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[9px] font-bold border transition-all ${pillClass}`} 
                          key={dayIdx}
                          title={tooltipTitle}
                        >
                          <span>{dayLabel}</span>
                          <span className="opacity-30">|</span>
                          <span>{badgeText}</span>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function PatternAssignmentForm({
  patterns,
  selectedPatternId,
  setSelectedPatternId,
  startDate,
  setStartDate,
  endDate,
  setEndDate,
  isForever,
  setIsForever,
  onAssign,
  isLoading,
  shifts,
}: PatternAssignmentFormProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      <div className="space-y-5">
        <PatternSelector
          patterns={patterns}
          selectedPatternId={selectedPatternId}
          setSelectedPatternId={setSelectedPatternId}
          shifts={shifts}
        />
        <div className="sticky bottom-0 bg-white dark:bg-slate-900 pt-4 pb-2 border-t border-slate-100 dark:border-slate-800/80 z-10">
          <div className="flex items-end gap-3.5 w-full">
            <div className="space-y-1.5 w-[140px] sm:w-[150px] flex-shrink-0">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-1.5 ml-1">
                <Calendar className="text-slate-400" size={12} /> {t("common.effective_from")}
              </label>
              <input
                className="w-full bg-slate-50/50 dark:bg-slate-800/40 border border-slate-200 dark:border-slate-700 rounded-xl px-3 py-2 font-bold text-xs text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 outline-none transition-all h-[38px]"
                onChange={(e) => { setStartDate(e.target.value); }}
                type="date"
                value={startDate}
              />
            </div>
            <div className="space-y-1.5 w-[190px] sm:w-[200px] flex-shrink-0">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-1.5 ml-1">
                <Calendar className="text-slate-400" size={12} /> {t("common.end_date")}
              </label>
              <div className="relative w-full">
                {isForever ? (
                  <input
                    className="w-full bg-slate-100 dark:bg-slate-800/60 text-slate-400 dark:text-slate-500 border border-slate-200/80 dark:border-slate-700 rounded-xl pl-3 pr-10 py-2 font-bold text-xs outline-none h-[38px]"
                    disabled
                    type="text"
                    value={t("common.indefinite")}
                  />
                ) : (
                  <input
                    className="w-full bg-slate-50/50 dark:bg-slate-800/40 border border-slate-200 dark:border-slate-700 rounded-xl pl-3 pr-10 py-2 font-bold text-xs text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 outline-none transition-all h-[38px]"
                    onChange={(e) => { setEndDate(e.target.value); }}
                    type="date"
                    value={endDate}
                  />
                )}
                <button
                  className={`absolute right-1.5 top-1/2 -translate-y-1/2 text-[12px] font-extrabold px-1.5 py-0.5 rounded-lg border transition-all h-[26px] w-[26px] flex items-center justify-center ${
                    isForever
                      ? 'bg-primary-600 text-white border-primary-600 shadow-sm shadow-primary-500/20'
                      : 'bg-white dark:bg-slate-700 text-slate-500 dark:text-slate-350 border-slate-200 dark:border-slate-650 hover:bg-slate-50 dark:hover:bg-slate-600'
                  }`}
                  onClick={() => { setIsForever(!isForever); }}
                  title={isForever ? t("common.definite") : t("common.indefinite")}
                  type="button"
                >
                  ∞
                </button>
              </div>
            </div>
            <Button className="flex-grow bg-slate-900 text-white font-bold py-2 rounded-xl flex items-center justify-center gap-1.5 hover:bg-primary-700 transition-all active:scale-[0.98] disabled:opacity-50 shadow-md shadow-slate-100 dark:shadow-black/20 uppercase tracking-widest text-[10px] h-[38px] self-end"
              disabled={isLoading}
              onClick={onAssign}
              variant="ghost"
            >
              <Save size={13} /> {t("shifts.assign_btn")}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

