import React, { useState } from 'react';
import { ChevronDown, ChevronUp, Layers, HelpCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export const PriorityLegend: React.FC = () => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);

  const priorities = [
    { label: t("shifts.traceability.p100_manual"), priority: 100, color: "bg-purple-500" },
    { label: t("shifts.traceability.p80_pattern"), priority: 80, color: "bg-blue-500" },
    { label: t("shifts.traceability.p60_rules"), priority: 60, color: "bg-indigo-500" },
    { label: t("shifts.traceability.p40_dept_pattern"), priority: 40, color: "bg-slate-500" },
    { label: t("shifts.traceability.p20_fixed"), priority: 20, color: "bg-emerald-500" },
    { label: t("shifts.traceability.p10_dept_fixed"), priority: 10, color: "bg-slate-400" },
  ];

  return (
    <div className="bg-white border border-slate-200 rounded-[24px] overflow-hidden transition-all duration-300 shadow-sm">
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-6 py-3 flex items-center justify-between hover:bg-slate-50 transition-colors"
      >
        <div className="flex items-center gap-3">
          <div className="p-1.5 bg-blue-50 text-blue-600 rounded-lg">
            <Layers size={14} />
          </div>
          <span className="text-[10px] font-black uppercase tracking-widest text-slate-500">
            {t("shifts.traceability.priority_hierarchy")}
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

            {priorities.map((p, idx) => (
              <div key={idx} className="relative flex items-start gap-4 pb-4 last:pb-0 group">
                {/* Node */}
                <div className="relative z-10 mt-1">
                  <div className={`w-3 h-3 rounded-full border-2 border-white ring-2 ring-slate-50 ${p.color} transition-transform group-hover:scale-125 duration-200`} />
                </div>

                {/* Content */}
                <div className="flex flex-col flex-1">
                  <div className="flex items-center justify-between">
                    <span className="text-[11px] font-black text-slate-700 uppercase tracking-tight">
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
                  {idx === priorities.length - 1 && (
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
              {t("shifts.traceability.hierarchy_explanation", "Rules are evaluated from top to bottom. Higher nodes override lower nodes.")}
            </p>
          </div>
        </div>
      )}
    </div>
  );
};
