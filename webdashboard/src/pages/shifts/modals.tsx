import React from "react";

import { CreateShiftModal } from "@/components/shifts/create-shift-modal";
import { EditShiftModal } from "@/components/shifts/edit-shift-modal";
import { ShiftPickerModal } from "@/components/shifts/shift-picker-modal";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsModalsProps {
  hook: ReturnType<typeof useShifts>;
}

export function ShiftsModals({ hook }: ShiftsModalsProps) {
  return (
    <>
      <CreateModal hook={hook} />
      <ShiftPicker hook={hook} />
      <EditModal hook={hook} />
    </>
  );
}

function CreateModal({ hook }: ShiftsModalsProps) {
  const {
    createShiftMutation,
    handleCreateShift,
    isAutoDeduct,
    isCreateModalOpen,
    isFixedBreak,
    newBreakEnd,
    newBreakStart,
    newShiftBreak,
    newShiftEnd,
    newShiftName,
    newShiftStart,
    newShiftSymbol,
    setIsAutoDeduct,
    setIsCreateModalOpen,
    setIsFixedBreak,
    setNewBreakEnd,
    setNewBreakStart,
    setNewShiftBreak,
    setNewShiftEnd,
    setNewShiftName,
    setNewShiftStart,
    setNewShiftSymbol,
  } = hook;

  return (
    <CreateShiftModal
      isAutoDeduct={isAutoDeduct}
      isFixedBreak={isFixedBreak}
      isLoading={createShiftMutation.isPending}
      isOpen={isCreateModalOpen}
      newBreakEnd={newBreakEnd}
      newBreakStart={newBreakStart}
      newShiftBreak={newShiftBreak}
      newShiftEnd={newShiftEnd}
      newShiftName={newShiftName}
      newShiftStart={newShiftStart}
      newShiftSymbol={newShiftSymbol}
      onClose={() => {
        setIsCreateModalOpen(false);
      }}
      onCreateShift={handleCreateShift}
      setIsAutoDeduct={setIsAutoDeduct}
      setIsFixedBreak={setIsFixedBreak}
      setNewBreakEnd={setNewBreakEnd}
      setNewBreakStart={setNewBreakStart}
      setNewShiftBreak={setNewShiftBreak}
      setNewShiftEnd={setNewShiftEnd}
      setNewShiftName={setNewShiftName}
      setNewShiftStart={setNewShiftStart}
      setNewShiftSymbol={setNewShiftSymbol}
    />
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

function EditModal({ hook }: ShiftsModalsProps) {
  const {
    correctionEffectiveDate,
    editingShift,
    handleUpdateShift,
    isAutoDeduct,
    isCorrection,
    isFixedBreak,
    newBreakEnd,
    newBreakStart,
    newShiftBreak,
    newShiftEnd,
    newShiftGrace,
    newShiftName,
    newShiftStart,
    newShiftSymbol,
    setCorrectionEffectiveDate,
    setEditingShift,
    setIsAutoDeduct,
    setIsCorrection,
    setIsFixedBreak,
    setNewBreakEnd,
    setNewBreakStart,
    setNewShiftBreak,
    setNewShiftEnd,
    setNewShiftGrace,
    setNewShiftName,
    setNewShiftStart,
    setNewShiftSymbol,
    updateShiftMutation,
  } = hook;

  return (
    <EditShiftModal
      correctionEffectiveDate={correctionEffectiveDate}
      editingShift={editingShift}
      isAutoDeduct={isAutoDeduct}
      isCorrection={isCorrection}
      isFixedBreak={isFixedBreak}
      isLoading={updateShiftMutation.isPending}
      newBreakEnd={newBreakEnd}
      newBreakStart={newBreakStart}
      newShiftBreak={newShiftBreak}
      newShiftEnd={newShiftEnd}
      newShiftGrace={newShiftGrace}
      newShiftName={newShiftName}
      newShiftStart={newShiftStart}
      newShiftSymbol={newShiftSymbol}
      onClose={() => {
        setEditingShift(null);
      }}
      onUpdateShift={handleUpdateShift}
      setCorrectionEffectiveDate={setCorrectionEffectiveDate}
      setIsAutoDeduct={setIsAutoDeduct}
      setIsCorrection={setIsCorrection}
      setIsFixedBreak={setIsFixedBreak}
      setNewBreakEnd={setNewBreakEnd}
      setNewBreakStart={setNewBreakStart}
      setNewShiftBreak={setNewShiftBreak}
      setNewShiftEnd={setNewShiftEnd}
      setNewShiftGrace={setNewShiftGrace}
      setNewShiftName={setNewShiftName}
      setNewShiftStart={setNewShiftStart}
      setNewShiftSymbol={setNewShiftSymbol}
    />
  );
}
