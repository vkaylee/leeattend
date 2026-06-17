import React from "react";

import { cn } from "@/lib/utils";

export const Table = React.forwardRef<
  HTMLTableElement,
  React.HTMLAttributes<HTMLTableElement>
>(({ className, ...props }, ref) => (
  <div className="bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden w-full">
    <div className="overflow-x-auto w-full">
      <table
        className={cn("w-full text-left border-collapse", className)}
        ref={ref}
        {...props}
      />
    </div>
  </div>
));
Table.displayName = "Table";

export const TableHeader = React.forwardRef<
  HTMLTableSectionElement,
  React.HTMLAttributes<HTMLTableSectionElement>
>(({ className, ...props }, ref) => (
  <thead
    className={cn("bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800", className)}
    ref={ref}
    {...props}
  />
));
TableHeader.displayName = "TableHeader";

export const TableBody = React.forwardRef<
  HTMLTableSectionElement,
  React.HTMLAttributes<HTMLTableSectionElement>
>(({ className, ...props }, ref) => (
  <tbody
    className={cn("divide-y divide-slate-50 dark:divide-slate-800", className)}
    ref={ref}
    {...props}
  />
));
TableBody.displayName = "TableBody";

export const TableRow = React.forwardRef<
  HTMLTableRowElement,
  React.HTMLAttributes<HTMLTableRowElement>
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

export const TableHead = React.forwardRef<
  HTMLTableCellElement,
  React.ThHTMLAttributes<HTMLTableCellElement>
>(({ className, ...props }, ref) => (
  <th
    className={cn(
      "px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] first:pl-10 last:pr-10 align-middle",
      className
    )}
    ref={ref}
    {...props}
  />
));
TableHead.displayName = "TableHead";

export const TableCell = React.forwardRef<
  HTMLTableCellElement,
  React.TdHTMLAttributes<HTMLTableCellElement>
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
