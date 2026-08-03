// PatternCard — extracted from PatternSelector in pattern-assignment-form.tsx to reduce complexity.
import { ChevronDown, ChevronUp } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { Shift, WorkingPattern, WorkingPatternDetail  } from "@/lib/api/types";

import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { getShiftIntervals } from "@/lib/shift-card-metrics";

interface PatternCardProps {
  details?: WorkingPatternDetail[];
  isExpanded: boolean;
  isLoadingDetails?: boolean;
  isSelected: boolean;
  onSelect: () => void;
  onToggleExpand: () => void;
  pattern: WorkingPattern;
  shifts?: Shift[];
}

function getDayShiftInfo(
  details: WorkingPatternDetail[] | undefined,
  shifts: Shift[] | undefined,
  dayIdx: number
) {
  if (!details) return null;
  const dayDetail = details.find((d) => d.schedule_day === dayIdx && d.shift_id);
  if (!dayDetail) return null;
  return shifts?.find((s) => s.id === dayDetail.shift_id);
}

 
function renderDayPill(shiftInfo: Shift | undefined | null, label: string, t: (k: string) => string) {
  const isOff = !shiftInfo;
  const dayLabel = label;
  const badgeText = isOff ? "OFF" : (shiftInfo.report_symbol ?? shiftInfo.name.substring(0, 2).toUpperCase());
  const tooltipTitle = (() => {
    if (isOff) return t("status_off");
    const ivs = getShiftIntervals(shiftInfo);
    if (ivs.length === 0) return shiftInfo.name;
    const times = ivs.map((iv) => `${iv.start_time}-${iv.end_time}`).join(", ");
    return `${shiftInfo.name} (${times})`;
  })();
  const pillClass = isOff
    ? "bg-slate-100 dark:bg-slate-800/60 text-slate-400 dark:text-slate-500 border-slate-200/55 dark:border-slate-700/55 opacity-75"
    : "bg-primary-50 dark:bg-primary-950/40 text-primary-700 dark:text-primary-300 border-primary-100 dark:border-primary-800/80";

  return (
    <div
      className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[9px] font-bold border transition-all ${pillClass}`}
      key={label}
      title={tooltipTitle}
    >
      <span>{dayLabel}</span>
      <span className="opacity-30">|</span>
      <span>{badgeText}</span>
    </div>
  );
}

// eslint-disable-next-line complexity -- card with status badges, rotation/cycle summary, expand toggle; presentation-only branching
export function PatternCard({
  details,
  isLoadingDetails,
  isExpanded,
  isSelected,
  onSelect,
  onToggleExpand,
  pattern,
  shifts,
}: PatternCardProps) {
  const { t } = useTranslation();
  const isRotation = pattern.pattern_type === "rotation";
  const cycleLength = pattern.cycle_length_days ?? 7;
  const isLongRotation = isRotation && cycleLength > 7;

  const daysOfWeekShort = [
    t("days.sunday_short"),
    t("days.monday_short"),
    t("days.tuesday_short"),
    t("days.wednesday_short"),
    t("days.thursday_short"),
    t("days.friday_short"),
    t("days.saturday_short"),
  ];

  return (
    <div
      className={`px-5 py-4 rounded-[24px] border-2 text-left transition-all relative flex flex-col gap-2.5 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-primary-500 ${
        isSelected
          ? "border-primary-600 bg-primary-50 dark:bg-primary-950/40 shadow-sm"
          : "border-slate-150 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700 bg-slate-50 dark:bg-slate-800/30 shadow-sm"
      }`}
      onClick={onSelect}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect();
        }
      }}
      role="button"
      tabIndex={0}
    >
      {isSelected ? (
        <div className="absolute top-4 right-4 w-2 h-2 bg-primary-600 dark:bg-primary-400 rounded-full flex items-center justify-center">
          <span className="absolute inline-flex h-full w-full rounded-full bg-primary-400 opacity-75 animate-ping" />
          <span className="relative inline-flex rounded-full h-1 w-1 bg-primary-600" />
        </div>
      ) : null}

      <div className="flex items-start justify-between gap-4 w-full">
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

        <div className="flex items-center gap-2 flex-shrink-0">
          {isLoadingDetails ? (
            <div className="flex items-center gap-2 text-[10px] text-slate-400 dark:text-slate-500 font-semibold italic">
              <div className="w-3 h-3 border-2 border-slate-350 dark:border-slate-650 border-t-transparent rounded-full animate-spin" />
              {t("common.loading", "Đang tải...")}
            </div>
          ) : isLongRotation ? (
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
                      off: offDaysCount,
                    })}
                  </div>
                );
              })()}
              <div
                className="p-1.5 rounded-lg text-slate-450 hover:text-slate-650 dark:text-slate-550 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-primary-500"
                onClick={(e) => {
                  e.stopPropagation();
                  onToggleExpand();
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.stopPropagation();
                    e.preventDefault();
                    onToggleExpand();
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

      {!isLoadingDetails && !isLongRotation && (
        <div className="w-full pt-1">
          <div className="flex flex-wrap items-center gap-2">
            {isRotation
              ? Array.from({ length: cycleLength }).map((_, dayIdx) =>
                  renderDayPill(getDayShiftInfo(details, shifts, dayIdx), `D${dayIdx + 1}`, t)
                )
              : [1, 2, 3, 4, 5, 6, 0].map((dayIdx) =>
                  renderDayPill(getDayShiftInfo(details, shifts, dayIdx), daysOfWeekShort[dayIdx], t)
                )}
          </div>
        </div>
      )}

      {isLongRotation && isExpanded ? (
        <div
          className="pt-4 border-t border-slate-150 dark:border-slate-850/80 animate-in slide-in-from-top-2 duration-300 w-full"
          onClick={(e) => { e.stopPropagation(); }}
          onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') e.stopPropagation(); }}
          role="button"
          tabIndex={0}
        >
          <div className="flex flex-wrap gap-2 max-h-48 overflow-y-auto pr-1">
            {Array.from({ length: cycleLength }).map((_, dayIdx) =>
              renderDayPill(getDayShiftInfo(details, shifts, dayIdx), `D${dayIdx + 1}`, t)
            )}
          </div>
        </div>
      ) : null}
    </div>
  );
}
