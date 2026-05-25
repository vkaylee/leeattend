import { Palette, Undo2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { useTheme } from "@/hooks/use-theme";
import { cn } from "@/lib/utils";

export const PersonalizationTab: React.FC = () => {
  const { t } = useTranslation();
  const { primaryColor, isOverridden, setUserTheme } = useTheme();

  const colorPresets = [
    { name: t("common.colors.emerald"), color: '#10b981' }, 
    { name: t("common.colors.blue"), color: '#2563eb' },
    { name: t("common.colors.emerald"), color: '#059669' }, 
    { name: t("common.colors.amber"), color: '#d97706' }, 
    { name: t("common.colors.rose"), color: '#e11d48' },
    { name: t("common.colors.slate"), color: '#475569' }, 
    { name: t("common.colors.black"), color: '#0f172a' },
  ];

  return (
    <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm p-12 space-y-12 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 rounded-2xl bg-pink-50 flex items-center justify-center text-pink-600 shadow-inner border border-pink-100/50">
            <Palette size={28} />
          </div>
          <div>
            <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-1" variant="h2">
              {t("profile.personalization.title")}
            </Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70" variant="p">
              {t("profile.personalization.subtitle")}
            </Typography>
          </div>
        </div>
        {isOverridden ? (
          <button 
            className="flex items-center gap-3 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-all bg-slate-50 dark:bg-slate-800 px-6 py-2.5 rounded-xl border border-slate-100 dark:border-slate-800 shadow-sm active:scale-95" 
            onClick={() => { setUserTheme(null); }}
          >
            <Undo2 size={14} /> {t("profile.personalization.use_workspace_default").toUpperCase()}
          </button>
        ) : null}
      </div>

      <div className="space-y-10">
        <div className="space-y-6">
          <label className={LABEL_BASE}>{t("common.workspace_settings.primary_color")}</label>
          <div className="flex flex-wrap gap-5 items-center">
            {colorPresets.map((preset) => (
              <button 
                className={cn(
                  "w-14 h-14 rounded-2xl transition-all border-4 flex items-center justify-center shadow-lg group active:scale-90", 
                  primaryColor === preset.color 
                    ? 'border-slate-900 scale-110' 
                    : 'border-white hover:scale-105 hover:border-slate-100 dark:border-slate-800'
                )} 
                key={preset.color} 
                onClick={() => { setUserTheme(preset.color); }} 
                style={{ backgroundColor: preset.color }} 
                title={preset.name}
              >
                {primaryColor === preset.color && (
                  <div className="w-3 h-3 bg-white dark:bg-slate-900 rounded-full shadow-2xl animate-in zoom-in duration-300" />
                )}
              </button>
            ))}
            <div className="h-12 w-px bg-slate-100 dark:bg-slate-700 mx-2" />
            <div className="flex items-center gap-4 bg-slate-50 dark:bg-slate-800 p-2.5 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner group">
              <input 
                className="w-12 h-12 rounded-xl cursor-pointer bg-transparent border-none p-0 overflow-hidden shadow-sm" 
                onChange={(e) => { setUserTheme(e.target.value); }} 
                type="color" 
                value={primaryColor} 
              />
              <input 
                className="bg-transparent border-none text-sm font-mono font-bold text-slate-600 dark:text-slate-300 w-24 outline-none tracking-tighter" 
                onChange={(e) => { setUserTheme(e.target.value); }} 
                type="text" 
                value={primaryColor} 
              />
            </div>
          </div>
        </div>
        <div className="p-10 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-inner">
          <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 leading-relaxed italic uppercase tracking-tight opacity-80" variant="p">
            {t("profile.personalization.note")}
          </Typography>
        </div>
      </div>
    </div>
  );
};
