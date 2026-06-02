 
import React from "react";

import { ShiftPickerModal } from "@/components/shifts/shift-picker-modal";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsModalsProps {
  hook: ReturnType<typeof useShifts>;
}

export function ShiftsModals({ hook }: ShiftsModalsProps) {
  return (
    <>
      <ShiftPicker hook={hook} />
    </>
  );
}

function ShiftPicker({ hook }: ShiftsModalsProps) {
  const {
    handleAssignBulk,
    isShiftPickerOpen,
    setIsShiftPickerOpen,
    setTargetDates,
    shifts,
    targetDates,
  } = hook;

  return (
    <ShiftPickerModal
      isOpen={isShiftPickerOpen}
      onClose={() => {
        setIsShiftPickerOpen(false);
        setTargetDates([]);
      }}
      onSelectShift={handleAssignBulk}
      shifts={shifts}
      targetDates={targetDates}
    />
  );
}
