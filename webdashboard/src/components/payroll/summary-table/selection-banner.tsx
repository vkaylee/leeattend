import { Info } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface SelectionBannerProps {
  filteredCount: number;
  isAllFilteredSelected: boolean;
  pageSize: number;
  selectAllFiltered: () => void;
  selectedCount: number;
  t: (key: string, options?: Record<string, unknown>) => string;
}

export function SelectionBanner({
  selectedCount,
  filteredCount,
  isAllFilteredSelected,
  pageSize,
  selectAllFiltered,
  t
}: SelectionBannerProps) {
  if (selectedCount === 0) return null;

  return (
    <div className="bg-primary-50 dark:bg-primary-950 px-8 py-3 border-b border-primary-100 dark:border-primary-800 flex items-center justify-between animate-in slide-in-from-top-2">
      <div className="flex items-center gap-3">
         <Info className="text-primary-600 dark:text-primary-400" size={16} />
         <Typography className="text-xs font-bold text-primary-900" variant="p">
           {selectedCount === filteredCount 
             ? t("payroll.selection.all_filtered_selected", { count: selectedCount, defaultValue: `All ${selectedCount} employees in this list are selected.` })
             : t("payroll.selection.count_selected", { count: selectedCount, defaultValue: `${selectedCount} employees selected.` })
           }
         </Typography>
      </div>
      {!isAllFilteredSelected && filteredCount > pageSize && (
        <Button 
          className="text-xs font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:underline decoration-2 underline-offset-4"
          onClick={selectAllFiltered}
        >
          {t("payroll.selection.select_all_filtered", { count: filteredCount, defaultValue: `Select all ${filteredCount} employees` })}
        </Button>
      )}
    </div>
  );
}
