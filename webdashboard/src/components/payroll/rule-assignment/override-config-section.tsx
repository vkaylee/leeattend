import { Sliders } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface OverrideConfigSectionProps {
  calculationMethod?: "percentage" | "fixed" | "tiered";
  hasOverride: boolean;
  overrideAmount: string;
  overrideBase: string;
  overrideRate: string;
  setHasOverride: (val: boolean) => void;
  setOverrideAmount: (val: string) => void;
  setOverrideBase: (val: string) => void;
  setOverrideRate: (val: string) => void;
}

function OverrideConfigToggle({
  hasOverride,
  setHasOverride,
}: {
  hasOverride: boolean;
  setHasOverride: (val: boolean) => void;
}) {
  const { t } = useTranslation();
  return (
    <>
      {/* eslint-disable-next-line jsx-a11y/label-has-associated-control */}
      <label
      className={cn(
        "flex items-center gap-4 p-5 rounded-2xl cursor-pointer transition-all border-2 active:scale-[0.98]",
        hasOverride
          ? "bg-orange-50 dark:bg-orange-950/30 border-orange-100 dark:border-orange-800 shadow-lg"
          : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:border-slate-200 dark:hover:border-slate-700"
      )}
      htmlFor="override-config"
    >
      <input
        checked={hasOverride}
        className="w-5 h-5 rounded-lg text-orange-600 focus:ring-orange-500 border-slate-300 dark:border-slate-600 cursor-pointer"
        id="override-config"
        onChange={(e) => { setHasOverride(e.target.checked); }}
        type="checkbox"
      />
      <div>
        <Typography className="font-bold text-sm text-slate-700 dark:text-slate-200 uppercase tracking-tight flex items-center gap-2" variant="p">
          <Sliders className="opacity-50" size={14} />{" "}
          {t("payroll.override_config", "Override Default Config")}
        </Typography>
        <Typography className="text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70" variant="p">
          {t(
            "payroll.override_config_desc",
            "Apply a different rate or amount for this specific assignment"
          )}
        </Typography>
      </div>
    </label>
    </>
  );
}

function PercentageOverrideForm({
  overrideRate, setOverrideRate,
  overrideBase, setOverrideBase,
}: {
  overrideBase: string;
  overrideRate: string;
  setOverrideBase: (val: string) => void;
  setOverrideRate: (val: string) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-2 gap-4">
      <div className="space-y-2">
        <label
          className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest"
          htmlFor="override-rate"
        >
          {t("payroll.override_rate", "Override Rate (%)")}
        </label>
        <input
          className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none"
          id="override-rate"
          onChange={(e) => { setOverrideRate(e.target.value); }}
          placeholder="e.g. 10.5"
          step="0.01"
          type="number"
          value={overrideRate}
        />
      </div>
      <div className="space-y-2">
        <label
          className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest"
          htmlFor="override-base"
        >
          {t("payroll.override_base", "Override Base")}
        </label>
        <select
          className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none uppercase tracking-tight"
          id="override-base"
          onChange={(e) => { setOverrideBase(e.target.value); }}
          value={overrideBase}
        >
          <option value="gross">
            {t("payroll.base_gross", "Gross Salary")}
          </option>
          <option value="gross_salary">
            {t("payroll.base_gross_alt", "Gross Salary (alt)")}
          </option>
          <option value="taxable_income">
            {t("payroll.base_taxable", "Taxable Income")}
          </option>
        </select>
      </div>
    </div>
  );
}

function FixedOverrideForm({ overrideAmount, setOverrideAmount }: { overrideAmount: string; setOverrideAmount: (val: string) => void; }) {
  const { t } = useTranslation();
  return (
    <div className="space-y-2">
      <label
        className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest"
        htmlFor="override-amount"
      >
        {t("payroll.override_amount", "Override Amount (₫)")}
      </label>
      <input
        className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none"
        id="override-amount"
        onChange={(e) => { setOverrideAmount(e.target.value); }}
        placeholder="e.g. 500000"
        type="number"
        value={overrideAmount}
      />
    </div>
  );
}

export function OverrideConfigSection({
  calculationMethod,
  hasOverride,
  overrideAmount,
  overrideBase,
  overrideRate,
  setHasOverride,
  setOverrideAmount,
  setOverrideBase,
  setOverrideRate,
}: OverrideConfigSectionProps) {

  if (calculationMethod !== "percentage" && calculationMethod !== "fixed") {
    return null;
  }

  return (
    <div className="space-y-4">
      <OverrideConfigToggle hasOverride={hasOverride} setHasOverride={setHasOverride} />

      {hasOverride ? (
        <div className="bg-orange-50 dark:bg-orange-950/20 rounded-2xl p-5 border border-orange-100 dark:border-orange-800 space-y-4 animate-in slide-in-from-top-2 duration-200">
          {calculationMethod === "percentage" ? (
            <PercentageOverrideForm overrideBase={overrideBase} overrideRate={overrideRate} setOverrideBase={setOverrideBase} setOverrideRate={setOverrideRate} />
          ) : null}
          {calculationMethod === "fixed" ? (
            <FixedOverrideForm overrideAmount={overrideAmount} setOverrideAmount={setOverrideAmount} />
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
