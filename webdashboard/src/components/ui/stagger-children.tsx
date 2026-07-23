import React, { Children, memo, useMemo } from "react";

import { cn } from "@/lib/utils";

interface StaggerChildrenProps {
  children: React.ReactNode;
  /** Container className */
  className?: string;
  /** Delay between each child in ms (default: 60) */
  delay?: number;
  /** Base animation duration in ms (default: 500) */
  duration?: number;
}

/**
 * Wraps each child in a container with staggered animation delay.
 * Each child fades in and slides up with an incremental delay.
 *
 * Respects `prefers-reduced-motion` via CSS media query — animations
 * are disabled entirely when the user prefers reduced motion.
 *
 * @example
 * <StaggerChildren delay={80} className="grid grid-cols-6 gap-4">
 *   <StatCard ... />  // delay: 0ms
 *   <StatCard ... />  // delay: 80ms
 *   <StatCard ... />  // delay: 160ms
 * </StaggerChildren>
 */
const StaggerChildren = memo(function StaggerChildren({
  delay = 60,
  duration = 500,
  className,
  children,
}: StaggerChildrenProps) {
  const childArray = useMemo(() => Children.toArray(children), [children]);

  return (
    <div className={cn(className)}>
      {childArray.map((child, index) => (
        <div
          className="animate-in fade-in slide-in-from-bottom-4 fill-mode-both motion-reduce:animate-none"
          key={index}
          style={{
            animationDelay: `${index * delay}ms`,
            animationDuration: `${duration}ms`,
          }}
        >
          {child}
        </div>
      ))}
    </div>
  );
});

StaggerChildren.displayName = "StaggerChildren";

export default StaggerChildren;
