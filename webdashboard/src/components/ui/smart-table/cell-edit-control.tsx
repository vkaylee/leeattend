import React, { memo, useCallback, useEffect, useRef } from "react";

import type { EditableCellValue } from "./types";

interface CellEditControlProps {
  editOptions?: { value: string; label: string }[];
  editType?: "text" | "number" | "select";
  editValue: EditableCellValue;
  onBlur: () => void;
  setEditValue: (val: EditableCellValue) => void;
}

const CellEditControl = memo(function CellEditControl({
  editType,
  editValue,
  setEditValue,
  editOptions,
  onBlur,
}: CellEditControlProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const selectRef = useRef<HTMLSelectElement>(null);

  useEffect(() => {
    if (editType === "select") {
      selectRef.current?.focus();
    } else {
      inputRef.current?.focus();
    }
  }, [editType]);

  const handleSelectChange = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
    setEditValue(e.target.value);
  }, [setEditValue]);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setEditValue(e.target.value);
  }, [setEditValue]);

  if (editType === "select") {
    return (
      <select
        className="w-full h-full bg-white dark:bg-slate-800 text-sm border-none outline-none ring-2 ring-primary-500 rounded px-2 py-1.5 focus:ring-primary-500"
        onBlur={onBlur}
        onChange={handleSelectChange}
        ref={selectRef}
        value={editValue as string}
      >
        {editOptions?.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
    );
  }

  return (
    <input
      className="w-full h-full bg-white dark:bg-slate-800 text-sm border-none outline-none ring-2 ring-primary-500 rounded px-2 py-1.5 focus:ring-primary-500 font-medium"
      onBlur={onBlur}
      onChange={handleInputChange}
      ref={inputRef}
      type={editType === "number" ? "number" : "text"}
      value={(editValue as string | undefined) ?? ""}
    />
  );
});

export default CellEditControl;
