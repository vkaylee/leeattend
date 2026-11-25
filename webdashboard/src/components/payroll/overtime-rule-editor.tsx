import { CheckCircle2, Clock, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import type { OvertimeDayType, OvertimePolicyVersion, OvertimeRuleV2 } from "@/lib/api/payroll";

import { useOvertimePolicyVersionMutations } from "./hooks/use-overtime-policies";

interface OvertimeRuleEditorProps {
  policyId: string;
  versions: OvertimePolicyVersion[];
}

export default function OvertimeRuleEditor({ policyId, versions }: OvertimeRuleEditorProps) {
  const { t } = useTranslation();
  const { createVersion, patchVersion, publishVersion } = useOvertimePolicyVersionMutations(policyId);

  const latestDraft = versions.find((v) => v.status === "draft") ?? null;
  const activeVersion = latestDraft ?? versions[0] ?? null;

  const [dailyCap, setDailyCap] = useState(activeVersion?.daily_cap_minutes ? String(activeVersion.daily_cap_minutes) : "240");
  const [monthlyCap, setMonthlyCap] = useState(activeVersion?.monthly_cap_hours ?? "40");
  const [yearlyCap, setYearlyCap] = useState(activeVersion?.yearly_cap_hours ?? "200");
  const [rules, setRules] = useState<OvertimeRuleV2[]>(activeVersion?.rules ?? []);

  const [dayType, setDayType] = useState<OvertimeDayType>("weekday");
  const [priority, setPriority] = useState("50");
  const [multiplier, setMultiplier] = useState("1.50");
  const [timeStart, setTimeStart] = useState("");
  const [timeEnd, setTimeEnd] = useState("");

  const addRuleToDraft = () => {
    const next: OvertimeRuleV2 = {
      priority: Number(priority) || 0,
      day_type: dayType,
      days_of_week: 127,
      multiplier: multiplier || "1.50",
      ...(timeStart && timeEnd ? { time_start: timeStart, time_end: timeEnd } : {}),
    };
    setRules((prev) => [...prev, next]);
  };

  const handleSaveDraft = () => {
    const payload = {
      daily_cap_minutes: dailyCap ? Number(dailyCap) : null,
      monthly_cap_hours: monthlyCap || null,
      yearly_cap_hours: yearlyCap || null,
      rules,
    };
    if (latestDraft) {
      patchVersion.mutate({ versionId: latestDraft.id, data: { ...payload, expected_row_version: latestDraft.row_version } });
    } else {
      createVersion.mutate(payload);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2 text-xs text-slate-500">
          {versions.map((v) => (
            <span
              className={`rounded-full px-2.5 py-0.5 font-semibold ${
                v.status === "published"
                  ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300"
                  : "bg-amber-50 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300"
              }`}
              key={v.id}
            >
              v{v.version_no} · {v.status}
            </span>
          ))}
        </div>
        <div className="flex gap-2">
          <Button disabled={patchVersion.isPending || createVersion.isPending} onClick={handleSaveDraft} size="sm" variant="outline">
            {latestDraft ? t("payroll.work_policies.save_draft", "Save Draft") : t("payroll.work_policies.new_draft", "New Draft Version")}
          </Button>
          {latestDraft ? (
            <Button
              disabled={publishVersion.isPending || rules.length === 0}
              onClick={() => { publishVersion.mutate({ versionId: latestDraft.id, expectedRowVersion: latestDraft.row_version }); }}
              size="sm"
              variant="primary"
            >
              <CheckCircle2 className="mr-1.5 h-3.5 w-3.5" />
              {t("payroll.work_policies.publish", "Publish")}
            </Button>
          ) : null}
        </div>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
        <label className="text-xs text-slate-600 dark:text-slate-300">
          {t("payroll.overtime_daily_cap", "Daily Cap (min)")}
          <input className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setDailyCap(e.target.value); }} type="number" value={dailyCap} />
        </label>
        <label className="text-xs text-slate-600 dark:text-slate-300">
          {t("payroll.overtime_monthly_cap", "Monthly Cap (h)")}
          <input className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setMonthlyCap(e.target.value); }} type="number" value={monthlyCap} />
        </label>
        <label className="text-xs text-slate-600 dark:text-slate-300">
          {t("payroll.overtime_yearly_cap", "Yearly Cap (h)")}
          <input className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setYearlyCap(e.target.value); }} type="number" value={yearlyCap} />
        </label>
      </div>

      <div className="space-y-2">
        {rules.length === 0 ? (
          <Typography className="text-xs italic text-slate-400" variant="p">
            {t("payroll.no_overtime_rules", "No overtime rules configured yet.")}
          </Typography>
        ) : (
          rules.map((r, idx) => (
            <div className="flex items-center justify-between rounded-xl border border-slate-100 bg-slate-50 px-3 py-2 text-xs dark:border-slate-800 dark:bg-slate-800/40" key={`${r.day_type}-${r.priority}-${idx}`}>
              <div className="flex flex-wrap items-center gap-3 font-mono">
                <span className="font-bold text-primary-600 dark:text-primary-400">{r.multiplier}x</span>
                <span>{r.day_type}</span>
                <span>P{r.priority}</span>
                {r.time_start && r.time_end ? (
                  <span className="inline-flex items-center gap-1 text-slate-500">
                    <Clock size={12} /> {r.time_start}–{r.time_end}
                  </span>
                ) : null}
              </div>
              <Button onClick={() => { setRules((prev) => prev.filter((_, i) => i !== idx)); }} size="sm" variant="ghost">
                <Trash2 size={14} />
              </Button>
            </div>
          ))
        )}
      </div>

      <div className="grid grid-cols-2 gap-2 rounded-xl border border-slate-200 p-3 dark:border-slate-700 sm:grid-cols-6">
        <select aria-label="Day type" className="rounded-lg border border-slate-200 bg-white px-2 py-1.5 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setDayType(e.target.value as OvertimeDayType); }} value={dayType}>
          <option value="weekday">weekday</option>
          <option value="weekend">weekend</option>
          <option value="holiday">holiday</option>
          <option value="any">any</option>
        </select>
        <input aria-label="Priority" className="rounded-lg border border-slate-200 bg-white px-2 py-1.5 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setPriority(e.target.value); }} placeholder="Priority" type="number" value={priority} />
        <input aria-label="Multiplier" className="rounded-lg border border-slate-200 bg-white px-2 py-1.5 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setMultiplier(e.target.value); }} placeholder="1.50" step="0.1" type="number" value={multiplier} />
        <input aria-label="Start time" className="rounded-lg border border-slate-200 bg-white px-2 py-1.5 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setTimeStart(e.target.value); }} type="time" value={timeStart} />
        <input aria-label="End time" className="rounded-lg border border-slate-200 bg-white px-2 py-1.5 text-xs dark:border-slate-700 dark:bg-slate-900" onChange={(e) => { setTimeEnd(e.target.value); }} type="time" value={timeEnd} />
        <Button onClick={addRuleToDraft} size="sm" variant="outline">
          <Plus className="mr-1 h-3.5 w-3.5" /> {t("common.add", "Add")}
        </Button>
      </div>
    </div>
  );
}
