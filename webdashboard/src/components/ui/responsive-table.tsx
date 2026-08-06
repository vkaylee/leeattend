import { type ColumnDef } from "@tanstack/react-table";
import { Inbox } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import EmptyState from "@/components/ui/empty-state";
import SmartTable, {
  type ContextMenuItem,
  type EditableCellValue,
  type SmartTableColumnMeta,
} from "@/components/ui/smart-table";
import { cn } from "@/lib/utils";

// Re-export SmartTable types that consumers need for callbacks
export type { ContextMenuItem, EditableCellValue, SmartTableColumnMeta } from "@/components/ui/smart-table";

// ── Types ──────────────────────────────────────────────────────────

/**
 * Shared props used by both SmartTable mode and custom-desktop mode.
 */
interface ResponsiveTableBase<T> {
  /** Breakpoint to switch between table and card. Default: "lg" */
  breakpoint?: "md" | "lg" | "xl";
  /** Data array */
  data: T[];
  /** Empty state configuration (shown when !isLoading && data.length === 0) */
  emptyState?: {
    action?: React.ReactNode;
    description?: string;
    icon: React.ReactNode;
    title: string;
  };
  /** Stable mobile card key extractor */
  getCardKey?: (item: T, index: number) => React.Key;
  /** Loading state */
  isLoading?: boolean;
  /** Optional mobile footer (e.g. infinite-scroll sentinel) */
  mobileFooter?: React.ReactNode;
  /** Mobile card renderer — receives item and index */
  renderCard: (item: T, index: number) => React.ReactNode;
  /** Mobile skeleton card — renders N times while loading. Falls back to a default if not provided. */
  renderSkeletonCard?: () => React.ReactNode;
  /** Number of skeleton rows/cards during loading. Default: 5 */
  skeletonCount?: number;
}

/**
 * SmartTable mode: pass columns + getRowId to render a full-featured
 * SmartTable on desktop.
 */
interface ResponsiveTableSmartMode<T> extends ResponsiveTableBase<T> {
  /** Columns for desktop SmartTable view */
  columns: (ColumnDef<T> & { meta?: SmartTableColumnMeta })[];
  /** Context menu items (desktop-only, right-click on rows) */
  contextMenuItems?: (row: T) => ContextMenuItem[];
  /** Enable drag-to-select (desktop-only) */
  enableDragSelect?: boolean;
  /** Optional footer row rendered below all data rows (desktop-only) */
  footerRow?: React.ReactNode;
  /** Per-row className override on the desktop table (desktop-only) */
  getRowClassName?: (row: T, index: number) => string | undefined;
  /** Unique row ID extractor (required by SmartTable) */
  getRowId: (row: T) => string;
  /** Bulk cell save callback (desktop editing) */
  onBulkCellSave?: (changes: { rowId: string; columnId: string; value: EditableCellValue }[]) => Promise<void> | void;
  /** Single cell save callback (desktop editing) */
  onCellSave?: (rowId: string, columnId: string, value: EditableCellValue) => Promise<void> | void;
  /** Row click handler */
  onRowClick?: (row: T, e: React.MouseEvent) => void;
  /** Selection change callback (desktop-only) */
  onSelectionChange?: (selectedIds: Set<string>) => void;
  /** Custom desktop table renderer (mutually exclusive with columns) */
  renderDesktopTable?: never;
  /** Renders a full-width detail row below a desktop row (desktop-only) */
  renderExpandedRow?: (row: T) => React.ReactNode;
  /** Custom desktop skeleton row, rendered `skeletonCount` times while loading */
  renderSkeletonRow?: (index: number) => React.ReactNode;
  /** Currently selected row IDs (for checkbox selection on desktop) */
  selectedRowIds?: Set<string>;
  /** Number of sticky left columns (desktop-only) */
  stickyLeftColumns?: number;
  /** Number of sticky right columns (desktop-only) */
  stickyRightColumns?: number;
  /** Optional className for the desktop table wrapper */
  tableClassName?: string;
  /** Cell validation callback (desktop editing) */
  validateCell?: (rowId: string, columnId: string, value: EditableCellValue) => string | null | undefined;
}

/**
 * Custom-desktop mode: pass renderDesktopTable to render a completely custom
 * desktop table (e.g. pages that use manual <table> markup instead of SmartTable).
 */
interface ResponsiveTableCustomMode<T> extends ResponsiveTableBase<T> {
  /** SmartTable columns — do NOT pass when using renderDesktopTable */
  columns?: never;
  /** Custom desktop table renderer. Receives the filtered data. */
  renderDesktopTable: (data: T[]) => React.ReactNode;
}

export type ResponsiveTableProps<T> =
  | ResponsiveTableSmartMode<T>
  | ResponsiveTableCustomMode<T>;

// ── Breakpoint Visibility Map ──────────────────────────────────────

const BP: Record<NonNullable<ResponsiveTableProps<unknown>["breakpoint"]>, { desktop: string; mobile: string }> = {
  md: { desktop: "hidden md:block", mobile: "md:hidden" },
  lg: { desktop: "hidden lg:block", mobile: "lg:hidden" },
  xl: { desktop: "hidden xl:block", mobile: "xl:hidden" },
};

// ── Default Mobile Skeleton ────────────────────────────────────────

function DefaultSkeletonCard() {
  return (
    <div className="p-5 space-y-4 bg-white dark:bg-slate-900 rounded-2xl border border-slate-100 dark:border-slate-800 animate-pulse">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-xl bg-slate-200 dark:bg-slate-800" />
        <div className="space-y-2 flex-1">
          <div className="h-4 w-32 bg-slate-200 dark:bg-slate-800 rounded" />
          <div className="h-3 w-20 bg-slate-200 dark:bg-slate-800 rounded" />
        </div>
      </div>
      <div className="h-16 bg-slate-100 dark:bg-slate-800 rounded-xl" />
      <div className="h-4 w-24 bg-slate-200 dark:bg-slate-800 rounded" />
    </div>
  );
}

// ── Component ──────────────────────────────────────────────────────

// eslint-disable-next-line complexity
export default function ResponsiveTable<T>(
  props: ResponsiveTableProps<T>,
) {
  const {
    data,
    getCardKey,
    renderCard,
    isLoading = false,
    breakpoint: bp = "lg",
    emptyState,
    mobileFooter,
    skeletonCount = 5,
    renderSkeletonCard,
  } = props;
  const { t } = useTranslation();
  const { desktop, mobile } = BP[bp];

  const hasData = data.length > 0;
  const showEmpty = !isLoading && data.length === 0;

  // Determine mode: SmartTable or custom desktop renderer
  const isSmartMode = "columns" in props && props.columns;

  // ── Desktop View ─────────────────────────────────────────────────
  const desktopTable = isSmartMode ? (
    <div className={cn(desktop, "h-full w-full")}>
      <SmartTable<T>
        className={props.tableClassName}
        columns={props.columns}
        contextMenuItems={props.contextMenuItems}
        data={data}
        emptyMessage={emptyState?.title ?? t("attendance_logs.messages.no_data", "No data")}
        enableDragSelect={props.enableDragSelect}
        footerRow={props.footerRow}
        getRowClassName={props.getRowClassName}
        getRowId={props.getRowId}
        isLoading={isLoading}
        onBulkCellSave={props.onBulkCellSave}
        onCellSave={props.onCellSave}
        onRowClick={props.onRowClick}
        onSelectionChange={props.onSelectionChange}
        renderExpandedRow={props.renderExpandedRow}
        renderSkeletonRow={props.renderSkeletonRow}
        selectedRowIds={props.selectedRowIds}
        skeletonCount={skeletonCount}
        stickyLeftColumns={props.stickyLeftColumns}
        stickyRightColumns={props.stickyRightColumns}
        validateCell={props.validateCell}
      />
    </div>
  ) : (
    <div className={cn(desktop, "h-full w-full")}>
      {props.renderDesktopTable(data)}
    </div>
  );

  // ── Mobile: Card List ────────────────────────────────────────────
  const mobileCards = (
    <div className={cn(mobile, "space-y-4 pb-4")}>
      {/* Loading skeleton */}
      {isLoading && !data.length ? (
        <>
          {Array.from({ length: skeletonCount }).map((_, i) => (
            <React.Fragment key={`skel-${i}`}>
              {renderSkeletonCard ? renderSkeletonCard() : <DefaultSkeletonCard />}
            </React.Fragment>
          ))}
        </>
      ) : null}

      {/* Empty state */}
      {showEmpty ? (
        <EmptyState
          action={emptyState?.action}
          className="rounded-[2rem] mx-2"
          description={emptyState?.description}
          icon={emptyState?.icon ?? <Inbox className="h-10 w-10" />}
          title={emptyState?.title ?? t("attendance_logs.messages.no_data", "No data")}
        />
      ) : null}

      {/* Data cards */}
      {hasData ? data.map((item, index) => (
        <React.Fragment key={getCardKey?.(item, index) ?? `card-${index}`}>
          {renderCard(item, index)}
        </React.Fragment>
      )) : null}

      {/* Loading overlay when fetching more (pagination/infinite scroll) */}
      {isLoading && data.length > 0 ? (
        <div className="space-y-4">
          {Array.from({ length: 2 }).map((_, i) => (
            <React.Fragment key={`more-skel-${i}`}>
              {renderSkeletonCard ? renderSkeletonCard() : <DefaultSkeletonCard />}
            </React.Fragment>
          ))}
        </div>
      ) : null}

      {mobileFooter}
    </div>
  );

  return (
    <>
      {desktopTable}
      {mobileCards}
    </>
  );
}
