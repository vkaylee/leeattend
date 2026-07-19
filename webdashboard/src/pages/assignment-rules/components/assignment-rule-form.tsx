import { Filter } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";
import { AssignmentRuleConditions } from "@/lib/api";

interface AssignmentRuleFormProps {
  conditions: AssignmentRuleConditions;
  departments: { id: string; name: string }[] | undefined;
  employers: { id: string; name: string }[] | undefined;
  endDate: string;
  isIndefinite: boolean;
  locations: { id: string; name: string }[] | undefined;
  name: string;
  patternId: string;
  patterns: { id: string; name: string }[] | undefined;
  priority: number;
  setConditions: (conditions: AssignmentRuleConditions) => void;
  setEndDate: (date: string) => void;
  setIsIndefinite: (isIndefinite: boolean) => void;
  setName: (name: string) => void;
  setPatternId: (id: string) => void;
  setPriority: (priority: number) => void;
  setStartDate: (date: string) => void;
  startDate: string;
}

  // eslint-disable-next-line max-lines-per-function
export const AssignmentRuleForm: React.FC<AssignmentRuleFormProps> = ({
  conditions,
  departments,
  employers,
  locations,
  name,
  patternId,
  patterns,
  priority,
  setConditions,
  setName,
  setPatternId,
  setPriority,
  startDate,
  setStartDate,
  endDate,
  setEndDate,
  isIndefinite,
  setIsIndefinite,
  // eslint-disable-next-line complexity
}) => {
  const { t } = useTranslation();

  return (
    <div className="space-y-6 pt-2">
      {/* Basic rule info */}
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

      {/* Validity Dates Section */}
      <div className="bg-slate-50 dark:bg-slate-800 rounded-[32px] p-6 border border-slate-100 dark:border-slate-800 space-y-4">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="h4">
          Thời hạn hiệu lực
        </Typography>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 items-end">
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.start_date", "Ngày bắt đầu")}</label>
            <input
              className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-700 rounded-xl px-4 py-2 text-sm font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
              onChange={(e) => { setStartDate(e.target.value); }}
              type="date"
              value={startDate}
            />
          </div>

          <div className="flex items-center space-x-2 py-3.5">
            <input
              checked={isIndefinite}
              className="w-4 h-4 text-primary-600 bg-white border-slate-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-slate-800 dark:bg-slate-700 dark:border-slate-600 cursor-pointer"
              id="is_indefinite"
              onChange={(e) => {
                setIsIndefinite(e.target.checked);
                if (e.target.checked) setEndDate("");
              }}
              type="checkbox"
            />
            <label className="text-xs font-bold text-slate-600 dark:text-slate-300 uppercase tracking-wider cursor-pointer select-none" htmlFor="is_indefinite">
              Vô thời hạn
            </label>
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("rules.form.end_date", "Ngày kết thúc")}</label>
            <input
              className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-700 rounded-xl px-4 py-2 text-sm font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 disabled:opacity-50 disabled:bg-slate-100 dark:disabled:bg-slate-800 transition-all"
              disabled={isIndefinite}
              onChange={(e) => { setEndDate(e.target.value); }}
              type="date"
              value={endDate}
            />
          </div>
        </div>
      </div>

      {/* Conditions Section */}
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
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("rules.form.employer", "Công ty pháp nhân")}</label>
            <Combobox
              className="w-full bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-700 rounded-xl text-sm font-bold text-slate-700 dark:text-slate-200"
              onChange={(value) => { setConditions({ ...conditions, employer_id: value || undefined }); }}
              options={(employers ?? []).map((e) => ({ value: e.id, label: e.name }))}
              placeholder={t("rules.form.all_employers", "Tất cả công ty")}
              value={conditions.employer_id ?? ""}
            />
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("rules.form.location", "Chi nhánh địa điểm")}</label>
            <Combobox
              className="w-full bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-700 rounded-xl text-sm font-bold text-slate-700 dark:text-slate-200"
              onChange={(value) => { setConditions({ ...conditions, location_id: value || undefined }); }}
              options={(locations ?? []).map((l) => ({ value: l.id, label: l.name }))}
              placeholder={t("rules.form.all_locations", "Tất cả địa điểm")}
              value={conditions.location_id ?? ""}
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

          <div className="space-y-2 md:col-span-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("rules.form.employee_id_numbers", "Gán riêng nhân viên (Mã nhân sự, ngăn cách bằng dấu phẩy)")}</label>
            <input
              className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-700 rounded-xl px-4 py-2 text-sm font-bold text-slate-700 dark:text-slate-200"
              onChange={(e) => {
                const ids = e.target.value.split(",").map(t => t.trim()).filter(t => t !== "");
                setConditions({ ...conditions, employee_id_numbers: ids.length > 0 ? ids : undefined });
              }}
              placeholder="VD: EMP001, EMP002"
              type="text"
              value={conditions.employee_id_numbers?.join(", ") ?? ""}
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
