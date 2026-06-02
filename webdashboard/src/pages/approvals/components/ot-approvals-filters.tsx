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
              "uppercase tracking-wider text-[10px]",
              sourceFilter === filter
                ? "bg-primary-600 text-white shadow-md shadow-primary-100/40 dark:shadow-none"
                : "bg-slate-100/80 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300 hover:bg-slate-200/80 dark:hover:bg-slate-700/80 border-transparent shadow-none"
            )}
            key={filter}
            onClick={() => {
              setSourceFilter(filter);
            }}
            size="sm"
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

