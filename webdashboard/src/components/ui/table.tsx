import { forwardRef, type HTMLAttributes, type TdHTMLAttributes, type ThHTMLAttributes } from "react";

import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { cn } from "@/lib/utils";

export const Table = forwardRef<
  HTMLTableElement,
  HTMLAttributes<HTMLTableElement>
>(({ className, ...props }, ref) => {
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>();

  return (
    <div className="bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden w-full">
      <div
        className="overflow-x-auto w-full"
        onClickCapture={handleHorizontalDragClick}
        ref={setHorizontalDragScrollRef}
      >
        <table
          className={cn("w-full text-left border-collapse", className)}
          ref={ref}
          {...props}
        />
      </div>
    </div>
  );
});
Table.displayName = "Table";

export const TableHeader = forwardRef<
  HTMLTableSectionElement,
  HTMLAttributes<HTMLTableSectionElement>
>(({ className, ...props }, ref) => (
  <thead
    className={cn("bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800", className)}
    ref={ref}
    {...props}
  />
));
TableHeader.displayName = "TableHeader";

export const TableBody = forwardRef<
  HTMLTableSectionElement,
  HTMLAttributes<HTMLTableSectionElement>
>(({ className, ...props }, ref) => (
  <tbody
    className={cn("divide-y divide-slate-50 dark:divide-slate-800", className)}
    ref={ref}
    {...props}
  />
));
TableBody.displayName = "TableBody";

export const TableRow = forwardRef<
  HTMLTableRowElement,
  HTMLAttributes<HTMLTableRowElement>
>(({ className, ...props }, ref) => (
  <tr
    className={cn(
      "transition-colors hover:bg-slate-50/50 dark:hover:bg-slate-800/30 data-[state=selected]:bg-slate-100 dark:data-[state=selected]:bg-slate-800",
      className
    )}
    ref={ref}
    {...props}
  />
));
TableRow.displayName = "TableRow";

export const TableHead = forwardRef<
  HTMLTableCellElement,
  ThHTMLAttributes<HTMLTableCellElement>
>(({ className, ...props }, ref) => (
  <th
    className={cn(
      "px-8 py-4 text-xs font-semibold text-slate-700 dark:text-slate-300 uppercase tracking-wider first:pl-10 last:pr-10 align-middle",
      className
    )}
    ref={ref}
    {...props}
  />
));
TableHead.displayName = "TableHead";

export const TableCell = forwardRef<
  HTMLTableCellElement,
  TdHTMLAttributes<HTMLTableCellElement>
>(({ className, ...props }, ref) => (
  <td
    className={cn(
      "px-8 py-6 align-middle first:pl-10 last:pr-10",
      className
    )}
    ref={ref}
    {...props}
  />
));
TableCell.displayName = "TableCell";
