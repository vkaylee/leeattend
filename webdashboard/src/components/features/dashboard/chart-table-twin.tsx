import { Table2 } from "lucide-react";
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";

import PaginationControls from "@/components/common/pagination-controls";
import SearchBox from "@/components/common/search-box";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useChartTable } from "@/hooks/use-chart-table";
import { cn } from "@/lib/utils";

export interface ChartTableColumn<T> {
  /** Header label key or literal text (prefers a translation key lookup first). */
  header: string;
  key: keyof T;
  /** Right-align numeric cells. */
  numeric?: boolean;
  /** Render a cell (defaults to String(value)). */
  render?: (row: T) => React.ReactNode;
}

interface ChartTableTwinProps<T extends object> {
  caption: string;
  columns: ChartTableColumn<T>[];
  /** Unique identifier prefix for the toggle's aria-controls wiring. */
  id?: string;
  rows: T[];
  /** Keys searched by the free-text box. */
  searchableKeys?: (keyof T)[];
  /** Toggle label (defaults to a translated "Table view"). */
  toggleLabel?: string;
}

/**
 * The universal accessible "table view" twin required alongside every chart.
 * Provides free-text search + client-side pagination so the full result set is
 * reachable regardless of chart Top-N truncation.
 */
export function ChartTableTwin<T extends object>({
  rows,
  columns,
  caption,
  searchableKeys = [],
  toggleLabel,
  id,
}: ChartTableTwinProps<T>) {
  const { t } = useTranslation();
  const autoId = useId();
  const baseId = id ?? `chart-table-${autoId}`;
  const [open, setOpen] = useState(false);
  const table = useChartTable(rows, { searchableKeys, pageSize: 10 });

  const headerText = (col: ChartTableColumn<T>): string => {
    const translated = t(col.header);
    return translated === col.header ? col.header : translated;
  };

  return (
    <div className="mt-6">
      <Button
        aria-controls={open ? `${baseId}-panel` : undefined}
        aria-expanded={open}
        className="inline-flex items-center gap-2 text-[9px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 h-auto px-0 shadow-none border border-transparent"
        id={`${baseId}-toggle`}
        onClick={() => { setOpen((v) => !v); }}
        type="button"
        variant="ghost"
      >
        <Table2 size={14} />
        {toggleLabel ?? t("dashboard.hr.table_view")}
      </Button>

      {open ? (
        <div
          aria-labelledby={`${baseId}-toggle`}
          className="mt-4 rounded-2xl border border-slate-100 dark:border-slate-800 overflow-hidden"
          id={`${baseId}-panel`}
          role="region"
        >
          <div className="p-3 border-b border-slate-100 dark:border-slate-800">
            <SearchBox
              onChange={table.setQuery}
              placeholder={t("common.search")}
              value={table.query}
            />
          </div>

          <div className="overflow-x-auto max-h-[360px]">
            <table className="w-full text-left text-[11px]">
              <caption className="sr-only">{caption}</caption>
              <thead className="sticky top-0 bg-slate-50 dark:bg-slate-800/80 text-slate-500 dark:text-slate-400 uppercase text-[9px] tracking-widest">
                <tr>
                  {columns.map((col) => (
                    <th
                      className={cn(
                        "px-4 py-3 font-bold whitespace-nowrap",
                        col.numeric && "text-right",
                      )}
                      key={String(col.key)}
                      scope="col"
                    >
                      {headerText(col)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                {table.pageRows.length === 0 ? (
                  <tr>
                    <td className="px-4 py-8 text-center text-slate-400 dark:text-slate-500" colSpan={columns.length}>
                      <Typography className="text-[10px] font-bold uppercase tracking-widest" variant="p">
                        {t("common.no_data")}
                      </Typography>
                    </td>
                  </tr>
                ) : (
                  table.pageRows.map((row, index) => (
                    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/40" key={index}>
                      {columns.map((col) => (
                        <td
                          className={cn("px-4 py-2.5 whitespace-nowrap", col.numeric && "text-right font-mono")}
                          key={String(col.key)}
                        >
                          {col.render ? col.render(row) : String(row[col.key] ?? "")}
                        </td>
                      ))}
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          <PaginationControls
            currentPage={table.page}
            onPageChange={table.setPage}
            pageSize={table.pageSize}
            totalItems={table.totalItems}
            totalPages={table.totalPages}
          />
        </div>
      ) : null}
    </div>
  );
}
