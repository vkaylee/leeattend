import React from "react";
import { PayrollSegment } from "@/lib/api/types";
import { differenceInDays, parseISO, startOfDay, endOfDay } from "date-fns";
import { cn } from "@/lib/utils";

interface SegmentVisualizerProps {
  segments: PayrollSegment[];
  periodStart: string | Date;
  periodEnd: string | Date;
  className?: string;
}

const REASON_COLORS: Record<string, string> = {
  "Standard": "from-emerald-400 to-emerald-500",
  "Salary Change": "from-blue-500 to-indigo-600",
  "New Hire": "from-amber-400 to-orange-500",
  "Termination": "from-rose-400 to-rose-600",
  "Department Transfer": "from-indigo-400 to-teal-600",
};

export default function SegmentVisualizer({ segments, periodStart, periodEnd, className }: SegmentVisualizerProps) {
  if (!segments || segments.length === 0) return null;

  const start = startOfDay(new Date(periodStart));
  const end = endOfDay(new Date(periodEnd));
  const totalDays = differenceInDays(end, start) + 1;

  if (totalDays <= 0) return null;

  return (
    <div className={cn("w-full h-2.5 bg-slate-100 dark:bg-slate-700/80 rounded-full flex overflow-hidden shadow-inner border border-slate-200 dark:border-slate-700/50", className)}>
      {segments.map((seg, idx) => {
        const segStart = parseISO(seg.start_date);
        const segEnd = parseISO(seg.end_date);
        const days = differenceInDays(segEnd, segStart) + 1;
        const width = Math.max((days / totalDays) * 100, 1);

        const reasonKey = Object.keys(REASON_COLORS).find(key => seg.reason.includes(key));
        const colorClass = reasonKey ? REASON_COLORS[reasonKey] : "from-slate-400 to-slate-500";

        return (
          <div 
            key={idx}
            className={cn("h-full first:rounded-l-full last:rounded-r-full hover:brightness-110 hover:scale-y-110 transition-all bg-gradient-to-b relative group cursor-help", colorClass)}
            style={{ width: `${width}%` }}
            title={`${seg.reason}: ${seg.start_date} - ${seg.end_date}`}
          />
        );
      })}
    </div>
  );
}
