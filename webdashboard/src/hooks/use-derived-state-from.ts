import { useCallback, useState } from "react";

/**
 * Syncs local state with an external value WITHOUT using useEffect.
 *
 * When the external value changes, the hook returns the new value immediately
 * during render — the component sees the synced state in a single render pass.
 * When the local setter is called, the hook tracks that the user has diverged
 * from the external value and stops auto-syncing until the external value
 * changes again (indicating fresh data).
 *
 * This eliminates the "set-state-in-effect" anti-pattern where components
 * initialize state as empty and then populate it via useEffect when data
 * arrives, causing double-renders and state sync bugs.
 *
 * Pattern is React-sanctioned: "adjusting state during render" is documented
 * as a legitimate pattern for derived/prop-synced state.
 * (https://react.dev/reference/react/useState#storing-information-from-previous-renders)
 *
 * @example
 * // Before (anti-pattern):
 * const [name, setName] = useState("");
 * useEffect(() => {
 *   if (user) setName(user.name);
 * }, [user]); // eslint-disable-next-line react-hooks/set-state-in-effect
 *
 * // After (clean):
 * const [name, setName] = useDerivedStateFrom(user?.name ?? "");
 */
export function useDerivedStateFrom<T>(
  externalValue: T,
): [T, React.Dispatch<React.SetStateAction<T>>] {
  const [state, setState] = useState(externalValue);
  // Track previous external + whether the user has locally diverged using state
  // (not refs) so the render-path sync is legal under react-hooks/refs.
  const [prevExternal, setPrevExternal] = useState(externalValue);
  const [isLocal, setIsLocal] = useState(false);

  // Adjust state during render when external value changes.
  if (prevExternal !== externalValue) {
    setPrevExternal(externalValue);
    setIsLocal(false);
    setState(externalValue);
  }

  const setLocalState = useCallback<React.Dispatch<React.SetStateAction<T>>>((value) => {
    setIsLocal(true);
    setState((previousState) =>
      typeof value === "function"
        ? (value as (previous: T) => T)(previousState)
        : value,
    );
  }, []);

  // After a sync adjustment above, React will re-render with isLocal=false and
  // state=externalValue. Until then, prefer external when not locally edited.
  const value = isLocal && prevExternal === externalValue ? state : externalValue;
  return [value, setLocalState];
}

/**
 * Like useDerivedStateFrom but also exposes a `reset` function to manually
 * re-sync with the external value. Useful for forms that need a "Reset"
 * button without re-mounting the component.
 */
export function useDerivedStateFromWithReset<T>(
  externalValue: T,
): readonly [T, React.Dispatch<React.SetStateAction<T>>, () => void] {
  const [state, setState] = useState(externalValue);
  const [prevExternal, setPrevExternal] = useState(externalValue);
  const [isLocal, setIsLocal] = useState(false);

  if (prevExternal !== externalValue) {
    setPrevExternal(externalValue);
    setIsLocal(false);
    setState(externalValue);
  }

  const setLocalState = useCallback<React.Dispatch<React.SetStateAction<T>>>((value) => {
    setIsLocal(true);
    setState((previousState) =>
      typeof value === "function"
        ? (value as (previous: T) => T)(previousState)
        : value,
    );
  }, []);

  const reset = useCallback(() => {
    setIsLocal(false);
    setState(externalValue);
  }, [externalValue]);

  const value = isLocal && prevExternal === externalValue ? state : externalValue;
  return [value, setLocalState, reset] as const;
}
