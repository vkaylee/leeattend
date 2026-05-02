import { Search, X } from "lucide-react";
import React from "react";

import { cn } from "@/lib/utils";

interface SearchBoxProps {
    className?: string;
    "data-testid"?: string;
    inputRef?: React.RefObject<HTMLInputElement | null>;

    onChange: (value: string) => void;
    onKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
    placeholder?: string;
    value: string;
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
                className="absolute left-6 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500 pointer-events-none"
                size={18}
            />
            <input
                className="w-full pl-14 pr-12 py-4 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl text-sm font-bold text-slate-700 dark:text-slate-200 placeholder:text-slate-400 dark:placeholder:text-slate-500 placeholder:font-medium focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 dark:focus:border-primary-700 transition-all shadow-sm tracking-tight"
                data-testid={testId}
 

                onChange={(e) => { onChange(e.target.value); }}
                onFocus={(e) => { e.target.select(); }}
                onKeyDown={onKeyDown}
                placeholder={placeholder}
                ref={inputRef}
                type="text"
                value={value}
            />
            {!!value && (
                <button
                    className="absolute right-4 top-1/2 -translate-y-1/2 p-1.5 rounded-lg text-slate-400 hover:text-rose-500 hover:bg-slate-100 dark:hover:bg-slate-800 transition-all active:scale-90"
                    onClick={() => { onChange(""); }}
                >
                    <X size={16} />
                </button>
              )}
        </div>
    );
}
