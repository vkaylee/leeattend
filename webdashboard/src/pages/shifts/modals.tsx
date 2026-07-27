 
import React from "react";
import { useTranslation } from "react-i18next";

import { ComplianceWarningModal } from "@/components/shifts/compliance-warning-modal";
import { PatternAssignmentForm } from "@/components/shifts/pattern-assignment-form";
import { ShiftPickerModal } from "@/components/shifts/shift-picker-modal";
import Modal from "@/components/ui/modal";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsModalsProps {
  hook: ReturnType<typeof useShifts>;
}

export function ShiftsModals({ hook }: ShiftsModalsProps) {
  return (
    <>
      <ShiftPicker hook={hook} />
      <PatternAssignmentPicker hook={hook} />
      <ComplianceWarningModal
        isLoading={hook.isComplianceResolving}
        isOpen={hook.complianceModalOpen}
        onClose={() => { hook.setComplianceModalOpen(false); }}
        onConfirm={(reason) => { void hook.onComplianceOverride(reason); }}
        violations={hook.complianceViolations}
      />
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
      maxWidth="3xl"
      onClose={() => {
        setIsPatternModalOpen(false);
      }}
      subtitle={t("patterns.modal.assign_subtitle")}
      title={t("patterns.modal.assign_title")}
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
          shifts={shifts}
          startDate={startDate}
        />
      </div>
    </Modal>
  );
}
