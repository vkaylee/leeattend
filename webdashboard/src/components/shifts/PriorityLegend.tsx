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
        <div className="px-6 pb-4 animate-in slide-in-from-top-2 duration-200">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-x-8 gap-y-3 pt-2 border-t border-slate-50">
            {priorities.map((p, idx) => (
              <div key={idx} className="flex items-center justify-between group">
                <div className="flex items-center gap-2">
                  <div className={`w-1.5 h-1.5 rounded-full ${p.color}`} />
                  <span className="text-[10px] font-bold text-slate-600 uppercase tracking-tight">
                    {p.label}
                  </span>
                </div>
                <span className="text-[9px] font-black text-slate-300 group-hover:text-slate-500 transition-colors">
                  P{p.priority}
                </span>
              </div>
            ))}
          </div>
          <p className="mt-4 text-[9px] text-slate-400 font-medium italic leading-relaxed">
            * {t("shifts.traceability.title")}: Rules with higher priority numbers will override lower ones. Manual changes (Roster) always have the final say.
          </p>
        </div>
      )}
    </div>
  );
};
