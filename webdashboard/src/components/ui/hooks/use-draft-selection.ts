import { useState, useCallback } from "react";

interface UseDraftSelectionOptions {
  /** When false, draft mode is bypassed (live-toggle mode). */
  enabled: boolean;
  /** Committed (external) selected values — the source of truth. */
  selected: string[];
}

interface UseDraftSelectionReturn {
  draftSelected: string[];
  /** Reset the draft to match the committed selection (call on panel open). */
  resetDraft: () => void;
  setDraftSelected: React.Dispatch<React.SetStateAction<string[]>>;
}

/**
 * Manages draft (uncommitted) selection state for filter panels with an
 * Apply/Cancel confirmation flow.
 *
 * Call `resetDraft()` in the panel's open handler (e.g. onOpenChange) to sync
 * the draft with the committed selection — this replaces the previous
 * useEffect-based sync and avoids cascading renders.
 *
 * When `enabled` is false (live-toggle mode), the caller should use `selected`
 * directly and ignore `draftSelected`.
 */
export function useDraftSelection({
  selected,
  enabled,
}: UseDraftSelectionOptions): UseDraftSelectionReturn {
  const [draftSelected, setDraftSelected] = useState<string[]>(selected);

  const resetDraft = useCallback(() => {
    if (enabled) {
      setDraftSelected([...selected]);
    }
  }, [selected, enabled]);

  return { draftSelected, setDraftSelected, resetDraft };
}
