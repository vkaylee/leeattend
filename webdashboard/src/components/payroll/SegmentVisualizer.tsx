import React from "react";
import { PayrollSegment } from "@/lib/api/types";
import { differenceInDays, parseISO, startOfDay, endOfDay } from "date-fns";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface SegmentVisualizerProps {
  segments: PayrollSegment[];
  periodStart: string | Date;
  periodEnd: string | Date;
  className?: string;
}

const REASON_COLORS: Record<string, string> = {
  "Standard": "bg-emerald-400",
  "Salary Change": "bg-blue-500",
  "New Hire": "bg-amber-400",
  "Termination": "bg-rose-400",
  "Department Transfer": "bg-indigo-400",
};

export default function SegmentVisualizer({ segments, periodStart, periodEnd, className }: SegmentVisualizerProps) {
  if (!segments || segments.length === 0) return null;

  const start = startOfDay(new Date(periodStart));
  const end = endOfDay(new Date(periodEnd));
  const totalDays = differenceInDays(end, start) + 1;

  if (totalDays <= 0) return null;

  return (
    <div className={cn("w-full h-2 bg-slate-100 rounded-full flex overflow-hidden", className)}>
      {segments.map((seg, idx) => {
        const segStart = parseISO(seg.start_date);
        const segEnd = parseISO(seg.end_date);
        const days = differenceInDays(segEnd, segStart) + 1;
        const width = Math.max((days / totalDays) * 100, 1); // Min 1% visibility

        const colorClass = Object.keys(REASON_COLORS).find(key => seg.reason.includes(key)) 
          ? REASON_COLORS[Object.keys(REASON_COLORS).find(key => seg.reason.includes(key))!]
          : "bg-slate-400";

        return (
          <div 
            key={idx}
            className={cn("h-full first:rounded-l-full last:rounded-r-full hover:opacity-80 transition-opacity relative group", colorClass)}
            style={{ width: `${width}%` }}
            title={`${seg.reason}: ${seg.start_date} - ${seg.end_date}`}
          >
             {/* Tooltip logic usually handled by parent or library, simplistic native title used here */}
          </div>
        );
      })}
    </div>
  );
}
