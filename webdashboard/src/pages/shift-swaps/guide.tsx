import { ChevronDown, ChevronRight, HelpCircle, Info } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";

interface ShiftSwapGuideProps {
  setShowGuide: (show: boolean) => void;
  showGuide: boolean;
  t: TFunction;
}

export function ShiftSwapGuide({
  setShowGuide,
  showGuide,
  t,
}: ShiftSwapGuideProps) {
  return (
    <div className="bg-gradient-to-br from-blue-50 to-sky-50 dark:from-blue-950/30 dark:to-sky-950/30 border border-blue-100 dark:border-blue-800/50 rounded-[2rem] overflow-hidden shadow-sm">
      <button
        className="flex items-center justify-between w-full px-8 py-5 group"
        onClick={() => {
          setShowGuide(!showGuide);
        }}
        type="button"
      >
        <div className="flex items-center gap-3">
          <div className="p-2 bg-blue-100 dark:bg-blue-900/50 rounded-xl">
            <HelpCircle className="text-blue-600 dark:text-blue-400" size={18} />
          </div>
          <span className="text-sm font-bold text-blue-700 dark:text-blue-300 uppercase tracking-widest">
            {t("shift_swaps.guide.title")}
          </span>
        </div>
        {showGuide ? (
          <ChevronDown className="text-blue-400" size={16} />
        ) : (
          <ChevronRight className="text-blue-400" size={16} />
        )}
      </button>

      {showGuide ? (
        <div className="px-8 pb-8 animate-in slide-in-from-top-4 duration-300">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 pt-4 border-t border-blue-100 dark:border-blue-800/30">
            <GuideStep
              description={t("shift_swaps.guide.step1.desc")}
              number={1}
              title={t("shift_swaps.guide.step1.title")}
            />
            <GuideStep
              description={t("shift_swaps.guide.step2.desc")}
              number={2}
              title={t("shift_swaps.guide.step2.title")}
            />
            <GuideStep
              description={t("shift_swaps.guide.step3.desc")}
              number={3}
              title={t("shift_swaps.guide.step3.title")}
            />
          </div>

          <div className="mt-6 p-4 bg-white/50 dark:bg-slate-900/30 rounded-2xl flex items-start gap-3">
            <Info className="text-blue-500 mt-0.5 shrink-0" size={16} />
            <Typography className="text-xs font-bold text-blue-800 dark:text-blue-200 tracking-tight leading-relaxed" variant="p">
              {t("shift_swaps.guide.note")}
            </Typography>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function GuideStep({
  description,
  number,
  title,
}: {
  description: string;
  number: number;
  title: string;
}) {
  return (
    <div className="space-y-3">
      <div className="w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400 flex items-center justify-center font-bold font-mono text-xs">
        {number}
      </div>
      <Typography className="font-bold text-sm text-slate-700 dark:text-slate-300" variant="h4">{title}</Typography>
      <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider leading-relaxed opacity-80" variant="p">
        {description}
      </Typography>
    </div>
  );
}
