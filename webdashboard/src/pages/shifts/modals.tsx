/* eslint-disable @typescript-eslint/no-unsafe-assignment */
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
    isBreakOvertime,
    isCreateModalOpen,
    isFixedBreak,
    newBreakEnd,
    newBreakStart,
    newShiftBreak,
    newShiftEnd,
    newShiftName,
    newShiftStart,
    newShiftSymbol,
    newShiftGrace,
    allowEarlyOt,
    maxEarlyOtMinutes,
    setIsAutoDeduct,
    setIsBreakOvertime,
    setIsCreateModalOpen,
    setIsFixedBreak,
    setNewBreakEnd,
    setNewBreakStart,
    setNewShiftBreak,
    setNewShiftEnd,
    setNewShiftName,
    setNewShiftStart,
    setNewShiftSymbol,
    setNewShiftGrace,
    setAllowEarlyOt,
    setMaxEarlyOtMinutes,
  } = hook;

  return (
    <CreateShiftModal
      isAutoDeduct={isAutoDeduct}
      isBreakOvertime={isBreakOvertime}
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
      newShiftGrace={newShiftGrace}
      allowEarlyOt={allowEarlyOt}
      maxEarlyOtMinutes={maxEarlyOtMinutes}
      onClose={() => {
        setIsCreateModalOpen(false);
      }}
      onCreateShift={handleCreateShift}
      setIsAutoDeduct={setIsAutoDeduct}
      setIsBreakOvertime={setIsBreakOvertime}
      setIsFixedBreak={setIsFixedBreak}
      setNewBreakEnd={setNewBreakEnd}
      setNewBreakStart={setNewBreakStart}
      setNewShiftBreak={setNewShiftBreak}
      setNewShiftEnd={setNewShiftEnd}
      setNewShiftName={setNewShiftName}
      setNewShiftStart={setNewShiftStart}
      setNewShiftSymbol={setNewShiftSymbol}
      setNewShiftGrace={setNewShiftGrace}
      setAllowEarlyOt={setAllowEarlyOt}
      setMaxEarlyOtMinutes={setMaxEarlyOtMinutes}
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
    isBreakOvertime,
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
    allowEarlyOt,
    maxEarlyOtMinutes,
    setCorrectionEffectiveDate,
    setEditingShift,
    setIsAutoDeduct,
    setIsBreakOvertime,
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
    setAllowEarlyOt,
    setMaxEarlyOtMinutes,
    updateShiftMutation,
  } = hook;

  return (
    <EditShiftModal
      correctionEffectiveDate={correctionEffectiveDate}
      editingShift={editingShift}
      isAutoDeduct={isAutoDeduct}
      isBreakOvertime={isBreakOvertime}
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
      allowEarlyOt={allowEarlyOt}
      maxEarlyOtMinutes={maxEarlyOtMinutes}
      onClose={() => {
        setEditingShift(null);
      }}
      onUpdateShift={handleUpdateShift}
      setCorrectionEffectiveDate={setCorrectionEffectiveDate}
      setIsAutoDeduct={setIsAutoDeduct}
      setIsBreakOvertime={setIsBreakOvertime}
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
      setAllowEarlyOt={setAllowEarlyOt}
      setMaxEarlyOtMinutes={setMaxEarlyOtMinutes}
    />
  );
}
