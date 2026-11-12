import { ArrowLeftRight } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";

interface ShiftSwapCreateFormProps {
  handleCreate: () => void;
  isPending: boolean;
  reason: string;
  requesterId: string;
  setReason: (v: string) => void;
  setRequesterId: (v: string) => void;
  setShowCreateForm: (v: boolean) => void;
  setSwapDate: (v: string) => void;
  setTargetId: (v: string) => void;
  swapDate: string;
  t: TFunction;
  targetId: string;
}

export function ShiftSwapCreateForm({
  handleCreate,
  isPending,
  reason,
  requesterId,
  setReason,
  setRequesterId,
  setShowCreateForm,
  setSwapDate,
  setTargetId,
  swapDate,
  t,
  targetId,
}: ShiftSwapCreateFormProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2rem] shadow-sm animate-in zoom-in-95 duration-300 mb-8">
      <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight mb-6 flex items-center gap-2" variant="h3">
        <ArrowLeftRight className="text-primary-500" />
        {t("shift_swaps.create_title")}
      </Typography>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-6">
        <EmployeeSelect
          label={t("shift_swaps.requester")}
          onChange={setRequesterId}
          placeholder={t("common.employee")}
          value={requesterId}
        />
        <EmployeeSelect
          disabled={!requesterId}
          excludeId={requesterId}
          label={t("shift_swaps.target")}
          onChange={setTargetId}
          placeholder={t("common.employee")}
          value={targetId}
        />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
        <FormField
          id="swap_date"
          inputClassName="text-slate-700 dark:text-slate-300 border-slate-100 dark:border-slate-800"
          label={t("shift_swaps.swap_date")}
          name="swap_date"
          onChange={(e) => {
            setSwapDate(e.target.value);
          }}
          required
          type="date"
          value={swapDate}
        />
        <FormField
          id="swap_reason"
          inputClassName="text-slate-700 dark:text-slate-300 border-slate-100 dark:border-slate-800"
          label={t("shift_swaps.reason")}
          name="reason"
          onChange={(e) => {
            setReason(e.target.value);
          }}
          placeholder={t("common.no_notes")}
          type="text"
          value={reason}
        />
      </div>

      <div className="flex justify-end gap-4 pt-6 border-t border-slate-50 dark:border-slate-800">
        <Button
          onClick={() => {
            setShowCreateForm(false);
          }}
          variant="outline"
        >
          {t("common.cancel")}
        </Button>
        <Button disabled={isPending || !requesterId || !targetId || !swapDate}
          onClick={() => {
            handleCreate();
          }}
          variant="primary"
        >
          {t("common.save")}
        </Button>
      </div>
    </div>
  );
}

function EmployeeSelect({
  disabled,
  excludeId,
  label,
  onChange,
  placeholder,
  value,
}: {
  disabled?: boolean;
  excludeId?: string;
  label: string;
  onChange: (v: string) => void;
  placeholder: string;
  value: string;
}) {
  return (
    <div className="space-y-3">
      <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
        {label}
      </label>
      <EmployeeSearch
        onChange={(selected) => {
          const id = typeof selected === "string" ? selected : selected[0] ?? "";
          onChange(id === excludeId ? "" : id);
        }}
        placeholder={placeholder}
        value={value}
      />
    </div>
  );
}
