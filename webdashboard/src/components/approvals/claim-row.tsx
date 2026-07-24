import React from "react";

import type { TFunction } from "i18next";

import type { ClaimItem } from "./confirm-claims-dialog";

interface ClaimRowProps {
  claim: ClaimItem;
  handleMinutesChange: (index: number, val: number) => void;
  handleReasonChange: (index: number, val: string) => void;
  idx: number;
  t: TFunction;
}

export function ClaimRow({
  claim,
  handleMinutesChange,
  handleReasonChange,
  idx,
  t,
}: ClaimRowProps) {
  return (
    <tr className="hover:bg-slate-50/50 dark:hover:bg-slate-800/20 transition-colors" key={`${claim.employment_id}-${claim.date}`}>
      <td className="p-3 pl-4">
        <div className="flex flex-col">
          <span className="font-bold text-xs text-slate-800 dark:text-slate-200">
            {claim.employee_name}
          </span>
          <span className="text-[9px] font-mono text-slate-400 dark:text-slate-500">
            {claim.employee_id_number} • {claim.date}
          </span>
        </div>
      </td>
      <td className="p-3 text-xs text-slate-600 dark:text-slate-400">
        {claim.computed_minutes} min ({(claim.computed_minutes / 60).toFixed(1)} hrs)
      </td>
      <td className="p-3">
        <div className="flex items-center gap-1.5">
          <input
            className="w-[70px] px-2 py-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-xs font-mono text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-primary-500"
            min={1}
            onChange={(e) => { handleMinutesChange(idx, parseInt(e.target.value) || 0); }}
            type="number"
            value={claim.claimed_minutes}
          />
          <span className="text-[10px] text-slate-400 font-mono whitespace-nowrap">
            {(claim.claimed_minutes / 60).toFixed(1)}h
          </span>
        </div>
      </td>
      <td className="p-3 pr-4">
        <input
          className="w-full px-2 py-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-xs text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-primary-500"
          onChange={(e) => { handleReasonChange(idx, e.target.value); }}
          placeholder={t("ot_approvals.delegated_claim.table.reason")}
          type="text"
          value={claim.reason}
        />
      </td>
    </tr>
  );
}
