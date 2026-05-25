import { Filter } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";
import { AssignmentRuleConditions } from "@/lib/api";

interface AssignmentRuleFormProps {
  conditions: AssignmentRuleConditions;
  departments: { id: string; name: string }[] | undefined;
  name: string;
  patternId: string;
  patterns: { id: string; name: string }[] | undefined;
  priority: number;
  setConditions: (conditions: AssignmentRuleConditions) => void;
  setName: (name: string) => void;
  setPatternId: (id: string) => void;
  setPriority: (priority: number) => void;
}

export const AssignmentRuleForm: React.FC<AssignmentRuleFormProps> = ({
  conditions,
  departments,
  name,
  patternId,
  patterns,
  priority,
  setConditions,
  setName,
  setPatternId,
  setPriority,
}) => {
  const { t } = useTranslation();

  return (
    <div className="space-y-6 pt-4">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.name", "Rule Name")}</label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
            onChange={(e) => { setName(e.target.value); }}
            placeholder={t("rules.form.name_placeholder")}
            type="text"
            value={name}
          />
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.priority")}</label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
            onChange={(e) => { setPriority(parseInt(e.target.value) || 0); }}
            type="number"
            value={priority}
          />
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.pattern")}</label>
        <Combobox
          className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl py-[5px] font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
          data-testid="pattern-combobox"
          onChange={(value) => { setPatternId(value); }}
          options={(patterns ?? []).map((p) => ({ value: p.id, label: p.name }))}
          placeholder={t("patterns.form.select_pattern_placeholder")}
          value={patternId}
        />
      </div>

      <div className="bg-slate-50 dark:bg-slate-800 rounded-[32px] p-6 border border-slate-100 dark:border-slate-800 space-y-4">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-2" variant="h4">
          <Filter size={14} /> {t("rules.conditions_title")}
        </Typography>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("common.department")}</label>
            <Combobox
              className="w-full bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-700 rounded-xl text-sm font-bold text-slate-700 dark:text-slate-200"
              onChange={(value) => { setConditions({ ...conditions, department_id: value || undefined }); }}
              options={(departments ?? []).map((d) => ({ value: d.id, label: d.name }))}
              placeholder={t("common.all_departments")}
              value={conditions.department_id ?? ""}
            />
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("common.tags")}</label>
            <input
              className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-700 rounded-xl px-4 py-2 text-sm font-bold text-slate-700 dark:text-slate-200"
              onChange={(e) => {
                const tags = e.target.value.split(",").map(t => t.trim()).filter(t => t !== "");
                setConditions({ ...conditions, tags: tags.length > 0 ? tags : undefined });
              }}
              placeholder={t("rules.form.tags_placeholder")}
              type="text"
              value={conditions.tags?.join(", ") ?? ""}
            />
          </div>
        </div>

        <Typography className="text-[10px] text-slate-500 dark:text-slate-400 italic" variant="p">
          {t("rules.condition_help")}
        </Typography>
      </div>
    </div>
  );
};
