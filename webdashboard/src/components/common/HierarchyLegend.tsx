import React, { useState } from 'react';
import { ChevronDown, ChevronUp, HelpCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';

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
    <div className="bg-white border border-slate-200 rounded-[24px] overflow-hidden transition-all duration-300 shadow-sm">
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-6 py-3 flex items-center justify-between hover:bg-slate-50 transition-colors"
      >
        <div className="flex items-center gap-3">
          <div className="p-1.5 bg-blue-50 text-blue-600 rounded-lg">
            {icon}
          </div>
          <span className="text-[10px] font-black uppercase tracking-widest text-slate-500">
            {title}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <HelpCircle size={14} className="text-slate-300" />
          {isOpen ? <ChevronUp size={14} className="text-slate-400" /> : <ChevronDown size={14} className="text-slate-400" />}
        </div>
      </button>

      {isOpen && (
        <div className="px-6 pb-6 pt-2 animate-in slide-in-from-top-2 duration-200">
          <div className="relative flex flex-col gap-0 pl-2">
            {/* Vertical Line */}
            <div className="absolute left-[7px] top-2 bottom-2 w-[2px] bg-slate-100" />

            {steps.map((p, idx) => (
              <div key={idx} className={`relative flex items-start gap-4 pb-4 last:pb-0 group ${p.isActive ? 'opacity-100' : 'opacity-60 grayscale hover:grayscale-0 transition-all'}`}>
                {/* Node */}
                <div className="relative z-10 mt-1">
                  <div className={`w-3 h-3 rounded-full border-2 border-white ring-2 ring-slate-50 ${p.color} transition-transform group-hover:scale-125 duration-200 ${p.isActive ? 'scale-125 ring-blue-100' : ''}`} />
                </div>

                {/* Content */}
                <div className="flex flex-col flex-1">
                  <div className="flex items-center justify-between">
                    <span className={`text-[11px] font-black uppercase tracking-tight ${p.isActive ? 'text-slate-900' : 'text-slate-500'}`}>
                      {p.label}
                    </span>
                    <span className="text-[9px] font-black text-slate-300 group-hover:text-slate-500 transition-colors bg-slate-50 px-1.5 py-0.5 rounded">
                      P{p.priority}
                    </span>
                  </div>
                  
                  {idx === 0 && (
                    <span className="text-[9px] font-bold text-blue-500 uppercase tracking-widest mt-0.5">
                      {t("shifts.traceability.highest_priority")}
                    </span>
                  )}
                  {idx === steps.length - 1 && (
                    <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest mt-0.5">
                      {t("shifts.traceability.lowest_priority")}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>

          <div className="mt-6 p-3 bg-blue-50/50 rounded-xl border border-blue-100/50">
            <p className="text-[10px] text-slate-500 font-medium leading-relaxed">
              <span className="font-black text-blue-600 uppercase mr-1">{t("common.note")}:</span>
              {explanation}
            </p>
          </div>
        </div>
      )}
    </div>
  );
};
