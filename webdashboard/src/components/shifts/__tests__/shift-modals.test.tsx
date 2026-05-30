/* eslint-disable */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";

import { CreateShiftModal } from "../create-shift-modal";
import { EditShiftModal } from "../edit-shift-modal";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

const mockShift = {
  id: "shift-1",
  name: "Morning Shift",
  start_time: "08:00",
  end_time: "17:00",
  break_duration_minutes: 60,
  is_break_auto_deduct: true,
  break_is_overtime: false,
};

describe("Shift Modals Break Policy Tests", () => {
  it("renders CreateShiftModal with three break policies", () => {
    const setIsAutoDeduct = vi.fn();
    const setIsBreakOvertime = vi.fn();

    render(
      <CreateShiftModal
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={false}
        isLoading={false}
        isOpen={true}
        newBreakEnd=""
        newBreakStart=""
        newShiftBreak={60}
        newShiftEnd="17:00"
        onClose={vi.fn()}
        onCreateShift={vi.fn()}
        setIsAutoDeduct={setIsAutoDeduct}
        setIsBreakOvertime={setIsBreakOvertime}
        setIsFixedBreak={vi.fn()}
        setNewBreakEnd={vi.fn()}
        setNewBreakStart={vi.fn()}
        setNewShiftBreak={vi.fn()}
        setNewShiftEnd={vi.fn()}
        setNewShiftName={vi.fn()}
        setNewShiftStart={vi.fn()}
        setNewShiftSymbol={vi.fn()}
        newShiftName="Test Shift"
        newShiftStart="08:00"
        newShiftSymbol="T"
      />
    );

    expect(screen.getByLabelText("shifts.modal.break_policy_auto_deduct")).toBeDefined();
    expect(screen.getByLabelText("shifts.modal.break_policy_no_deduct")).toBeDefined();
    expect(screen.getByLabelText("shifts.modal.break_policy_overtime")).toBeDefined();
  });

  it("triggers correct state changes on CreateShiftModal break policy selection", async () => {
    const user = userEvent.setup();
    const setIsAutoDeduct = vi.fn();
    const setIsBreakOvertime = vi.fn();

    render(
      <CreateShiftModal
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={false}
        isLoading={false}
        isOpen={true}
        newBreakEnd=""
        newBreakStart=""
        newShiftBreak={60}
        newShiftEnd="17:00"
        onClose={vi.fn()}
        onCreateShift={vi.fn()}
        setIsAutoDeduct={setIsAutoDeduct}
        setIsBreakOvertime={setIsBreakOvertime}
        setIsFixedBreak={vi.fn()}
        setNewBreakEnd={vi.fn()}
        setNewBreakStart={vi.fn()}
        setNewShiftBreak={vi.fn()}
        setNewShiftEnd={vi.fn()}
        setNewShiftName={vi.fn()}
        setNewShiftStart={vi.fn()}
        setNewShiftSymbol={vi.fn()}
        newShiftName="Test Shift"
        newShiftStart="08:00"
        newShiftSymbol="T"
      />
    );

    // Option 1 (auto_deduct) should be checked
    const autoDeductRadio = screen.getByLabelText("shifts.modal.break_policy_auto_deduct") as HTMLInputElement;
    expect(autoDeductRadio.checked).toBe(true);

    // Click Option 2 (no_deduct)
    const noDeductRadio = screen.getByLabelText("shifts.modal.break_policy_no_deduct");
    await user.click(noDeductRadio);
    expect(setIsAutoDeduct).toHaveBeenCalledWith(false);
    expect(setIsBreakOvertime).toHaveBeenCalledWith(false);

    // Click Option 3 (overtime)
    const overtimeRadio = screen.getByLabelText("shifts.modal.break_policy_overtime");
    await user.click(overtimeRadio);
    expect(setIsAutoDeduct).toHaveBeenCalledWith(false);
    expect(setIsBreakOvertime).toHaveBeenCalledWith(true);
  });

  it("renders EditShiftModal with preselected options based on state", () => {
    render(
      <EditShiftModal
        correctionEffectiveDate="2026-05-30"
        editingShift={mockShift}
        isAutoDeduct={false}
        isBreakOvertime={true}
        isCorrection={false}
        isFixedBreak={false}
        isLoading={false}
        newBreakEnd=""
        newBreakStart=""
        newShiftBreak={60}
        newShiftEnd="17:00"
        newShiftGrace={15}
        newShiftName="Test Shift"
        newShiftStart="08:00"
        newShiftSymbol="T"
        onClose={vi.fn()}
        onUpdateShift={vi.fn()}
        setCorrectionEffectiveDate={vi.fn()}
        setIsAutoDeduct={vi.fn()}
        setIsBreakOvertime={vi.fn()}
        setIsCorrection={vi.fn()}
        setIsFixedBreak={vi.fn()}
        setNewBreakEnd={vi.fn()}
        setNewBreakStart={vi.fn()}
        setNewShiftBreak={vi.fn()}
        setNewShiftEnd={vi.fn()}
        setNewShiftGrace={vi.fn()}
        setNewShiftName={vi.fn()}
        setNewShiftStart={vi.fn()}
        setNewShiftSymbol={vi.fn()}
      />
    );

    // Overtime break option should be checked
    const overtimeRadio = screen.getByLabelText("shifts.modal.break_policy_overtime") as HTMLInputElement;
    expect(overtimeRadio.checked).toBe(true);

    const autoDeductRadio = screen.getByLabelText("shifts.modal.break_policy_auto_deduct") as HTMLInputElement;
    expect(autoDeductRadio.checked).toBe(false);
  });

  it("renders Break Start and Break End inputs in CreateShiftModal when isFixedBreak is true", () => {
    render(
      <CreateShiftModal
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={true}
        isLoading={false}
        isOpen={true}
        newBreakEnd="13:00"
        newBreakStart="12:00"
        newShiftBreak={60}
        newShiftEnd="17:00"
        onClose={vi.fn()}
        onCreateShift={vi.fn()}
        setIsAutoDeduct={vi.fn()}
        setIsBreakOvertime={vi.fn()}
        setIsFixedBreak={vi.fn()}
        setNewBreakEnd={vi.fn()}
        setNewBreakStart={vi.fn()}
        setNewShiftBreak={vi.fn()}
        setNewShiftEnd={vi.fn()}
        setNewShiftName={vi.fn()}
        setNewShiftStart={vi.fn()}
        setNewShiftSymbol={vi.fn()}
        newShiftName="Test Shift"
        newShiftStart="08:00"
        newShiftSymbol="T"
      />
    );

    // Should display start/end labels and query elements
    expect(screen.getByText("shifts.modal.break_start")).toBeDefined();
    expect(screen.getByText("shifts.modal.break_end")).toBeDefined();
    expect(screen.queryByText("shifts.modal.break_duration")).toBeNull();
  });

  it("renders Break Duration input in CreateShiftModal when isFixedBreak is false", () => {
    render(
      <CreateShiftModal
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={false}
        isLoading={false}
        isOpen={true}
        newBreakEnd=""
        newBreakStart=""
        newShiftBreak={60}
        newShiftEnd="17:00"
        onClose={vi.fn()}
        onCreateShift={vi.fn()}
        setIsAutoDeduct={vi.fn()}
        setIsBreakOvertime={vi.fn()}
        setIsFixedBreak={vi.fn()}
        setNewBreakEnd={vi.fn()}
        setNewBreakStart={vi.fn()}
        setNewShiftBreak={vi.fn()}
        setNewShiftEnd={vi.fn()}
        setNewShiftName={vi.fn()}
        setNewShiftStart={vi.fn()}
        setNewShiftSymbol={vi.fn()}
        newShiftName="Test Shift"
        newShiftStart="08:00"
        newShiftSymbol="T"
      />
    );

    // Should display duration label and query elements
    expect(screen.getByText(/shifts.modal.break_duration/)).toBeDefined();
    expect(screen.queryByText("shifts.modal.break_start")).toBeNull();
    expect(screen.queryByText("shifts.modal.break_end")).toBeNull();
  });

  it("triggers setIsFixedBreak when Fixed Break toggle switch is clicked", async () => {
    const user = userEvent.setup();
    const setIsFixedBreak = vi.fn();

    render(
      <CreateShiftModal
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={false}
        isLoading={false}
        isOpen={true}
        newBreakEnd=""
        newBreakStart=""
        newShiftBreak={60}
        newShiftEnd="17:00"
        onClose={vi.fn()}
        onCreateShift={vi.fn()}
        setIsAutoDeduct={vi.fn()}
        setIsBreakOvertime={vi.fn()}
        setIsFixedBreak={setIsFixedBreak}
        setNewBreakEnd={vi.fn()}
        setNewBreakStart={vi.fn()}
        setNewShiftBreak={vi.fn()}
        setNewShiftEnd={vi.fn()}
        setNewShiftName={vi.fn()}
        setNewShiftStart={vi.fn()}
        setNewShiftSymbol={vi.fn()}
        newShiftName="Test Shift"
        newShiftStart="08:00"
        newShiftSymbol="T"
      />
    );

    const toggleSwitch = screen.getByRole("button", { name: "shifts.types.fixed" });
    await user.click(toggleSwitch);
    expect(setIsFixedBreak).toHaveBeenCalledWith(true);
  });
});
