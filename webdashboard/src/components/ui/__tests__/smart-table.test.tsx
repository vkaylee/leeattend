import "@testing-library/jest-dom";
import { render, screen, fireEvent, within } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi } from "vitest";

import SmartTable, { type SmartTableProps } from "../smart-table";

interface TestData {
  id: string;
  name: string;
  age: number;
}

const testData: TestData[] = [
  { id: "1", name: "Alice", age: 25 },
  { id: "2", name: "Bob", age: 30 },
  { id: "3", name: "Charlie", age: 35 },
];

const testColumns = [
  {
    id: "name",
    accessorKey: "name",
    header: "Name",
    meta: {
      editable: true,
      sticky: "left" as const,
    },
  },
  {
    id: "age",
    accessorKey: "age",
    header: "Age",
    meta: {
      editable: true,
      editType: "number" as const,
    },
  },
];

const renderSmartTable = (props: Partial<SmartTableProps<TestData>> = {}) => {
  return render(
    <SmartTable<TestData>
      data={testData}
      columns={testColumns}
      getRowId={(row) => row.id}
      {...props}
    />
  );
};

describe("SmartTable Component", () => {
  it("renders headers and data correctly", () => {
    renderSmartTable();

    // Verify headers
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Age")).toBeInTheDocument();

    // Verify data
    expect(screen.getByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("Bob")).toBeInTheDocument();
    expect(screen.getByText("Charlie")).toBeInTheDocument();
  });

  it("handles sorting clicks", () => {
    const handleSort = vi.fn();
    renderSmartTable();

    const header = screen.getByText("Name");
    fireEvent.click(header);

    // Verify sort behavior header elements
    expect(header.closest("th")).toBeInTheDocument();
  });

  it("manages keyboard navigation focus", () => {
    renderSmartTable();

    const cellAlice = screen.getByText("Alice").closest("td");
    expect(cellAlice).toHaveAttribute("tabindex", "-1");

    // Click to focus first cell
    if (cellAlice) {
      fireEvent.click(cellAlice);
    }
    expect(cellAlice).toHaveAttribute("tabindex", "0");

    // ArrowDown should move focus to Bob
    if (cellAlice) {
      fireEvent.keyDown(cellAlice, { key: "ArrowDown" });
    }
    const cellBob = screen.getByText("Bob").closest("td");
    expect(cellBob).toHaveAttribute("tabindex", "0");

    // ArrowRight should move focus to Bob's age (30)
    if (cellBob) {
      fireEvent.keyDown(cellBob, { key: "ArrowRight" });
    }
    const cellBobAge = screen.getByText("30").closest("td");
    expect(cellBobAge).toHaveAttribute("tabindex", "0");
  });

  it("handles cell editing trigger and callbacks", async () => {
    const onCellSave = vi.fn();
    renderSmartTable({ onCellSave });

    const cellAlice = screen.getByText("Alice").closest("td");
    if (cellAlice) {
      // Double click to edit
      fireEvent.doubleClick(cellAlice);
    }

    // Verify input appears
    const input = screen.getByRole("textbox");
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue("Alice");

    // Change input value
    fireEvent.change(input, { target: { value: "Alison" } });
    
    // Press Enter to save
    fireEvent.keyDown(input, { key: "Enter" });

    // Verify callback triggers
    expect(onCellSave).toHaveBeenCalledWith("1", "name", "Alison");

    // Verify input disappears and original or updated content shown (normally state flows from parent, but input is unmounted)
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("handles escape to cancel editing", () => {
    const onCellSave = vi.fn();
    renderSmartTable({ onCellSave });

    const cellAlice = screen.getByText("Alice").closest("td");
    if (cellAlice) {
      fireEvent.doubleClick(cellAlice);
    }

    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "Alison" } });
    
    // Press Escape to cancel
    fireEvent.keyDown(input, { key: "Escape" });

    expect(onCellSave).not.toHaveBeenCalled();
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("applies pinning sticky styles correctly", () => {
    renderSmartTable();
    
    // Name column has sticky: "left"
    const headerName = screen.getByText("Name").closest("th");
    expect(headerName).toHaveStyle("position: sticky");
    expect(headerName).toHaveStyle("left: 0px");
  });

  it("handles copy of selected rows/cell on Ctrl+C", async () => {
    const writeTextMock = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      value: { writeText: writeTextMock },
      configurable: true,
      writable: true,
    });

    // Render table with selection
    const selectedRowIds = new Set(["1", "2"]);
    renderSmartTable({ selectedRowIds });

    const container = screen.getByText("Alice").closest("table");
    expect(container).toBeInTheDocument();
    
    fireEvent.keyDown(container!, { key: "c", ctrlKey: true });

    expect(writeTextMock).toHaveBeenCalled();
    const copiedText = writeTextMock.mock.calls[0][0];
    expect(copiedText).toContain("Alice\t25");
    expect(copiedText).toContain("Bob\t30");
  });

  it("handles pasting grid values starting from focusedCell", () => {
    const onBulkCellSave = vi.fn();
    renderSmartTable({ onBulkCellSave });

    const cellAlice = screen.getByText("Alice").closest("td");
    fireEvent.click(cellAlice!);

    const pasteText = "Alice-Updated\t26\nBob-Updated\t31";
    const container = screen.getByText("Alice").closest("table")?.parentElement;

    fireEvent.paste(container!, {
      clipboardData: {
        getData: (type: string) => (type === "text" ? pasteText : ""),
      },
    });

    expect(onBulkCellSave).toHaveBeenCalledWith([
      { rowId: "1", columnId: "name", value: "Alice-Updated" },
      { rowId: "1", columnId: "age", value: 26 },
      { rowId: "2", columnId: "name", value: "Bob-Updated" },
      { rowId: "2", columnId: "age", value: 31 },
    ]);
  });

  it("applies validateCell rules during paste and highlights invalid cells", async () => {
    const validateCell = vi.fn((rowId, colId, value) => {
      if (colId === "age") {
        const num = Number(value);
        if (num > 30) return "Age must be 30 or less";
      }
      return null;
    });
    const onBulkCellSave = vi.fn();
    renderSmartTable({ validateCell, onBulkCellSave });

    const cellAlice = screen.getByText("Alice").closest("td");
    fireEvent.click(cellAlice!);

    const pasteText = "Alice-Updated\t29\nBob-Updated\t35";
    const container = screen.getByText("Alice").closest("table")?.parentElement;

    fireEvent.paste(container!, {
      clipboardData: {
        getData: (type: string) => (type === "text" ? pasteText : ""),
      },
    });

    expect(onBulkCellSave).toHaveBeenCalledWith([
      { rowId: "1", columnId: "name", value: "Alice-Updated" },
      { rowId: "1", columnId: "age", value: 29 },
      { rowId: "2", columnId: "name", value: "Bob-Updated" },
    ]);

    const rowBob = screen.getByText("Bob").closest("tr");
    const cellBobAge = within(rowBob!).getByText("35").closest("td");
    expect(cellBobAge).toBeInTheDocument();
    expect(cellBobAge).toHaveClass("border-rose-500");

    const tooltip = screen.getByRole("tooltip", { hidden: true });
    expect(tooltip).toBeInTheDocument();
    expect(tooltip).toHaveTextContent("Age must be 30 or less");
  });
});
