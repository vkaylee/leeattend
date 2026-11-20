import { memo } from "react";

import { getSegmentStyles, timeToMinutes } from "@/lib/shift-timeline-math";

import type { TimelineBarProps } from "./types";

export const TimelineBar = memo(function TimelineBar({
  breakEnd,
  breakStart,
  endMin,
  isCrossMidnight,
  isFixedBreak,
  shiftEnd,
  shiftStart,
  startMin,
}: TimelineBarProps) {
  const bStart = timeToMinutes(breakStart);
  const bEnd = timeToMinutes(breakEnd);
  const bCross = bEnd < bStart;

  return (
    <div className="relative h-14 bg-slate-200 dark:bg-slate-900 rounded-3xl overflow-hidden shadow-inner border border-slate-150 dark:border-slate-800">
      {/* Hour markers background */}
      <div className="absolute inset-0 flex justify-between px-2 pointer-events-none opacity-20">
        {Array.from({ length: 7 }).map((_, i) => (
          <div className="h-full w-px bg-slate-400 dark:bg-slate-500" key={i} />
        ))}
      </div>

      {/* Render Shift Duration */}
      {shiftStart && shiftEnd ? (
        !isCrossMidnight ? (
          <div
            className="absolute h-full bg-primary-500/20 dark:bg-primary-500/10 border-x border-primary-500/50 flex items-center justify-center"
            style={getSegmentStyles(startMin, endMin)}
          >
            <div className="h-[70%] w-full bg-primary-600 dark:bg-primary-500 rounded-2xl mx-1 shadow-sm opacity-90" />
          </div>
        ) : (
          <>
            {/* Segment 1: start to midnight */}
            <div
              className="absolute h-full bg-primary-500/20 dark:bg-primary-500/10 border-l border-primary-500/50 flex items-center justify-center"
              style={getSegmentStyles(startMin, 1440)}
            >
              <div className="h-[70%] w-full bg-primary-600 dark:bg-primary-500 rounded-l-2xl ml-1 shadow-sm opacity-90" />
            </div>
            {/* Segment 2: midnight to end */}
            <div
              className="absolute h-full bg-primary-500/20 dark:bg-primary-500/10 border-r border-primary-500/50 flex items-center justify-center"
              style={getSegmentStyles(0, endMin)}
            >
              <div className="h-[70%] w-full bg-primary-600 dark:bg-primary-500 rounded-r-2xl mr-1 shadow-sm opacity-90" />
            </div>
          </>
        )
      ) : null}

      {/* Render Fixed Break Segment */}
      {isFixedBreak && breakStart && breakEnd ? (
        !bCross ? (
          <div
            className="absolute h-full flex items-center justify-center z-10"
            style={getSegmentStyles(bStart, bEnd)}
          >
            <div className="h-[70%] w-full bg-amber-500 dark:bg-amber-600 rounded-2xl mx-0.5 shadow-sm border border-amber-400/30 flex items-center justify-center text-[9px] text-white font-bold">
              ☕
            </div>
          </div>
        ) : (
          <>
            <div
              className="absolute h-full flex items-center justify-center z-10"
              style={getSegmentStyles(bStart, 1440)}
            >
              <div className="h-[70%] w-full bg-amber-500 dark:bg-amber-600 rounded-l-2xl ml-0.5 shadow-sm flex items-center justify-center text-[9px] text-white font-bold">
                ☕
              </div>
            </div>
            <div
              className="absolute h-full flex items-center justify-center z-10"
              style={getSegmentStyles(0, bEnd)}
            >
              <div className="h-[70%] w-full bg-amber-500 dark:bg-amber-600 rounded-r-2xl mr-0.5 shadow-sm flex items-center justify-center text-[9px] text-white font-bold">
                ☕
              </div>
            </div>
          </>
        )
      ) : null}
    </div>
  );
});
