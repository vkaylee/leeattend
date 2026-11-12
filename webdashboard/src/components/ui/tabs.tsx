import React, { createContext, memo, useCallback, useContext, useId, useMemo } from "react";

import { type ControlSize, CONTROL_SIZE_STYLES, CONTROL_TEXT_STYLE } from "@/components/ui/control-styles";
import { cn } from "@/lib/utils";

interface TabsContextProps {
  baseId: string;
  onValueChange: (value: string) => void;
  value: string;
}

const TabsContext = createContext<TabsContextProps | undefined>(undefined);

export interface TabsProps {
  children: React.ReactNode;
  className?: string;
  onValueChange: (value: string) => void;
  value: string;
}

export const Tabs = memo(function Tabs({ value, onValueChange, children, className }: TabsProps) {
  const generatedId = useId();
  const contextValue = useMemo(
    () => ({ baseId: `tabs-${generatedId}`, value, onValueChange }),
    [generatedId, onValueChange, value]
  );

  return (
    <TabsContext.Provider value={contextValue}>
      <div className={cn("w-full", className)}>{children}</div>
    </TabsContext.Provider>
  );
});
Tabs.displayName = "Tabs";

export interface TabsListProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export const TabsList = memo(function TabsList({ children, className, ...props }: TabsListProps) {
  return (
    <div {...props} className={cn("flex gap-2 border-b border-slate-100 dark:border-slate-800", className)} role="tablist">
      {children}
    </div>
  );
});
TabsList.displayName = "TabsList";

type TabsTriggerProps = {
  value: string;
  children: React.ReactNode;
  className?: string;
  size?: ControlSize;
} & Omit<React.ComponentPropsWithoutRef<"button">, "className" | "children">;

export const TabsTrigger = memo(function TabsTrigger({
  value,
  children,
  className,
  size = "md",
  disabled = false,
  onClick,
  onKeyDown,
  ...props
}: TabsTriggerProps) {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsTrigger must be used within Tabs");

  const isActive = context.value === value;
  const triggerId = `${context.baseId}-trigger-${value}`;
  const contentId = `${context.baseId}-content-${value}`;

  const select = useCallback(() => {
    if (!disabled) context.onValueChange(value);
  }, [context.onValueChange, disabled, value]);

  const handleClick = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      onClick?.(event);
      if (!event.defaultPrevented) select();
    },
    [onClick, select]
  );

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>) => {
      onKeyDown?.(event);
      if (event.defaultPrevented || disabled) return;

      const list = event.currentTarget.parentElement;
      if (!list) return;
      const triggers = Array.from(list.querySelectorAll<HTMLButtonElement>('[role="tab"]:not(:disabled)'));
      const currentIndex = triggers.indexOf(event.currentTarget);
      if (currentIndex < 0) return;

      let nextIndex = currentIndex;
      if (event.key === "ArrowRight" || event.key === "ArrowDown") nextIndex = (currentIndex + 1) % triggers.length;
      if (event.key === "ArrowLeft" || event.key === "ArrowUp") nextIndex = (currentIndex - 1 + triggers.length) % triggers.length;
      if (event.key === "Home") nextIndex = 0;
      if (event.key === "End") nextIndex = triggers.length - 1;
      if (nextIndex === currentIndex) return;

      event.preventDefault();
      triggers[nextIndex]?.focus();
      const nextValue = triggers[nextIndex]?.dataset.value;
      if (nextValue) context.onValueChange(nextValue);
    },
    [context.onValueChange, disabled, onKeyDown]
  );

  return (
    <button
      {...props}
      aria-controls={contentId}
      aria-selected={isActive}
      className={cn(
        CONTROL_SIZE_STYLES[size],
        CONTROL_TEXT_STYLE,
        "-mb-[2px] cursor-pointer border-b-2 font-bold transition-all",
        isActive
          ? "border-primary-500 text-primary-600 dark:text-primary-400"
          : "border-transparent text-slate-500 hover:text-slate-600 dark:text-slate-400 dark:hover:text-slate-300",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500",
        className
      )}
      data-value={value}
      disabled={disabled}
      id={triggerId}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      role="tab"
      tabIndex={disabled ? -1 : isActive ? 0 : -1}
      type="button"
    >
      {children}
    </button>
  );
});
TabsTrigger.displayName = "TabsTrigger";

export interface TabsContentProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string;
  children: React.ReactNode;
}

export const TabsContent = memo(function TabsContent({ value, children, className, ...props }: TabsContentProps) {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsContent must be used within Tabs");
  if (context.value !== value) return null;

  return (
    <div
      {...props}
      aria-labelledby={`${context.baseId}-trigger-${value}`}
      className={cn("mt-6 animate-in fade-in slide-in-from-bottom-2 duration-300", className)}
      id={`${context.baseId}-content-${value}`}
      role="tabpanel"
      tabIndex={0}
    >
      {children}
    </div>
  );
});
TabsContent.displayName = "TabsContent";
