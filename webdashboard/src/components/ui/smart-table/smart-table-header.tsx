/* eslint-disable complexity */
import {
  flexRender,
  type Table,
  type Column,
  type HeaderGroup,
  type Header,
} from "@tanstack/react-table";
import { ChevronUp, ChevronDown } from "lucide-react";
import React, { memo } from "react";

import { cn } from "@/lib/utils";
import type { TableDensity } from "../table-density-toggle";

export interface SmartTableHeaderProps<T> {
  density?: TableDensity;
  getPinStyles: (column: Column<T>) => React.CSSProperties;
  table: Table<T>;
}
function SmartTableHeaderInner<T>({ table, getPinStyles, density = "normal" }: SmartTableHeaderProps<T>) {
  return (
    <thead className="sticky top-0 z-20 bg-white/95 dark:bg-slate-900/95 backdrop-blur border-b border-slate-100 dark:border-slate-800 shadow-sm">
      {table.getHeaderGroups().map((headerGroup: HeaderGroup<T>) => (
        <tr key={headerGroup.id}>

          {headerGroup.headers.map((header: Header<T, unknown>) => {
            const isPinned = header.column.getIsPinned();
            const isLastPinnedLeft = isPinned === "left" && header.column.getIsLastColumn("left");
            const isFirstPinnedRight = isPinned === "right" && header.column.getIsFirstColumn("right");
            const meta = header.column.columnDef.meta;

            return (
              <th
                className={cn(
                  "group relative font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest bg-slate-50/50 dark:bg-slate-900/50 transition-colors border-r border-b border-slate-100 dark:border-slate-800 last:border-r-0 select-none",
                  density === "compact"
                    ? "px-3 py-2 text-[9px]"
                    : density === "comfortable"
                      ? "px-6 py-4 text-[10px]"
                      : "px-5 py-3 text-[10px]",
                  isPinned && "bg-slate-100/60 dark:bg-slate-900",
                  isLastPinnedLeft && "border-r border-r-slate-200 dark:border-r-slate-700 shadow-[2px_0_5px_-2px_rgba(0,0,0,0.1)]",
                  isFirstPinnedRight && "border-l border-l-slate-200 dark:border-l-slate-700 shadow-[-2px_0_5px_-2px_rgba(0,0,0,0.1)]",
                  meta?.headerClassName
                )}
                key={header.id}
                style={{
                  width: header.getSize(),
                  ...getPinStyles(header.column),
                }}
              >
                {header.isPlaceholder ? null : header.column.getCanSort() ? (
                  <button
                    className="flex items-center gap-1.5 cursor-pointer font-bold uppercase tracking-widest text-[10px] text-slate-500 dark:text-slate-400 bg-transparent border-none p-0 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary-500 rounded"
                    onClick={header.column.getToggleSortingHandler()}
                    type="button"
                  >
                    {flexRender(header.column.columnDef.header, header.getContext())}
                    {{
                      asc: <ChevronUp className="h-3 w-3 text-primary-500" />,
                      desc: <ChevronDown className="h-3 w-3 text-primary-500" />,
                    }[header.column.getIsSorted() as string] ?? null}
                  </button>
                ) : (
                  <div className="flex items-center gap-1.5">
                    {flexRender(header.column.columnDef.header, header.getContext())}
                  </div>
                )}

                {/* Resizer handle */}
                {header.column.getCanResize() && (
                  /* eslint-disable-next-line jsx-a11y/no-static-element-interactions */
                  <div
                    className={cn(
                      "absolute right-0 top-0 h-full w-1 cursor-col-resize select-none bg-slate-200 dark:bg-slate-700 opacity-0 group-hover:opacity-100 transition-opacity",
                      header.column.getIsResizing() && "bg-primary-500 opacity-100 w-1.5"
                    )}
                    onMouseDown={header.getResizeHandler()}
                    onTouchStart={header.getResizeHandler()}
                  />
                )}
              </th>
            );
          })}
        </tr>
      ))}
    </thead>
  );
}

export const SmartTableHeader = memo(SmartTableHeaderInner) as typeof SmartTableHeaderInner;
