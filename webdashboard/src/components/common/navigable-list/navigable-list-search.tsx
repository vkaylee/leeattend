import { Search } from "lucide-react";
import React from "react";

import { cn } from "@/lib/utils";

import type { SearchProps } from "./types";

export function NavigableListSearchContent({
  searchValue,
  onSearchChange,
  searchPlaceholder,
  searchInputRef,
  listNav,
  searchTestId,
}: SearchProps) {
  if (searchValue === undefined) return null;

  return (
    <div className="relative group">
      <Search className="absolute left-6 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 transition-colors" size={18} />
      <input
        className="w-full bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] pl-16 pr-8 py-5 text-sm font-bold shadow-sm focus:ring-4 focus:ring-primary-500/10 focus:border-primary-200 dark:focus:border-primary-800 outline-none transition-all placeholder:text-slate-300 dark:placeholder:text-slate-700"
        onChange={(e) => onSearchChange?.(e.target.value)}
        onFocus={(e) => { e.target.select(); }}
        onKeyDown={listNav.onKeyDown}
        placeholder={searchPlaceholder}
        ref={searchInputRef}
        type="text"
        value={searchValue}
        {...(searchTestId ? { "data-testid": searchTestId } : {})}
      />
    </div>
  );
}

export function NavigableListHeaderWrapper({
  smartHeaderEnabled,
  smartHeaderRef,
  smartHeaderVisible,
  stickySearchRef,
  headerSlot,
  isFullPage,
  searchProps,
}: {
  headerSlot?: React.ReactNode;
  isFullPage: boolean;
  searchProps: SearchProps;
  smartHeaderEnabled: boolean;
  smartHeaderRef: React.Ref<HTMLDivElement>;
  smartHeaderVisible: boolean;
  stickySearchRef: React.Ref<HTMLDivElement>;
}) {
  return (
    <>
      {smartHeaderEnabled ? (
        <div
          className={cn(
            "fixed top-0 left-0 right-0 z-40 bg-white/80 dark:bg-slate-900/80 backdrop-blur-md border-b border-slate-100 dark:border-slate-800 transition-all duration-300",
            smartHeaderVisible ? "translate-y-0 opacity-100" : "-translate-y-full opacity-0"
          )}
          ref={smartHeaderRef}
        >
          <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between gap-6">
            <div className="flex-1 max-w-2xl">
              <NavigableListSearchContent {...searchProps} />
            </div>
            {headerSlot}
          </div>
        </div>
      ) : null}

      {isFullPage && !smartHeaderEnabled ? (
        <div className="sticky top-0 z-40 bg-slate-50/50 dark:bg-slate-900/50 backdrop-blur-sm pb-6 -mx-2 px-2" ref={stickySearchRef}>
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
            <div className="flex-1 max-w-2xl">
              <NavigableListSearchContent {...searchProps} />
            </div>
            {headerSlot}
          </div>
        </div>
      ) : null}
    </>
  );
}

export function SearchBarSlot({
  headerSlot,
  searchProps,
}: {
  headerSlot?: React.ReactNode;
  searchProps: SearchProps;
}) {
  return (
    <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
      <div className="flex-1 max-w-2xl">
        <NavigableListSearchContent {...searchProps} />
      </div>
      {headerSlot}
    </div>
  );
}
