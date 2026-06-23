 
import React from "react";
import { useTranslation } from "react-i18next";

import { ShiftPickerModal } from "@/components/shifts/shift-picker-modal";
import Modal from "@/components/ui/modal";
import { PatternAssignmentForm } from "@/components/shifts/pattern-assignment-form";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsModalsProps {
  hook: ReturnType<typeof useShifts>;
}

export function ShiftsModals({ hook }: ShiftsModalsProps) {
  return (
    <>
      <ShiftPicker hook={hook} />
      <PatternAssignmentPicker hook={hook} />
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

function PatternAssignmentPicker({ hook }: ShiftsModalsProps) {
  const { t } = useTranslation();
  const {
    isPatternModalOpen,
    setIsPatternModalOpen,
    endDate,
    isForever,
    assignPatternMutation,
    handleAssign,
    patterns,
    selectedEmployee,
    selectedPatternId,
    setEndDate,
    setIsForever,
    setSelectedPatternId,
    setStartDate,
    startDate,
    shifts,
  } = hook;

  if (!selectedEmployee) return null;

  return (
    <Modal
      backdropClassName="bg-slate-900/5 dark:bg-slate-950/10 backdrop-blur-none"
      isOpen={isPatternModalOpen}
      onClose={() => {
        setIsPatternModalOpen(false);
      }}
      maxWidth="3xl"
      title={t("patterns.modal.assign_title")}
      subtitle={t("patterns.modal.assign_subtitle")}
    >
      <div className="pt-2">
        <PatternAssignmentForm
          endDate={endDate}
          isForever={isForever}
          isLoading={assignPatternMutation.isPending}
          onAssign={handleAssign}
          patterns={patterns}
          selectedEmployee={selectedEmployee}
          selectedPatternId={selectedPatternId}
          setEndDate={setEndDate}
          setIsForever={setIsForever}
          setSelectedPatternId={setSelectedPatternId}
          setStartDate={setStartDate}
          startDate={startDate}
          shifts={shifts}
        />
      </div>
    </Modal>
  );
}
