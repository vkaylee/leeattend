import React, { createContext, memo, useCallback, useContext, useMemo } from "react";

import { cn } from "@/lib/utils";

interface TabsContextProps {
  onValueChange: (value: string) => void;
  value: string;
}

const TabsContext = createContext<TabsContextProps | undefined>(undefined);

export const Tabs = memo(function Tabs({ value, onValueChange, children }: { value: string; onValueChange: (v: string) => void; children: React.ReactNode }) {
  const contextValue = useMemo(() => ({ value, onValueChange }), [value, onValueChange]);
  return (
    <TabsContext.Provider value={contextValue}>
      <div className="w-full">{children}</div>
    </TabsContext.Provider>
  );
});
Tabs.displayName = "Tabs";

export const TabsList = memo(function TabsList({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={`flex gap-2 border-b border-slate-100 dark:border-slate-800 ${className}`}>
      {children}
    </div>
  );
});
TabsList.displayName = "TabsList";

type TabsTriggerProps = {
  value: string;
  children: React.ReactNode;
  className?: string;
} & React.ComponentPropsWithoutRef<"button">;

export const TabsTrigger = memo(function TabsTrigger({
  value,
  children,
  className = "",
  ...props
}: TabsTriggerProps) {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsTrigger must be used within Tabs");

  const isActive = context.value === value;

  const handleClick = useCallback(() => {
    context.onValueChange(value);
  }, [context, value]);

  return (
    <button
      className={cn(
        "px-4 py-2 text-sm font-bold transition-all border-b-2 -mb-[2px] cursor-pointer",
        isActive
          ? "border-primary-500 text-primary-600 dark:text-primary-400"
          : "border-transparent text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300",
        className
      )}
      onClick={handleClick}
      {...props}
    >
      {children}
    </button>
  );
});
TabsTrigger.displayName = "TabsTrigger";

export const TabsContent = memo(function TabsContent({ value, children }: { value: string; children: React.ReactNode }) {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsContent must be used within Tabs");

  if (context.value !== value) return null;

  return <div className="mt-6 animate-in fade-in slide-in-from-bottom-2 duration-300">{children}</div>;
});
TabsContent.displayName = "TabsContent";
