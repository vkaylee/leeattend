import React, { useState } from 'react';
import { ChevronDown, ChevronUp, HelpCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '@/lib/utils';

export interface HierarchyStep {
  label: string;
  priority: number;
  color: string;
  isActive?: boolean;
}

export interface HierarchyLegendProps {
  steps: HierarchyStep[];
  title: string;
  icon: React.ReactNode;
  explanation: string;
}

export const HierarchyLegend: React.FC<HierarchyLegendProps> = ({ steps, title, icon, explanation }) => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="bg-white border border-slate-100 rounded-[2rem] overflow-hidden transition-all duration-500 shadow-2xl shadow-primary-900/5">
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-8 py-4 flex items-center justify-between hover:bg-slate-50 transition-all bg-slate-50/30"
      >
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-xl bg-primary-50 text-primary-600 flex items-center justify-center border border-primary-100 shadow-inner">
            {icon}
          </div>
          <span className="text-[10px] font-black uppercase tracking-[0.25em] text-slate-500">
            {title}
          </span>
        </div>
        <div className="flex items-center gap-3">
          <HelpCircle size={16} className="text-slate-200" />
          <div className={cn("w-8 h-8 rounded-lg bg-white border border-slate-100 flex items-center justify-center text-slate-400 transition-transform duration-300", isOpen && "rotate-180")}>
            <ChevronDown size={16} />
          </div>
        </div>
      </button>

      {isOpen && (
        <div className="px-8 pb-10 pt-6 animate-in slide-in-from-top-4 duration-500">
          <div className="relative flex flex-col gap-0 pl-3">
            {/* Vertical Line */}
            <div className="absolute left-[9.5px] top-4 bottom-4 w-px bg-slate-100" />

            {steps.map((p, idx) => (
              <div key={idx} className={`relative flex items-start gap-6 pb-6 last:pb-0 group ${p.isActive ? 'opacity-100' : 'opacity-40 grayscale hover:opacity-100 hover:grayscale-0 transition-all duration-500'}`}>
                {/* Node */}
                <div className="relative z-10 mt-1.5">
                  <div className={`w-3.5 h-3.5 rounded-full border-[3px] border-white shadow-md ring-4 ${p.color} transition-transform group-hover:scale-125 duration-300 ${p.isActive ? 'scale-125 ring-primary-50' : 'ring-slate-50'}`} />
                </div>

                {/* Content */}
                <div className="flex flex-col flex-1">
                  <div className="flex items-center justify-between gap-4">
                    <span className={`text-[11px] font-black uppercase tracking-tight leading-tight ${p.isActive ? 'text-slate-900' : 'text-slate-500'}`}>
                      {p.label}
                    </span>
                    <span className="text-[9px] font-black text-slate-400 group-hover:text-primary-600 transition-colors bg-slate-50 px-2 py-1 rounded-lg border border-slate-100 shadow-sm">
                      P{p.priority}
                    </span>
                  </div>
                  
                  {idx === 0 && (
                    <div className="mt-1.5 flex items-center gap-1.5"><div className="w-1 h-1 bg-primary-500 rounded-full animate-pulse" /><span className="text-[8px] font-black text-primary-500 uppercase tracking-widest">{t("shifts.traceability.highest_priority")}</span></div>
                  )}
                  {idx === steps.length - 1 && (
                    <div className="mt-1.5 flex items-center gap-1.5"><div className="w-1 h-1 bg-slate-300 rounded-full" /><span className="text-[8px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.traceability.lowest_priority")}</span></div>
                  )}
                </div>
              </div>
            ))}
          </div>

          <div className="mt-10 p-6 bg-slate-50 rounded-[1.5rem] border border-slate-100 shadow-inner group/note">
            <div className="flex items-center gap-2 mb-3">
               <HelpCircle size={14} className="text-primary-500 opacity-50" />
               <span className="text-[9px] font-black text-slate-400 uppercase tracking-[0.2em]">{t("common.note")}</span>
            </div>
            <p className="text-[11px] text-slate-500 font-bold leading-relaxed uppercase tracking-tight">
              {explanation}
            </p>
          </div>
        </div>
      )}
    </div>
  );
};
