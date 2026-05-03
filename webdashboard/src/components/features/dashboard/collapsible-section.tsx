import { ChevronDown } from "lucide-react";
import { useState, useRef, useEffect, type ReactNode } from "react";

import { cn } from "@/lib/utils";

interface CollapsibleSectionProps {
    children: ReactNode;
    className?: string;
    id: string;
    isCollapsed?: boolean;
    onToggle?: (id: string) => void;
    title: string;
}

export default function CollapsibleSection({
    id,
    title,
    children,
    isCollapsed = false,
    onToggle,
    className,
}: CollapsibleSectionProps) {
    const contentRef = useRef<HTMLDivElement>(null);
    const [height, setHeight] = useState<number | "auto">(isCollapsed ? 0 : "auto");

    useEffect(() => {
        if (!contentRef.current) return;

        if (isCollapsed) {
            // Animate to 0
            setHeight(contentRef.current.scrollHeight);
            requestAnimationFrame(() => {
                setHeight(0);
            });
        } else {
            // Animate to full height
            setHeight(contentRef.current.scrollHeight);
            const timeout = setTimeout(() => { setHeight("auto"); }, 300);
            return () => { clearTimeout(timeout); };
        }
    }, [isCollapsed]);

    return (
        <section className={cn("group/section", className)}>
            <button
                aria-controls={`section-${id}`}
                aria-expanded={!isCollapsed}
                className="flex items-center justify-between w-full px-4 py-3 rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
                onClick={() => onToggle?.(id)}
                type="button"
            >
                <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
                    {title}
                </h2>
                <ChevronDown
                    className={cn(
                        "text-slate-400 dark:text-slate-500 transition-transform duration-300",
                        isCollapsed && "-rotate-90"
                    )}
                    size={16}
                />
            </button>

            <div
                className="transition-[height] duration-300 ease-in-out"
                id={`section-${id}`}
                ref={contentRef}
                style={{
                    height: typeof height === "number" ? `${height}px` : "auto",
                    overflow: isCollapsed ? "hidden" : "visible",
                }}
            >
                <div className="pt-4">{children}</div>
            </div>
        </section>
    );
}
