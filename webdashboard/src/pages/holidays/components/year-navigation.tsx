import { ChevronLeft, ChevronRight } from "lucide-react";

import Button from "@/components/ui/button";

interface YearNavigationProps {
    setYear: (year: number | ((y: number) => number)) => void;
    year: number;
}

export function YearNavigation({ year, setYear }: YearNavigationProps) {
    return (
        <div className="flex items-center justify-center gap-4">
            <Button className="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
                onClick={() => { setYear(y => y - 1); }}
                variant="ghost"
            >
                <ChevronLeft size={16} />
            </Button>
            <span className="text-xl font-bold text-slate-900 dark:text-white tabular-nums">
                {year}
            </span>
            <Button className="p-2 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors"
                onClick={() => { setYear(y => y + 1); }}
                variant="ghost"
            >
                <ChevronRight size={16} />
            </Button>
        </div>
    );
}
