import "@testing-library/jest-dom";
import { vi } from "vitest";

// Mock Combobox to be a simple select for existing tests to work
vi.mock("@/components/ui/Combobox", () => ({
  Combobox: ({ 
    options, 
    value, 
    onChange, 
    id, 
    "data-testid": testId 
  }: any) => (
    <select
      id={id}
      data-testid={testId}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    >
      {options.map((opt: any) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));
