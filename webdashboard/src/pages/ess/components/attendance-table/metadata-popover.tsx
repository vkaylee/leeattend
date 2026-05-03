import * as Popover from "@radix-ui/react-popover";
import { Activity, Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { AttendanceEntry } from "@/lib/api/types";

interface MetadataPopoverProps {
  entry: AttendanceEntry;
}

export function MetadataPopover({ entry }: MetadataPopoverProps) {
  const { t } = useTranslation();
  if (!entry.generation_metadata) return null;

  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button
          className="p-2.5 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90 opacity-0 group-hover:opacity-100"
          type="button"
        >
          <Info size={18} />
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          align="center"
          className="z-50 w-96 bg-slate-900 rounded-[2.5rem] shadow-premium-2xl border border-white/10 p-8 animate-in fade-in zoom-in-95 duration-500"
          side="left"
          sideOffset={16}
        >
          <div className="space-y-6">
            <div className="flex items-center gap-4 border-b border-white/10 pb-6">
              <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/10 flex items-center justify-center text-primary-400 shadow-inner border border-primary-500/20">
                <Activity size={20} />
              </div>
              <span className="text-[10px] font-bold uppercase tracking-[0.3em] text-white opacity-80">
                {t("ess.attendance.generation_info").toUpperCase()}
              </span>
            </div>
            <div className="bg-black/40 p-6 rounded-[1.5rem] border border-white/5 shadow-inner">
              <pre className="text-[11px] text-emerald-400 overflow-x-auto whitespace-pre-wrap font-mono leading-relaxed selection:bg-emerald-50 dark:bg-emerald-950/20">
                {JSON.stringify(entry.generation_metadata, null, 2)}
              </pre>
            </div>
          </div>
          <Popover.Arrow className="fill-slate-900" height={10} width={20} />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
