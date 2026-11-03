/* eslint-disable react/forbid-elements */
import { useQueries } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { CalendarRange, Calendar, Save } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Employee, fetchPatternDetails, getStandaloneOrganizationId } from "@/lib/api";
import { Shift, WorkingPattern } from "@/lib/api/types";

import { PatternCard } from "./pattern-card";

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
  const wsId = getStandaloneOrganizationId() ?? "";
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
      <div className="max-h-[45dvh] lg:max-h-[50dvh] min-h-[240px] overflow-y-auto pr-1.5 flex flex-col gap-3.5 custom-scrollbar">
        {patterns?.map((pattern, index) => (
          <PatternCard
            details={patternDetailsQueries[index]?.data}
            isExpanded={expandedPatternId === pattern.id}
            isLoadingDetails={patternDetailsQueries[index]?.isLoading}
            isSelected={selectedPatternId === pattern.id}
            key={pattern.id}
            onSelect={() => { setSelectedPatternId(pattern.id); }}
            onToggleExpand={() => {
              setExpandedPatternId(expandedPatternId === pattern.id ? null : pattern.id);
            }}
            pattern={pattern}
            shifts={shifts}
          />
        ))}
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
