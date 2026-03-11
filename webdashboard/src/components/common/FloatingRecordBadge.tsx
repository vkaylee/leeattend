import React from "react";
import { cn } from "@/lib/utils";

interface FloatingRecordBadgeProps {
    loaded: number;
    total: number;
    label?: string;
    /** Color variant: "default" (slate/primary) or "amber" */
    variant?: "default" | "amber";
}

const FloatingRecordBadge = React.memo(function FloatingRecordBadge({
    loaded,
    total,
    label = "records",
    variant = "default",
}: FloatingRecordBadgeProps) {
    const isAmber = variant === "amber";

    return (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div
                className={cn(
                    "inline-flex items-center gap-2 backdrop-blur-lg rounded-full px-5 py-2.5 shadow-2xl shadow-black/10",
                    isAmber
                        ? "bg-white/95 dark:bg-slate-900/95 border border-amber-200 dark:border-amber-800"
                        : "bg-white/95 dark:bg-slate-900/95 border border-slate-200 dark:border-slate-700"
                )}
            >
                <span
                    className={cn(
                        "text-[10px] font-bold uppercase tracking-[0.15em]",
                        isAmber
                            ? "text-amber-600 dark:text-amber-400"
                            : "text-slate-500 dark:text-slate-400"
                    )}
                >
                    Showing
                </span>

                <span
                    className={cn(
                        "inline-flex items-center justify-center min-w-[2rem] px-2 py-0.5 rounded-full text-xs font-bold tabular-nums",
                        isAmber
                            ? "bg-amber-50 dark:bg-amber-950 text-amber-700 dark:text-amber-300"
                            : "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400"
                    )}
                >
                    {loaded}
                </span>

                <span
                    className={cn(
                        "text-[10px] font-bold",
                        isAmber
                            ? "text-amber-400 dark:text-amber-600"
                            : "text-slate-400 dark:text-slate-500"
                    )}
                >
                    /
                </span>

                <span
                    className={cn(
                        "inline-flex items-center justify-center min-w-[2rem] px-2 py-0.5 rounded-full text-xs font-bold tabular-nums",
                        isAmber
                            ? "bg-amber-100 dark:bg-amber-900 text-amber-800 dark:text-amber-200"
                            : "bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-200"
                    )}
                >
                    {total}
                </span>

                <span
                    className={cn(
                        "text-[10px] font-bold uppercase tracking-[0.15em]",
                        isAmber
                            ? "text-amber-600 dark:text-amber-400"
                            : "text-slate-500 dark:text-slate-400"
                    )}
                >
                    {label}
                </span>
            </div>
        </div>
    );
});

export default FloatingRecordBadge;
