import { render, screen, fireEvent } from "@testing-library/react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import { LeaveTypeForm } from "../leave-type-form";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "settings.leave.type_form.type_code": "Type Code",
        "settings.leave.type_form.type_code_placeholder": "e.g. AL, SL, UL",
        "settings.leave.type_form.display_name": "Display Name",
        "settings.leave.type_form.display_name_placeholder": "e.g. Annual Leave",
        "settings.leave.type_form.paid_status": "Paid Status",
        "settings.leave.type_form.paid_time_off": "Paid time off",
        "settings.leave.type_form.unpaid_leave": "Unpaid leave",
        "settings.leave.type_form.change_template": "Change Template",
        "settings.leave.type_form.creating": "Creating...",
        "settings.leave.type_form.confirm_create": "Confirm & Create",
        "settings.leave.type_manager.create_error_duplicate_code":
          "A leave type with this code already exists",
      };
      return translations[key] || key;
    },
  }),
}));

const baseProps = {
  existingCodes: ["al", "sl"],
  initialData: { code: "", name: "", is_paid: true },
  isSaving: false,
  onBackToTemplates: vi.fn(),
  onCancel: vi.fn(),
  onSave: vi.fn(),
};

describe("LeaveTypeForm", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("flags a duplicate code case-insensitively and disables the submit button", () => {
    render(<LeaveTypeForm {...baseProps} />);

    const codeInput = screen.getByLabelText("Type Code");
    fireEvent.change(codeInput, { target: { value: "AL" } });

    expect(
      screen.getByText("A leave type with this code already exists"),
    ).toBeInTheDocument();

    const submit = screen.getByRole("button", { name: "Confirm & Create" });
    expect(submit).toBeDisabled();
  });

  it("does not flag a unique code", () => {
    render(<LeaveTypeForm {...baseProps} />);

    const codeInput = screen.getByLabelText("Type Code");
    fireEvent.change(codeInput, { target: { value: "ML" } });

    expect(
      screen.queryByText("A leave type with this code already exists"),
    ).not.toBeInTheDocument();

    const submit = screen.getByRole("button", { name: "Confirm & Create" });
    expect(submit).not.toBeDisabled();
  });

  it("does not submit when the code is a duplicate", () => {
    render(<LeaveTypeForm {...baseProps} />);

    const codeInput = screen.getByLabelText("Type Code");
    fireEvent.change(codeInput, { target: { value: "sl" } });
    fireEvent.click(screen.getByRole("button", { name: "Confirm & Create" }));

    expect(baseProps.onSave).not.toHaveBeenCalled();
  });

  it("submits the form data when the code is unique", () => {
    render(<LeaveTypeForm {...baseProps} />);

    fireEvent.change(screen.getByLabelText("Type Code"), {
      target: { value: "ML" },
    });
    fireEvent.change(screen.getByLabelText("Display Name"), {
      target: { value: "Maternity Leave" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Confirm & Create" }));

    expect(baseProps.onSave).toHaveBeenCalledWith({
      code: "ML",
      name: "Maternity Leave",
      is_paid: true,
    });
  });
});
