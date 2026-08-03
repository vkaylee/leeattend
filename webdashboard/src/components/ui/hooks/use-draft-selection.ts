import { useState, useEffect } from "react";

interface UseDraftSelectionOptions {
  /** When false, draft mode is bypassed (live-toggle mode). */
  enabled: boolean;
  /** Whether the panel hosting the draft is currently open. */
  isOpen: boolean;
  /** Committed (external) selected values — the source of truth. */
  selected: string[];
}

interface UseDraftSelectionReturn {
  draftSelected: string[];
  setDraftSelected: React.Dispatch<React.SetStateAction<string[]>>;
}

/**
 * Manages draft (uncommitted) selection state for filter panels with an
 * Apply/Cancel confirmation flow. Syncs the draft to the external `selected`
 * value whenever the panel opens, so the user always starts from the committed
 * state.
 *
 * When `enabled` is false (live-toggle mode), the caller should use `selected`
 * directly and ignore `draftSelected`.
 */
export function useDraftSelection({
  selected,
  isOpen,
  enabled,
}: UseDraftSelectionOptions): UseDraftSelectionReturn {
  const [draftSelected, setDraftSelected] = useState<string[]>(selected);

  useEffect(() => {
    if (isOpen && enabled) {
      setDraftSelected([...selected]);
    }
  }, [isOpen, selected, enabled]);

  return { draftSelected, setDraftSelected };
}
