import { Filter } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

export type SourceFilter = "all" | "paper_form" | "self_service";

interface OtApprovalsFiltersProps {
  setSourceFilter: (filter: SourceFilter) => void;
  sourceFilter: SourceFilter;
}

export function OtApprovalsFilters({
  sourceFilter,
  setSourceFilter,
}: OtApprovalsFiltersProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-4">
      <Filter className="text-slate-400 dark:text-slate-500" size={16} />
      <div className="flex gap-2">
        {(["all", "paper_form", "self_service"] as SourceFilter[]).map((filter) => (
          <Button
            className={cn(
              "h-auto px-4 py-2 rounded-xl text-[10px] uppercase tracking-wider",
              sourceFilter === filter
                ? "bg-primary-600 text-white shadow-lg"
                : "bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-600 border-transparent shadow-none"
            )}
            key={filter}
            onClick={() => {
              setSourceFilter(filter);
            }}
            variant={sourceFilter === filter ? "primary" : "outline"}
          >
            {filter === "all"
              ? t("common.all")
              : filter === "paper_form"
              ? t("approvals.filter.paper_forms")
              : t("approvals.filter.self_service")}
          </Button>
        ))}
      </div>
    </div>
  );
}

