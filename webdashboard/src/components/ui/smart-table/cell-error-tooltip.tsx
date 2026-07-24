import React, { memo } from "react";

const CellErrorTooltip = memo(function CellErrorTooltip({
  cellError,
  rowId,
  columnId,
}: {
  cellError: string;
  rowId: string;
  columnId: string;
}) {
  return (
    <>
      <span className="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-rose-500 animate-pulse pointer-events-none" />
      <div
        className="absolute hidden group-hover:block group-focus-within:block bottom-full left-1/2 -translate-x-1/2 mb-2 z-30 px-2.5 py-1.5 text-xs font-semibold text-white bg-rose-600 dark:bg-rose-700 rounded-lg shadow-lg whitespace-nowrap pointer-events-none transition-all duration-200"
        id={`error-${rowId}-${columnId}`}
        role="tooltip"
      >
        {cellError}
        <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-1 border-4 border-transparent border-t-rose-600 dark:border-t-rose-700" />
      </div>
    </>
  );
});

export default CellErrorTooltip;
