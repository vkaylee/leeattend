import { Loader2 } from "lucide-react";

import { Typography } from "@/components/ui/typography";

export function GlobalLoader() {
  return (
    <div className="flex flex-col h-dvh w-full items-center justify-center bg-slate-50 dark:bg-slate-900 animate-in fade-in duration-500">
      <div className="relative flex items-center justify-center">
        <div className="absolute inset-0 bg-primary-500/20 blur-2xl rounded-full animate-pulse transform scale-150" />
        <div className="relative bg-white dark:bg-slate-800 p-5 rounded-[2rem] shadow-2xl border border-primary-100 dark:border-primary-800 flex items-center justify-center">
          <Loader2 className="w-10 h-10 text-primary-600 dark:text-primary-400 animate-spin" />
        </div>
      </div>
      <Typography className="mt-8 text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-[0.3em] animate-pulse opacity-80" variant="p">
        Initializing Organization...
      </Typography>
    </div>
  );
}
