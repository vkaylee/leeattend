import { useState, useCallback } from "react";

import { useDraftSelection } from "@/components/ui/hooks/use-draft-selection";
import { useKeyboardShortcut } from "@/hooks/use-keyboard-shortcut";

import type { UseFilterPanelStateOptions } from "./types";

// ── Hook Return Type ────────────────────────────────────────────────────

export interface UseFilterPanelStateReturn {
  draftSelected: string[];
  effectiveSelected: string[];
  handleApply: () => void;
  handleCancel: () => void;
  handleClear: () => void;
  handleDropdownToggle: (value: string) => void;
  handleToggle: (value: string) => void;
  open: boolean;
  setDraftSelected: React.Dispatch<React.SetStateAction<string[]>>;
  setOpen: (open: boolean) => void;
}

/**
 * useFilterPanelState — encapsulates all state management for FilterPanel:
 * open/close, draft selection (apply-button mode), keyboard shortcut,
 * and the toggle/apply/cancel/clear handlers for both popover and dropdown
 * containers.
 */
export function useFilterPanelState({
  selected,
  onToggle,
  onClear,
  onApply,
  mode,
  showApplyButton,
  keyboardShortcut,
}: UseFilterPanelStateOptions): UseFilterPanelStateReturn {
  const [open, setOpen] = useState(false);

  // Draft state for apply-button mode
  const { draftSelected, setDraftSelected } = useDraftSelection({
    selected,
    isOpen: open,
    enabled: showApplyButton,
  });

  // Keyboard shortcut (opt-in, no default)
  useKeyboardShortcut({
    shortcut: keyboardShortcut,
    onTrigger: () => { setOpen(true); },
    disabled: open,
  });

  const effectiveSelected = showApplyButton ? draftSelected : selected;

  const handleToggle = useCallback(
    (value: string) => {
      if (showApplyButton) {
        setDraftSelected((prev) =>
          prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value],
        );
        return;
      }

      onToggle(value);
      if (mode === "single") {
        setOpen(false);
      }
    },
    [showApplyButton, onToggle, mode, setDraftSelected],
  );

  const handleApply = useCallback(() => {
    if (onApply) {
      onApply(draftSelected);
    }
    setOpen(false);
  }, [onApply, draftSelected]);

  const handleCancel = useCallback(() => {
    setDraftSelected([...selected]);
    setOpen(false);
  }, [selected, setDraftSelected]);

  const handleClear = useCallback(() => {
    if (showApplyButton) {
      setDraftSelected([]);
    } else {
      onClear();
    }
  }, [showApplyButton, onClear, setDraftSelected]);

  const handleDropdownToggle = useCallback(
    (value: string) => {
      if (showApplyButton) {
        setDraftSelected((prev) =>
          prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value],
        );
        // prevent close on multi mode
        if (mode === "single") {
          setOpen(false);
          setTimeout(() => { onToggle(value); }, 0);
        }
        return;
      }

      onToggle(value);
      if (mode === "single") {
        // DropdownMenu auto-closes on item select
      }
    },
    [showApplyButton, onToggle, mode, setDraftSelected],
  );

  return {
    open,
    setOpen,
    effectiveSelected,
    draftSelected,
    setDraftSelected,
    handleToggle,
    handleDropdownToggle,
    handleApply,
    handleCancel,
    handleClear,
  };
}
