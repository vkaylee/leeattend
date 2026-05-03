import { ChevronLeft, ChevronRight } from "lucide-react";

interface YearNavigationProps {
    year: number;
    setYear: (year: number | ((y: number) => number)) => void;
}

export function YearNavigation({ year, setYear }: YearNavigationProps) {
    return (
        <div className="flex items-center justify-center gap-4">
            <button
                className="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
                onClick={() => { setYear(y => y - 1); }}
            >
                <ChevronLeft size={16} />
            </button>
            <span className="text-xl font-bold text-slate-900 dark:text-white tabular-nums">
                {year}
            </span>
            <button
                className="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
                onClick={() => { setYear(y => y + 1); }}
            >
                <ChevronRight size={16} />
            </button>
        </div>
    );
}
