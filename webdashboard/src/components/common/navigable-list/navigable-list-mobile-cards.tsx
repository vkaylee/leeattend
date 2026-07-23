import type { TFunction } from "i18next";

import { Loader2 } from "lucide-react";
import React from "react";

import { NavigableListMobileSkeleton } from "./navigable-list-skeleton";

export function NavigableListMobileCards<T>({
  emptyMessage,
  isFetchingNextPage,
  isLoading,
  itemKey,
  items,
  loadMoreRef,
  renderMobileCard,
  skeletonRows,
  t,
}: {
  emptyMessage?: string;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  itemKey: (item: T) => string;
  items: T[];
  loadMoreRef: React.Ref<HTMLDivElement>;
  renderMobileCard: (item: T, index: number) => React.ReactNode;
  skeletonRows: number;
  t: TFunction;
}) {
  if (isLoading && items.length === 0) {
    return <NavigableListMobileSkeleton rows={skeletonRows} />;
  }

  if (items.length === 0) {
    return (
      <div className="rounded-2xl border border-dashed border-slate-200 bg-white px-6 py-20 text-center text-[10px] font-bold uppercase tracking-widest text-slate-400 dark:border-slate-800 dark:bg-slate-900">
        {emptyMessage ?? t("common.no_data")}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {items.map((item, index) => (
        <React.Fragment key={itemKey(item)}>{renderMobileCard(item, index)}</React.Fragment>
      ))}
      <div className="flex min-h-10 items-center justify-center" ref={loadMoreRef}>
        {isFetchingNextPage ? <Loader2 className="animate-spin text-slate-400" size={18} /> : null}
      </div>
    </div>
  );
}
