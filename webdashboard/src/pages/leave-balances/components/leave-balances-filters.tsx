import { Filter, Calendar, Search } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";

interface LeaveBalancesFiltersProps {
  searchTerm: string;
  selectedYear: number;
  setSearchTerm: (term: string) => void;
  setSelectedYear: (year: number) => void;
  totalItems: number;
}

export const LeaveBalancesFilters: React.FC<LeaveBalancesFiltersProps> = ({
  searchTerm,
  selectedYear,
  setSearchTerm,
  setSelectedYear,
  totalItems,
}) => {
  const { t } = useTranslation();
  const currentYear = new Date().getFullYear();
  const yearOptions = Array.from({ length: 5 }, (_, index) => currentYear - 2 + index);

  return (
    <div className="p-5 md:p-8 border-b border-slate-50 dark:border-slate-800 flex flex-col md:flex-row md:items-center justify-between gap-5 md:gap-6 bg-slate-50 dark:bg-slate-800/30">
      <div className="flex items-center gap-3">
        <Filter className="w-5 h-5 text-primary-600 dark:text-primary-400" />
        <h2 className="text-lg md:text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">{t("leave_balances.summary_table")}</h2>
        {totalItems > 0 && (
          <span className="px-2 py-0.5 bg-slate-200 dark:bg-slate-600 text-slate-600 dark:text-slate-300 rounded-full text-xs font-bold">
            {totalItems}
          </span>
        )}
        <div className="flex items-center gap-2 ml-2">
          <Calendar className="w-4 h-4 text-slate-400 dark:text-slate-500" />
          <Combobox
            className="w-24 bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 rounded-xl text-xs font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider focus:ring-2 focus:ring-primary-500/20 py-[5px]"
            onChange={(value) => {
              if (value) {
                setSelectedYear(Number(value));
              }
            }}
            options={yearOptions.map(y => ({ value: y.toString(), label: y.toString() }))}
            placeholder="Select Year"
            value={selectedYear.toString()}
          />
        </div>
      </div>

      <div className="relative group w-full md:w-80">
        <Search className="absolute left-4 md:left-5 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" size={18} />
        <input 
          className="w-full pl-12 md:pl-14 pr-5 md:pr-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm min-h-[44px]" 
          onChange={(e) => { setSearchTerm(e.target.value); }} 
          placeholder={t("common.search_placeholder")} 
          type="text" 
          value={searchTerm}
        />
      </div>
    </div>
  );
};
