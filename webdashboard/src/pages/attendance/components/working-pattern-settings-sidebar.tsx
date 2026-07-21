import { CalendarDays, RefreshCw } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import FormField, { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface WorkingPatternSettingsSidebarProps {
  cycleLengthDays: number;
  description: string;
  handleCycleLengthChange: (len: number) => void;
  handlePatternTypeChange: (type: "weekly" | "rotation") => void;
  name: string;
  patternType: "weekly" | "rotation";
  setDescription: (val: string) => void;
  setIsDirty: (dirty: boolean) => void;
  setName: (val: string) => void;
}

export function WorkingPatternSettingsSidebar({
  cycleLengthDays,
  description,
  handleCycleLengthChange,
  handlePatternTypeChange,
  name,
  patternType,
  setDescription,
  setIsDirty,
  setName,
}: WorkingPatternSettingsSidebarProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-6 sm:p-10 shadow-sm space-y-8 flex flex-col justify-start">
      <div className="space-y-6">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800 pb-4" variant="h3">
          {t("patterns.form.desc_label").toUpperCase()}
        </Typography>
        <FormField
          inputClassName="font-bold tracking-tight rounded-2xl"
          label={t("patterns.form.name_label")}
          onChange={(e) => {
            setName(e.target.value);
            setIsDirty(true);
          }}
          placeholder={t("patterns.form.name_placeholder")}
          required
          value={name}
        />
        <FormField
          inputClassName="rounded-2xl min-h-[5rem]"
          label={t("patterns.form.desc_label")}
          onChange={(e) => {
            setDescription(e.target.value);
            setIsDirty(true);
          }}
          placeholder={t("patterns.form.desc_placeholder")}
          type="textarea"
          value={description}
        />
      </div>

      <div className="space-y-6">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] border-b border-slate-50 dark:border-slate-800 pb-4" variant="h3">
          {t("patterns.form.pattern_type").toUpperCase()}
        </Typography>
        
        <div className="space-y-4">
          <label className={LABEL_BASE}>{t("patterns.form.pattern_type").toUpperCase()}</label>
          <div className="flex bg-slate-50 dark:bg-slate-800 p-1.5 rounded-2xl border border-slate-100 dark:border-slate-700/50">
            <Button className={cn(
                "flex-1 flex items-center justify-center gap-2.5 py-3 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                patternType === "weekly"
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100/50 dark:border-slate-850"
                  : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
              )}
              onClick={() => { handlePatternTypeChange("weekly"); }}
              type="button"
              variant="ghost"
            >
              <CalendarDays size={14} />
              {t("patterns.form.pattern_type_weekly") || "Weekly"}
            </Button>
            <Button className={cn(
                "flex-1 flex items-center justify-center gap-2.5 py-3 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                patternType === "rotation"
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100/50 dark:border-slate-850"
                  : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
              )}
              onClick={() => { handlePatternTypeChange("rotation"); }}
              type="button"
              variant="ghost"
            >
              <RefreshCw size={14} />
              {t("patterns.form.pattern_type_rotation") || "Rotation"}
            </Button>
          </div>
        </div>

        {patternType === "rotation" && (
          <div className="p-5 bg-slate-50/50 dark:bg-slate-850/30 border border-slate-100 dark:border-slate-800 rounded-2xl space-y-4 animate-in fade-in slide-in-from-top-4 duration-300">
            <div className="flex items-center gap-2 text-slate-700 dark:text-slate-200">
              <RefreshCw className="text-primary-500" size={14} />
              <span className="text-[10px] font-extrabold uppercase tracking-wider">
                {t("patterns.form.cycle_config_title") || "Rotation Configuration"}
              </span>
            </div>
            
            <div className="w-28">
              <FormField
                className="!px-0"
                inputClassName="font-bold tracking-tight rounded-2xl text-center"
                label={t("patterns.form.cycle_length") || "Days"}
                onChange={(e) => {
                  handleCycleLengthChange(parseInt(e.target.value) || 1);
                }}
                placeholder="e.g. 6"
                required
                type="number"
                value={String(cycleLengthDays)}
              />
            </div>

            <Typography className="text-[9px] text-slate-400 font-bold uppercase leading-relaxed italic opacity-75" variant="p">
              {t("patterns.form.cycle_length_note")}
            </Typography>
          </div>
        )}
      </div>
    </div>
  );
}
