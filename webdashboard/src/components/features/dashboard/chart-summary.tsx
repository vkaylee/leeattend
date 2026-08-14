import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface ChartSummaryProps {
  /** Screen-reader–only when `visible` is false (default). */
  className?: string;
  text: string;
  visible?: boolean;
}

/**
 * A visible (or screen-reader–only) textual summary of what a chart shows, so the
 * chart's meaning never depends on hover-only tooltips.
 */
export function ChartSummary({ text, visible = false, className }: ChartSummaryProps) {
  if (!text) return null;
  return (
    <Typography
      className={cn(
        visible
          ? "text-[10px] font-medium text-slate-500 dark:text-slate-400 mb-3"
          : "sr-only",
        className,
      )}
      variant="p"
    >
      {text}
    </Typography>
  );
}
