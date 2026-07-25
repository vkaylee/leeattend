import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

/** ── Props ──────────────────────────────────────────────────────────── */

interface AddShiftTriggerProps {
  availableCount: number;
  /** Set to true so the trigger fills its container (rotation grid). */
  fullWidth?: boolean;
  onClick: () => void;
}

/** ── AddShiftTrigger ────────────────────────────────────────────────── */

/**
 * Dashed-border trigger button that opens the ShiftPickerModal for a
 * specific day. Disabled when all shifts are already assigned.
 */
export function AddShiftTrigger({
  availableCount,
  fullWidth,
  onClick,
}: AddShiftTriggerProps) {
  const { t } = useTranslation();
  const exhausted = availableCount === 0;

  return (
    <Button
      aria-label={t("patterns.form.add_shift")}
      className={cn(
        "!py-1.5 !px-3 rounded-xl text-[9px] font-bold uppercase tracking-tight",
        "border border-dashed",
        exhausted
          ? "border-slate-200 dark:border-slate-700 text-slate-350 dark:text-slate-600 cursor-not-allowed"
          : "border-primary-300 dark:border-primary-700 text-primary-500 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/50",
        fullWidth ? "w-full" : "w-36 flex-shrink-0",
      )}
      disabled={exhausted}
      icon={!exhausted && <Plus size={10} />}
      onClick={onClick}
      title={
        exhausted
          ? t("patterns.form.all_shifts_committed") || t("patterns.form.add_shift")
          : undefined
      }
      type="button"
      variant="ghost"
    >
      {exhausted ? (
        <span className="italic opacity-60">
          {t("patterns.form.all_shifts_committed") || t("status_off")}
        </span>
      ) : (
        `+ ${t("patterns.form.add_shift")}`
      )}
    </Button>
  );
}
