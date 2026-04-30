import React from "react";

import { Search, X } from "lucide-react";

import { cn } from "@/lib/utils";

interface SearchBoxProps {
    value: string;
    onChange: (value: string) => void;
    placeholder?: string;

    inputRef?: React.RefObject<HTMLInputElement | null>;
    onKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
    className?: string;
    "data-testid"?: string;
}

export default function SearchBox({
    value,
    onChange,
    placeholder = "Search...",

    inputRef,
    onKeyDown,
    className,
    "data-testid": testId = "search-box",
}: SearchBoxProps) {
    return (
        <div className={cn("relative", className)}>
            <Search
                size={18}
                className="absolute left-6 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500 pointer-events-none"
            />
            <input
                ref={inputRef}
                data-testid={testId}
 

                type="text"
                value={value}
                onChange={(e) => onChange(e.target.value)}
                onKeyDown={onKeyDown}
                onFocus={(e) => e.target.select()}
                placeholder={placeholder}
                className="w-full pl-14 pr-12 py-4 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl text-sm font-bold text-slate-700 dark:text-slate-200 placeholder:text-slate-400 dark:placeholder:text-slate-500 placeholder:font-medium focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 dark:focus:border-primary-700 transition-all shadow-sm tracking-tight"
            />
            {value && (
                <button
                    onClick={() => onChange("")}
                    className="absolute right-4 top-1/2 -translate-y-1/2 p-1.5 rounded-lg text-slate-400 hover:text-rose-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-all active:scale-90"
                >
                    <X size={16} />
                </button>
            )}
        </div>
    );
}
