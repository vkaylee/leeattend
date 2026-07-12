import * as Popover from "@radix-ui/react-popover";
import { useQuery } from "@tanstack/react-query";
import { Check, ChevronDown } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { fetchShifts, getCurrentWorkspaceId  } from "@/lib/api";
import { cn } from "@/lib/utils";

/**
 * Multi-select shift-type filter built on Radix Popover (the existing `Combobox`
 * is single-select only). Options come from the shift catalogue via `fetchShifts`,
 * de-duplicated by name. Selected names are sent to the backend as `shift_type`.
 *
 * Uses the project's <Button> component everywhere for WCAG-compliant focus
 * rings (never a raw <button>).
 */
export function ShiftTypePicker({
  selected,
  onChange,
}: {
  selected: string[];
  onChange: (names: string[]) => void;
}) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [open, setOpen] = useState(false);

  const { data: shifts = [] } = useQuery({
    queryKey: ["shifts-catalogue", wsId],
    queryFn: () => fetchShifts(wsId!),
    enabled: !!wsId,
    staleTime: 5 * 60 * 1000,
  });

  // De-duplicated by name, sorted, active shifts first.
  const options = useMemo(() => {
    const byName = new Map<string, { name: string; active: boolean }>();
    for (const s of shifts) {
      const existing = byName.get(s.name);
      if (existing?.active) continue;
      byName.set(s.name, { name: s.name, active: s.is_active });
    }
    return Array.from(byName.values()).sort((a, b) => {
      if (a.active !== b.active) return a.active ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
  }, [shifts]);

  const toggle = (name: string) => {
    onChange(
      selected.includes(name)
        ? selected.filter((n) => n !== name)
        : [...selected, name],
    );
  };

  return (
    <Popover.Root onOpenChange={setOpen} open={open}>
      <Popover.Trigger asChild>
        <Button
          aria-haspopup="listbox"
          aria-label={t("unrecorded_shifts.filter_shift_type")}
          className="px-3 py-2 justify-between normal-case"
          id="unrecorded-shift-type-picker"
          variant="outline"
        >
          <span className="flex items-center gap-2">
            {t("unrecorded_shifts.filter_shift_type")}
            {selected.length > 0 && (
              <span className="px-1.5 py-0.5 bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300 rounded-md text-[9px]">
                {selected.length}
              </span>
            )}
          </span>
          <ChevronDown
            className={cn("size-3.5 transition-transform", open && "rotate-180")}
          />
        </Button>
      </Popover.Trigger>

      <Popover.Portal>
        <Popover.Content
          align="start"
          className="z-50 w-64 max-h-80 overflow-y-auto rounded-xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-2 shadow-lg animate-in fade-in zoom-in-95"
          sideOffset={6}
        >
          {options.length === 0 ? (
            <p className="px-3 py-2 text-xs text-slate-400">
              {t("unrecorded_shifts.filter_shift_type_empty", {
                defaultValue: "No shift types",
              })}
            </p>
          ) : (
            options.map((opt) => {
              const checked = selected.includes(opt.name);
              return (
                <Button
                  aria-selected={checked}
                  className={cn(
                    "w-full justify-start gap-3 px-3 py-2 rounded-lg text-xs font-semibold normal-case",
                    checked && "bg-primary-50 dark:bg-primary-950/40 text-primary-700 dark:text-primary-300",
                  )}
                  key={opt.name}
                  onClick={() => { toggle(opt.name); }}
                  role="option"
                  variant="ghost"
                >
                  <span
                    className={cn(
                      "size-4 rounded border flex items-center justify-center shrink-0",
                      checked
                        ? "bg-primary-600 border-primary-600 text-white"
                        : "border-slate-300 dark:border-slate-600",
                    )}
                  >
                    {!!checked && <Check className="size-3" />}
                  </span>
                  {!opt.active && (
                    <span className="text-[9px] uppercase text-slate-400 tracking-wider">
                      {t("unrecorded_shifts.filter_shift_inactive", {
                        defaultValue: "inactive",
                      })}
                    </span>
                  )}
                  {opt.name}
                </Button>
              );
            })
          )}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
