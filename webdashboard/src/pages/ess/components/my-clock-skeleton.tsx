import React from "react";

export function MyClockSkeleton() {
  return (
    <div className="space-y-10 animate-pulse pb-20">
      {/* Header Skeleton */}
      <div className="space-y-3 px-2">
        <div className="h-9 w-64 bg-slate-200 dark:bg-slate-800 rounded-2xl" />
        <div className="h-3.5 w-96 bg-slate-100 dark:bg-slate-800/60 rounded-full" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        {/* PunchCard Skeleton */}
        <div className="lg:col-span-5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-3xl sm:rounded-[40px] p-6 sm:p-10 flex flex-col items-center justify-between min-h-[460px]">
          <div className="w-24 h-24 rounded-[2rem] bg-slate-200 dark:bg-slate-800 mb-8" />
          <div className="space-y-3 w-full items-center flex flex-col mb-10">
            <div className="h-6 w-48 bg-slate-200 dark:bg-slate-800 rounded-xl" />
            <div className="h-3 w-32 bg-slate-100 dark:bg-slate-800/60 rounded-full" />
          </div>
          <div className="w-full h-14 rounded-2xl bg-slate-200 dark:bg-slate-800 mb-6" />
          <div className="h-3 w-36 bg-slate-100 dark:bg-slate-800/60 rounded-full" />
        </div>

        {/* HistoryTable Skeleton */}
        <div className="lg:col-span-7 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-3xl sm:rounded-[40px] overflow-hidden flex flex-col">
          <div className="px-8 py-6 border-b border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex items-center">
            <div className="w-5 h-5 bg-slate-200 dark:bg-slate-800 rounded-lg mr-3" />
            <div className="h-4 w-40 bg-slate-200 dark:bg-slate-800 rounded-lg" />
          </div>
          <div className="p-8 space-y-6 flex-1">
            <div className="flex justify-between border-b border-slate-100 dark:border-slate-800 pb-4">
              <div className="h-3 w-20 bg-slate-200 dark:bg-slate-800 rounded-full" />
              <div className="h-3 w-16 bg-slate-200 dark:bg-slate-800 rounded-full" />
              <div className="h-3 w-16 bg-slate-200 dark:bg-slate-800 rounded-full" />
            </div>
            {[1, 2, 3, 4].map((i) => (
              <div className="flex justify-between items-center py-4 border-b border-slate-50 dark:border-slate-800/50" key={i}>
                <div className="h-4 w-28 bg-slate-200 dark:bg-slate-800 rounded-lg" />
                <div className="h-4 w-16 bg-slate-150 dark:bg-slate-800/80 rounded-lg" />
                <div className="h-4 w-16 bg-slate-150 dark:bg-slate-800/80 rounded-lg" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
