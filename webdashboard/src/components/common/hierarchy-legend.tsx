import { ChevronDown, HelpCircle } from 'lucide-react';
import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Typography } from "@/components/ui/typography";
import { cn } from '@/lib/utils';

export interface HierarchyStep {
  color: string;
  isActive?: boolean;
  label: string;
  priority: number;
}

export interface HierarchyLegendProps {
  explanation: string;
  icon: React.ReactNode;
  steps: HierarchyStep[];
  title: string;
}

export const HierarchyLegend: React.FC<HierarchyLegendProps> = ({ steps, title, icon, explanation }) => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] overflow-hidden transition-all duration-500 shadow-2xl shadow-primary-900/5">
      <button 
        className="w-full px-8 py-4 flex items-center justify-between hover:bg-slate-50 dark:hover:bg-slate-800 transition-all bg-slate-50 dark:bg-slate-800/30"
        onClick={() => { setIsOpen(!isOpen); }}
      >
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 flex items-center justify-center border border-primary-100 dark:border-primary-800 shadow-inner">
            {icon}
          </div>
          <span className="text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400">
            {title}
          </span>
        </div>
        <div className="flex items-center gap-3">
          <HelpCircle className="text-slate-200 dark:text-slate-700" size={16} />
          <div className={cn("w-8 h-8 rounded-lg bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 transition-transform duration-300", isOpen && "rotate-180")}>
            <ChevronDown size={16} />
          </div>
        </div>
      </button>

      {isOpen ? (
        <div className="px-8 pb-10 pt-6 animate-in slide-in-from-top-4 duration-500">
          <div className="relative flex flex-col gap-0 pl-3">
            {/* Vertical Line */}
            <div className="absolute left-[9.5px] top-4 bottom-4 w-px bg-slate-100 dark:bg-slate-700" />

            {steps.map((p, index) => (
              <div className={`relative flex items-start gap-6 pb-6 last:pb-0 group ${p.isActive ? 'opacity-100' : 'opacity-40 grayscale hover:opacity-100 hover:grayscale-0 transition-all duration-500'}`} key={index}>
                {/* Node */}
                <div className="relative z-10 mt-1.5">
                  <div className={`w-3.5 h-3.5 rounded-full border-[3px] border-white shadow-md ring-4 ${p.color} transition-transform group-hover:scale-125 duration-300 ${p.isActive ? 'scale-125 ring-primary-50' : 'ring-slate-50'}`} />
                </div>

                {/* Content */}
                <div className="flex flex-col flex-1">
                  <div className="flex items-center justify-between gap-4">
                    <span className={`text-[11px] font-bold uppercase tracking-tight leading-tight ${p.isActive ? 'text-slate-700 dark:text-slate-200' : 'text-slate-500 dark:text-slate-400'}`}>
                      {p.label}
                    </span>
                    <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors bg-slate-50 dark:bg-slate-800 px-2 py-1 rounded-lg border border-slate-100 dark:border-slate-800 shadow-sm">
                      P{p.priority}
                    </span>
                  </div>
                  
                  {(index === 0) && (
                    <div className="mt-1.5 flex items-center gap-1.5"><div className="w-1 h-1 bg-primary-50 dark:bg-primary-950 rounded-full animate-pulse" /><span className="text-[8px] font-bold text-primary-500 uppercase tracking-widest">{t("shifts.traceability.highest_priority")}</span></div>
                  )}
                  {(index === steps.length - 1) && (
                    <div className="mt-1.5 flex items-center gap-1.5"><div className="w-1 h-1 bg-slate-300 rounded-full" /><span className="text-[8px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.traceability.lowest_priority")}</span></div>
                  )}
                </div>
              </div>
            ))}
          </div>

          <div className="mt-10 p-6 bg-slate-50 dark:bg-slate-800 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner group/note">
            <div className="flex items-center gap-2 mb-3">
               <HelpCircle className="text-primary-500 opacity-50" size={14} />
               <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">{t("common.note")}</span>
            </div>
            <Typography className="text-[11px] text-slate-500 dark:text-slate-400 font-bold leading-relaxed uppercase tracking-tight" variant="p">
              {explanation}
            </Typography>
          </div>
        </div>
      ) : null}
    </div>
  );
};
