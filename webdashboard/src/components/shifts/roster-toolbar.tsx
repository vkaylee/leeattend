import { memo } from "react";

import { Employee } from "@/lib/api/types";
import { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { MonthViewControls } from "./month-view-controls";
import {
  MonthOverlay,
  RosterNavHeader,
  WeekActionBar,
  WeekViewControls,
} from "./roster-toolbar-parts";

interface RosterToolbarProps {
  hook: ReturnType<typeof useShifts>;
  isCopying?: boolean;
  isMutating?: boolean;
  onApplyShift?: (shiftId: string | null) => void;
  onAssignSelected?: () => void;
  onCancelSelection?: () => void;
  onClearOverrides?: () => void;
  onClearWeekSelection?: () => void;
  onCopyMonth?: () => void;
  onOpenWeekShiftPicker?: (dates: string[]) => void;
  selectedCells?: Set<string>;
  selectedDays?: string[];
  selectedEmployeesMeta?: Employee[];
  viewMode: "week" | "month";
}

export const RosterToolbar = memo(function RosterToolbar(props: RosterToolbarProps) {
  const { hook, viewMode, selectedCells, selectedDays } = props;

  return (
    <div className="relative">
      <div className="flex flex-col gap-4 bg-white dark:bg-slate-900 p-5 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm">
        <RosterNavHeader hook={hook} isCopying={props.isCopying} onCopyMonth={props.onCopyMonth} viewMode={viewMode} />
        <div className="h-px bg-slate-100 dark:bg-slate-800 w-full" />
        <div className="flex flex-col gap-3.5 w-full animate-in fade-in duration-200">
          {viewMode === "month" ? <MonthViewControls hook={hook} selectedEmployeesMeta={props.selectedEmployeesMeta} /> : <WeekViewControls hook={hook} />}
        </div>
      </div>

      {viewMode === "month" && <MonthOverlay hook={hook} isMutating={props.isMutating} onApplyShift={props.onApplyShift} onAssignSelected={props.onAssignSelected} onCancelSelection={props.onCancelSelection} onClearOverrides={props.onClearOverrides} selectedCells={selectedCells} />}
      {viewMode === "week" && !!selectedDays && selectedDays.length > 0 && <WeekActionBar onClearWeekSelection={props.onClearWeekSelection} onOpenWeekShiftPicker={props.onOpenWeekShiftPicker} selectedDays={selectedDays} />}
    </div>
  );
});
