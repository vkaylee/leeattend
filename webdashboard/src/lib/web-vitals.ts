import { onCLS, onFCP, onINP, onLCP, onTTFB } from "web-vitals";

interface VitalMetric {
  name: string;
  rating: "good" | "needs-improvement" | "poor";
  value: number;
}

function getRating(name: string, value: number): VitalMetric["rating"] {
  const thresholds: Partial<Record<string, [number, number]>> = {
    LCP: [2500, 4000],
    FCP: [1800, 3000],
    CLS: [0.1, 0.25],
    INP: [200, 500],
    TTFB: [800, 1800],
  };
  const [good, poor] = thresholds[name] ?? [0, Infinity];
  if (value <= good) return "good";
  if (value <= poor) return "needs-improvement";
  return "poor";
}

function reportMetric(metric: VitalMetric) {
  if (import.meta.env.DEV) {
    console.debug(`[Web Vitals] ${metric.name}: ${metric.value} (${metric.rating})`);
  }
}

export function trackWebVitals() {
  const report = (name: string, value: number) => {
    reportMetric({ name, value, rating: getRating(name, value) });
  };

  onCLS((m) => { report("CLS", m.value); });
  onFCP((m) => { report("FCP", m.value); });
  onINP((m) => { report("INP", m.value); });
  onLCP((m) => { report("LCP", m.value); });
  onTTFB((m) => { report("TTFB", m.value); });
}
