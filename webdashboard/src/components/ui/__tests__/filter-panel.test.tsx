import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { FilterPanel } from "../filter-panel";
import { FilterTrigger } from "../filter-trigger";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key === "common.selected_count") return `Đã chọn ${opts?.selected}/${opts?.total}`;
      if (key === "common.all_departments") return "Tất cả";
      if (key === "common.clear_filter") return "Xóa bộ lọc";
      if (key === "common.combobox.search") return "Tìm kiếm...";
      if (key === "common.no_options") return "Không có tùy chọn";
      if (key === "common.all_selected") return "Tất cả";
      if (key === "common.clear") return "Xóa";
      if (key === "common.cancel") return "Hủy";
      if (key === "common.apply") return "Áp dụng";
      return key;
    },
  }),
}));

const DEPT_OPTIONS = [
  { value: "dept-1", label: "Engineering" },
  { value: "dept-2", label: "Design" },
  { value: "dept-3", label: "Marketing" },
];

function renderPopover(props?: Partial<Parameters<typeof FilterPanel>[0]>) {
  return render(
    <FilterPanel
      align="start"
      container="popover"
      data-testid="filter-panel"
      mode="multi"
      onClear={vi.fn()}
      onToggle={vi.fn()}
      options={DEPT_OPTIONS}
      selected={[]}
      trigger={
        <FilterTrigger
          data-testid="filter-trigger"
          label="Department"
          variant="pill-button"
        />
      }
      {...props}
    />,
  );
}

describe("FilterPanel — popover container, multi mode", () => {
  it("opens popover when trigger is clicked", async () => {
    renderPopover();
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-content")).toBeInTheDocument();
    });
  });

  it("renders all options in the panel", async () => {
    renderPopover();
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-option-dept-1")).toBeInTheDocument();
      expect(screen.getByTestId("filter-panel-option-dept-2")).toBeInTheDocument();
      expect(screen.getByTestId("filter-panel-option-dept-3")).toBeInTheDocument();
    });
  });

  it("renders selected options with checked state", async () => {
    renderPopover({ selected: ["dept-1"] });
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      const opt = screen.getByTestId("filter-panel-option-dept-1");
      expect(opt).toHaveAttribute("aria-selected", "true");
    });
  });

  it("calls onToggle when an option is clicked (live mode)", async () => {
    const onToggle = vi.fn();
    renderPopover({ onToggle });
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-option-dept-2")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("filter-panel-option-dept-2"));
    expect(onToggle).toHaveBeenCalledWith("dept-2");
  });

  it("calls onClear when clear button is clicked", async () => {
    const onClear = vi.fn();
    renderPopover({ selected: ["dept-1", "dept-2"], onClear });
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-clear")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("filter-panel-clear"));
    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it("does not show clear button when no filters active", async () => {
    renderPopover({ selected: [], showClearButton: true });
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-content")).toBeInTheDocument();
    });

    expect(screen.queryByTestId("filter-panel-clear")).toBeNull();
  });

  it("shows empty message when options list is empty", async () => {
    renderPopover({ options: [], emptyMessage: "Nothing here" });
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByText("Nothing here")).toBeInTheDocument();
    });
  });
});

describe("FilterPanel — single mode", () => {
  it("renders radio-style options", async () => {
    render(
      <FilterPanel
        container="popover"
        data-testid="filter-panel"
        mode="single"
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={[]}
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Status"
            variant="pill-button"
          />
        }
      />,
    );
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      const opt = screen.getByTestId("filter-panel-option-dept-1");
      expect(opt).toBeInTheDocument();
    });
  });
});

describe("FilterPanel — dropdown container", () => {
  it("opens dropdown when trigger is clicked", async () => {
    const user = userEvent.setup();
    render(
      <FilterPanel
        container="dropdown"
        data-testid="filter-panel"
        mode="multi"
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={[]}
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Department"
            variant="pill-button"
          />
        }
      />,
    );
    await user.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-content")).toBeInTheDocument();
    });
  });

  it("shows 'All' item in multi mode", async () => {
    const user = userEvent.setup();
    render(
      <FilterPanel
        container="dropdown"
        data-testid="filter-panel"
        mode="multi"
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={[]}
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Department"
            variant="pill-button"
          />
        }
      />,
    );
    await user.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-option-all")).toBeInTheDocument();
    });
  });

  it("calls onClear when 'All' is clicked", async () => {
    const user = userEvent.setup();
    const onClear = vi.fn();
    render(
      <FilterPanel
        container="dropdown"
        data-testid="filter-panel"
        mode="multi"
        onClear={onClear}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={["dept-1"]}
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Department"
            variant="pill-button"
          />
        }
      />,
    );
    await user.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-option-all")).toBeInTheDocument();
    });

    await user.click(screen.getByTestId("filter-panel-option-all"));
    expect(onClear).toHaveBeenCalledTimes(1);
  });
});

describe("FilterPanel — apply-button mode", () => {
  it("shows Apply and Cancel buttons", async () => {
    const onApply = vi.fn();
    render(
      <FilterPanel
        container="popover"
        data-testid="filter-panel"
        mode="multi"
        onApply={onApply}
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={[]}
        showApplyButton
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Custom Fields"
            variant="pill-button"
          />
        }
      />,
    );
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByText("Áp dụng")).toBeInTheDocument();
      expect(screen.getByText("Hủy")).toBeInTheDocument();
    });
  });

  it("commits draft on Apply", async () => {
    const onApply = vi.fn();
    render(
      <FilterPanel
        container="popover"
        data-testid="filter-panel"
        mode="multi"
        onApply={onApply}
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={[]}
        showApplyButton
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Custom Fields"
            variant="pill-button"
          />
        }
      />,
    );
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-option-dept-1")).toBeInTheDocument();
    });

    // Toggle an option in draft mode
    fireEvent.click(screen.getByTestId("filter-panel-option-dept-1"));
    // Click Apply
    fireEvent.click(screen.getByText("Áp dụng"));

    expect(onApply).toHaveBeenCalledWith(["dept-1"]);
  });

  it("discards draft on Cancel", async () => {
    const onApply = vi.fn();
    render(
      <FilterPanel
        container="popover"
        data-testid="filter-panel"
        mode="multi"
        onApply={onApply}
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={["dept-2"]}
        showApplyButton
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Custom Fields"
            variant="pill-button"
          />
        }
      />,
    );
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByText("Hủy")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Hủy"));
    // onApply should NOT be called
    expect(onApply).not.toHaveBeenCalled();
  });
});

describe("FilterPanel — searchable", () => {
  it("shows search input when searchable=true", async () => {
    render(
      <FilterPanel
        container="popover"
        data-testid="filter-panel"
        mode="multi"
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        searchable
        selected={[]}
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Shift Type"
            variant="pill-button"
          />
        }
      />,
    );
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      const input = document.querySelector("input");
      expect(input).toBeInTheDocument();
    });
  });
});

describe("FilterPanel — showSelectionCount", () => {
  it("renders selection count footer", async () => {
    render(
      <FilterPanel
        container="popover"
        data-testid="filter-panel"
        mode="multi"
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={DEPT_OPTIONS}
        selected={["dept-1"]}
        showSelectionCount
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Department"
            variant="pill-button"
          />
        }
      />,
    );
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByText("Đã chọn 1/3")).toBeInTheDocument();
    });
  });
});

describe("FilterPanel — custom children", () => {
  it("renders custom children instead of option list", async () => {
    render(
      <FilterPanel
        container="popover"
        data-testid="filter-panel"
        mode="multi"
        onClear={vi.fn()}
        onToggle={vi.fn()}
        options={[]}
        selected={[]}
        trigger={
          <FilterTrigger
            data-testid="filter-trigger"
            label="Custom"
            variant="pill-button"
          />
        }
      >
        <div data-testid="custom-content">Custom form fields here</div>
      </FilterPanel>,
    );
    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("custom-content")).toBeInTheDocument();
    });
  });
});

describe("FilterPanel — keyboard shortcut", () => {
  it("opens panel with keyboard shortcut Ctrl+Shift+D", async () => {
    renderPopover({ keyboardShortcut: "Ctrl+Shift+D" });

    // Panel should be closed initially
    expect(screen.queryByTestId("filter-panel-content")).toBeNull();

    // Fire the shortcut
    fireEvent.keyDown(window, {
      key: "D",
      ctrlKey: true,
      shiftKey: true,
    });

    await waitFor(() => {
      expect(screen.getByTestId("filter-panel-content")).toBeInTheDocument();
    });
  });

  it("does not open panel when modifier keys don't match", async () => {
    renderPopover({ keyboardShortcut: "Ctrl+Shift+D" });

    // Only Ctrl+D (missing Shift)
    fireEvent.keyDown(window, {
      key: "D",
      ctrlKey: true,
      shiftKey: false,
    });

    // Should still be closed
    expect(screen.queryByTestId("filter-panel-content")).toBeNull();
  });

  it("does not fire when focus is inside an input", async () => {
    render(
      <div>
        <input data-testid="search-input" placeholder="Search..." />
        <FilterPanel
          container="popover"
          data-testid="filter-panel"
          keyboardShortcut="Ctrl+Shift+F"
          mode="multi"
          onClear={vi.fn()}
          onToggle={vi.fn()}
          options={DEPT_OPTIONS}
          selected={[]}
          trigger={
            <FilterTrigger
              data-testid="filter-trigger"
              label="Department"
              variant="pill-button"
            />
          }
        />
      </div>,
    );

    const input = screen.getByTestId("search-input");
    input.focus();

    fireEvent.keyDown(input, {
      key: "F",
      ctrlKey: true,
      shiftKey: true,
    });

    // Panel should NOT open because focus is in an input
    expect(screen.queryByTestId("filter-panel-content")).toBeNull();
  });

  it("no-op when keyboardShortcut is not set", async () => {
    renderPopover();

    // Random shortcut should not open panel
    fireEvent.keyDown(window, {
      key: "D",
      ctrlKey: true,
      shiftKey: true,
    });

    expect(screen.queryByTestId("filter-panel-content")).toBeNull();
  });
});
