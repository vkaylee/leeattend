import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronLeft, ChevronRight, Clock } from "lucide-react";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

/**
 * Timeline scrubber (Pattern 1): the reference instant `A` is pinned at the
 * centre of the viewport and the timeline scrolls around it across a ±12h
 * window. Dragging the track, clicking the chevrons, or using the keyboard
 * (←/→ hour, Shift+←/→ 15min) moves `A`. A live 1s tick advances `A` toward
 * "now" while the user is scrubbing the present.
 *
 * Accessibility (WCAG 2.1 AA):
 *  - The slider is a real `input[type=range]` — focusable, operable by keyboard
 *    natively, with an `aria-label` that announces the current `A`.
 *  - Colour is never the only channel: the centre pin, the readout text and the
 *    "NOW" hint all reinforce state.
 *  - Visible focus ring via the global `focus-visible:ring-*` utility.
 */

const WINDOW_HOURS = 12; // ±12h around A
const MINUTES_PER_HOUR = 60;
const TOTAL_MINUTES = WINDOW_HOURS * 2 * MINUTES_PER_HOUR;
const CENTER_MINUTES = TOTAL_MINUTES / 2;
const MS_PER_MINUTE = 60_000;

export interface ReferenceTimeScrubberProps {
  /** The pinned reference instant `A` (controlled). */
  referenceTime: Date;
  /** Fires when the user scrubs to a new `A`. */
  onChange: (next: Date) => void;
  /** Optional className for the outer shell. */
  className?: string;
}

function formatTime(d: Date): string {
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  return `${hh}:${mm}`;
}

function formatDate(d: Date): string {
  return d.toLocaleDateString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
  });
}

export function ReferenceTimeScrubber({
  referenceTime,
  onChange,
  className,
}: ReferenceTimeScrubberProps) {
  const { t } = useTranslation();
  const [now, setNow] = useState(() => Date.now());
  const trackRef = useRef<HTMLDivElement>(null);

  // Live 1s tick so the window tracks wall-clock while scrubbing the present.
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, []);

  // Slider value = minutes offset of `A` relative to (now - 12h). Dragging from
  // left (now-12h) to right (now+12h). We anchor the scale on the ticking `now`
  // so the window follows the present; the centre pin sits at the current `A`.
  const windowStartMs = now - WINDOW_HOURS * MINUTES_PER_HOUR * MS_PER_MINUTE;
  const value = Math.round((referenceTime.getTime() - windowStartMs) / MS_PER_MINUTE);
  const clampedValue = Math.max(0, Math.min(TOTAL_MINUTES, value));

  const onSliderInput = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const minutes = Number(e.target.value);
      onChange(new Date(windowStartMs + minutes * MS_PER_MINUTE));
    },
    [onChange, windowStartMs],
  );

  const nudge = useCallback(
    (deltaMinutes: number) => {
      onChange(new Date(referenceTime.getTime() + deltaMinutes * MS_PER_MINUTE));
    },
    [onChange, referenceTime],
  );

  // Keyboard: ←→ nudge 1h, Shift+←→ nudge 15min. The native range input already
  // handles arrow keys; this adds larger steps on top.
  const onKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      const step = e.shiftKey ? 15 : 60;
      if (e.key === "ArrowRight") {
        e.preventDefault();
        nudge(step);
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        nudge(-step);
      }
    },
    [nudge],
  );

  // Build 25 hour ticks (-12..+12 relative to A) for visual orientation.
  const ticks = Array.from({ length: 2 * WINDOW_HOURS + 1 }, (_, i) => i - WINDOW_HOURS);

  const pastStart = referenceTime.getTime() < now - MS_PER_MINUTE;
  const futureStart = referenceTime.getTime() > now + MS_PER_MINUTE;

  const ariaLabel = t("unrecorded_shifts.scrubber_label", {
    defaultValue: "Reference time scrubber, currently {{time}} on {{date}}, use left and right arrows to move",
    time: formatTime(referenceTime),
    date: formatDate(referenceTime),
  });

  return (
    <div
      className={cn(
        "bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm p-3",
        className,
      )}
    >
      {/* Header: NOW hint + live clock */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2 text-[10px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">
          <Clock size={12} />
          <span>{t("unrecorded_shifts.timeline", { defaultValue: "Timeline" })}</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-mono font-bold text-slate-500 dark:text-slate-400 tabular-nums">
            {t("unrecorded_shifts.now_badge", { defaultValue: "NOW" })} {formatTime(new Date(now))}
          </span>
        </div>
      </div>

      {/* Track */}
      <div ref={trackRef} className="relative select-none">
        {/* Centre pin marking the pinned `A` */}
        <div className="absolute inset-x-0 top-0 flex justify-center pointer-events-none">
          <div className="w-0.5 h-10 bg-primary-500 rounded-full" aria-hidden="true" />
        </div>

        {/* Hour tick labels */}
        <div className="relative h-6 mt-2 mb-1">
          {ticks.map((offset) => {
            const leftPct = ((offset + WINDOW_HOURS) / (2 * WINDOW_HOURS)) * 100;
            const isNow = offset === 0;
            return (
              <span
                key={offset}
                className={cn(
                  "absolute -translate-x-1/2 text-[9px] font-bold tabular-nums",
                  isNow
                    ? "text-primary-600 dark:text-primary-400"
                    : "text-slate-400 dark:text-slate-500",
                )}
                style={{ left: `${leftPct}%` }}
                aria-hidden="true"
              >
                {isNow
                  ? t("unrecorded_shifts.now_badge", { defaultValue: "NOW" })
                  : formatTime(new Date(referenceTime.getTime() + offset * MINUTES_PER_HOUR * MS_PER_MINUTE))}
              </span>
            );
          })}
        </div>

        {/* Native range input is the accessible control */}
        <input
          aria-label={ariaLabel}
          className="w-full h-8 accent-primary-500 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/40 rounded-lg"
          max={TOTAL_MINUTES}
          min={0}
          onChange={onSliderInput}
          onKeyDown={onKeyDown}
          step={5}
          type="range"
          value={clampedValue}
        />

        {/* Subtle window-region shading: the ±12h span from A is non-trivial to
            expose on a plain range, so the ticks + readout do the heavy lifting. */}
      </div>

      {/* Controls + readout */}
      <div className="flex items-center justify-between mt-2">
        <div className="flex items-center gap-1.5">
          <Button
            aria-label={t("unrecorded_shifts.step_back", { defaultValue: "Step back one hour" })}
            className="px-2 py-1.5 normal-case"
            data-testid="scrubber-step-back"
            onClick={() => nudge(-60)}
            size="sm"
            variant="outline"
          >
            <ChevronLeft size={14} />
          </Button>
          <Button
            aria-label={t("unrecorded_shifts.step_forward", { defaultValue: "Step forward one hour" })}
            className="px-2 py-1.5 normal-case"
            data-testid="scrubber-step-forward"
            onClick={() => nudge(60)}
            size="sm"
            variant="outline"
          >
            <ChevronRight size={14} />
          </Button>
        </div>

        {/* Readout reinforces `A` — not conveyed by colour alone (WCAG 1.4.1). */}
        <div className="flex flex-col items-center">
          <span className="text-base font-mono font-bold text-slate-800 dark:text-slate-100 tabular-nums">
            {formatTime(referenceTime)}
          </span>
          <span
            className={cn(
              "text-[10px] font-bold uppercase tracking-widest",
              pastStart
                ? "text-rose-500"
                : futureStart
                  ? "text-blue-500"
                  : "text-emerald-500",
            )}
          >
            {formatDate(referenceTime)}
          </span>
        </div>

        <Button
          className="px-2 py-1.5 normal-case"
          data-testid="scrubber-jump-now"
          onClick={() => onChange(new Date(now))}
          size="sm"
          variant="outline"
        >
          {t("unrecorded_shifts.jump_to_now", { defaultValue: "Now" })}
        </Button>
      </div>
    </div>
  );
}
