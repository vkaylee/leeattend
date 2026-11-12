import { Filter } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";
import { SearchInput } from "@/components/ui/search-input";

export interface ContractsFiltersProps {
  onSearchChange: (value: string) => void;
  onStatusChange: (value: string) => void;
  onTypeChange: (value: string) => void;
  search: string;
  selectedStatus: string;
  selectedType: string;
}

export function ContractsFilters({
  search,
  onSearchChange,
  selectedType,
  onTypeChange,
  selectedStatus,
  onStatusChange
}: ContractsFiltersProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-6 rounded-[2rem] shadow-sm flex flex-col md:flex-row items-center justify-between gap-4">
      <div className="w-full md:w-80">
        <SearchInput
          aria-label={t("employees.details.contracts.search_label", { defaultValue: "Search contracts" })}
          containerClassName="h-10"
          onChange={(e) => onSearchChange(e.target.value)}
          onClear={() => onSearchChange("")}
          placeholder={t("employees.details.contracts.search_placeholder_input", { defaultValue: "Search employee, ID or contract..." })}
          size="sm"
          value={search}
        />
      </div>
      <div className="w-full md:w-auto flex flex-wrap md:flex-nowrap items-center gap-3">
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
            <Filter size={12} /> {t("common.type", { defaultValue: "TYPE" })}
          </span>
          <Combobox
            className="h-10 text-xs bg-slate-50 dark:bg-slate-855 border border-slate-200 dark:border-slate-800 rounded-xl text-slate-600 dark:text-slate-300 w-44"
            onChange={onTypeChange}
            options={[
              { value: "all", label: t("employees.details.contracts.types.all", { defaultValue: "ALL TYPES" }) },
              { value: "probation", label: t("employees.details.contracts.types.probation_upper", { defaultValue: "PROBATION" }) },
              { value: "training", label: t("employees.details.contracts.types.training_upper", { defaultValue: "TRAINING" }) },
              { value: "fixed_term", label: t("employees.details.contracts.types.fixed_term_upper", { defaultValue: "FIXED TERM" }) },
              { value: "indefinite", label: t("employees.details.contracts.types.indefinite_upper", { defaultValue: "INDEFINITE" }) }
            ]}
            placeholder={t("employees.details.contracts.types.all", { defaultValue: "ALL TYPES" })}
            value={selectedType}
          />
        </div>

        <div className="flex items-center gap-2">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1">
            <Filter size={12} /> {t("common.status", { defaultValue: "STATUS" })}
          </span>
          <Combobox
            className="h-10 text-xs bg-slate-50 dark:bg-slate-855 border border-slate-200 dark:border-slate-800 rounded-xl text-slate-600 dark:text-slate-350 w-44"
            onChange={onStatusChange}
            options={[
              { value: "all", label: t("employees.details.contracts.status.all", { defaultValue: "ALL STATUS" }) },
              { value: "draft", label: t("employees.details.contracts.status.draft_upper", { defaultValue: "DRAFT" }) },
              { value: "active", label: t("employees.details.contracts.status.active_upper", { defaultValue: "ACTIVE" }) },
              { value: "expired", label: t("employees.details.contracts.status.expired_upper", { defaultValue: "EXPIRED" }) },
              { value: "terminated", label: t("employees.details.contracts.status.terminated_upper", { defaultValue: "TERMINATED" }) }
            ]}
            placeholder={t("employees.details.contracts.status.all", { defaultValue: "ALL STATUS" })}
            value={selectedStatus}
          />
        </div>
      </div>
    </div>
  );
}
