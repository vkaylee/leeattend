import { memo } from "react";

import type { ShiftSegment } from "@/lib/shift-card-metrics";

import { cn } from "@/lib/utils";

const MINUTES_PER_DAY = 1440;
const AXIS_HOURS = [0, 6, 12, 18] as const;

interface ShiftCardTimelineProps {
  /** Tailwind background class for work blocks, so the card accent stays in sync. */
  accentClassName: string;
  segments: ShiftSegment[];
}

/**
 * Compact 24h strip showing when a shift actually runs. Purely decorative — the
 * same information is available as text in the card, so it is hidden from
 * assistive tech (WCAG 1.1.1: no information conveyed by the graphic alone).
 */
export const ShiftCardTimeline = memo(function ShiftCardTimeline({
  accentClassName,
  segments,
}: ShiftCardTimelineProps) {
  return (
    <div
      aria-hidden="true"
      className="relative h-2 w-full overflow-hidden rounded-full bg-slate-200/80 dark:bg-slate-700/50"
    >
      {/* 6-hour tick marks for temporal orientation */}
      {AXIS_HOURS.slice(1).map((hour) => (
        <span
          className="absolute top-0 bottom-0 w-px bg-white/70 dark:bg-slate-900/60"
          key={hour}
          style={{ left: `${String((hour / 24) * 100)}%` }}
        />
      ))}

      {segments.map((segment, index) => (
        <span
          className={cn(
            "absolute top-0 bottom-0",
            segment.kind === "break"
              ? "z-10 bg-amber-400 dark:bg-amber-500"
              : cn("rounded-full", accentClassName),
          )}
          key={`${segment.kind}-${String(segment.startMin)}-${String(index)}`}
          style={{
            left: `${String((segment.startMin / MINUTES_PER_DAY) * 100)}%`,
            width: `${String(((segment.endMin - segment.startMin) / MINUTES_PER_DAY) * 100)}%`,
          }}
        />
      ))}
    </div>
  );
});
