import { useEffect } from "react";

const lockedContainers = new Map<HTMLElement, { count: number; overflow: string }>();

export function useScrollLock(isLocked: boolean) {
  useEffect(() => {
    if (!isLocked) return;

    const container = document.querySelector<HTMLElement>("[data-app-scroll-container]");
    if (!container) return;

    const existing = lockedContainers.get(container);
    if (existing) {
      existing.count += 1;
    } else {
      lockedContainers.set(container, { count: 1, overflow: container.style.overflow });
      container.style.overflow = "hidden";
    }

    return () => {
      const current = lockedContainers.get(container);
      if (!current) return;

      if (current.count === 1) {
        container.style.overflow = current.overflow;
        lockedContainers.delete(container);
      } else {
        current.count -= 1;
      }
    };
  }, [isLocked]);
}
