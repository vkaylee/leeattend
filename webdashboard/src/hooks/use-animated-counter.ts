import { useEffect, useRef, useState } from "react";

interface UseAnimatedCounterOptions {
  /** Delay before animation starts in ms (default: 0) */
  delay?: number;
  /** Animation duration in ms (default: 800) */
  duration?: number;
  /** Skip animation when false (e.g. data still loading) */
  enabled?: boolean;
  /** Format the number after counting (e.g. formatCurrency) */
  formatter?: (n: number) => string;
}

/** Ease-out exponential — fast start, smooth deceleration */
function easeOutExpo(t: number): number {
  return t >= 1 ? 1 : 1 - Math.pow(2, -10 * t);
}

/**
 * Animates a numeric value from its previous value to the target.
 * Uses requestAnimationFrame for smooth 60fps animation.
 * Respects `prefers-reduced-motion` — returns formatted target instantly.
 *
 * @example
 * const display = useAnimatedCounter(stats.total_employees, { duration: 800 });
 * <span>{display}</span>
 */
export function useAnimatedCounter(
  target: number,
  options: UseAnimatedCounterOptions = {},
): string {
  const {
    duration = 800,
    delay = 0,
    formatter,
    enabled = true,
  } = options;

  const [displayValue, setDisplayValue] = useState(target);
  const previousTarget = useRef(target);
  const rafId = useRef<number>(0);
  const timeoutId = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  // Check reduced motion preference once
  const prefersReducedMotion = useRef(
    typeof window !== "undefined"
      ? window.matchMedia("(prefers-reduced-motion: reduce)").matches
      : false,
  );

  useEffect(() => {
    // Skip animation if disabled or reduced motion preferred
    if (!enabled || prefersReducedMotion.current) {
      setDisplayValue(target);
      previousTarget.current = target;
      return;
    }

    const from = previousTarget.current;
    const to = target;
    previousTarget.current = target;

    // No animation needed if values are the same
    if (from === to) {
      setDisplayValue(to);
      return;
    }

    // Clean up any previous animation
    if (rafId.current) cancelAnimationFrame(rafId.current);
    if (timeoutId.current) clearTimeout(timeoutId.current);

    const startAnimation = () => {
      const startTime = performance.now();

      const animate = (currentTime: number) => {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / duration, 1);
        const easedProgress = easeOutExpo(progress);

        const current = from + (to - from) * easedProgress;
        setDisplayValue(Math.round(current));

        if (progress < 1) {
          rafId.current = requestAnimationFrame(animate);
        } else {
          setDisplayValue(to);
        }
      };

      rafId.current = requestAnimationFrame(animate);
    };

    if (delay > 0) {
      timeoutId.current = setTimeout(startAnimation, delay);
    } else {
      startAnimation();
    }

    return () => {
      if (rafId.current) cancelAnimationFrame(rafId.current);
      if (timeoutId.current) clearTimeout(timeoutId.current);
    };
  }, [target, duration, delay, enabled]);

  if (formatter) {
    return formatter(displayValue);
  }

  return String(displayValue);
}
