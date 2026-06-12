import React, { createContext, useContext } from "react";

import { cn } from "@/lib/utils";

interface TabsContextProps {
  onValueChange: (value: string) => void;
  value: string;
}

const TabsContext = createContext<TabsContextProps | undefined>(undefined);

export function Tabs({ value, onValueChange, children }: { value: string; onValueChange: (v: string) => void; children: React.ReactNode }) {
  return (
    <TabsContext.Provider value={{ value, onValueChange }}>
      <div className="w-full">{children}</div>
    </TabsContext.Provider>
  );
}

export function TabsList({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={`flex gap-2 border-b border-slate-100 dark:border-slate-800 ${className}`}>
      {children}
    </div>
  );
}

export function TabsTrigger({
  value,
  children,
  className = "",
  ...props
}: {
  value: string;
  children: React.ReactNode;
  className?: string;
  [key: string]: any;
}) {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsTrigger must be used within Tabs");

  const isActive = context.value === value;

  return (
    <button
      className={cn(
        "px-4 py-2 text-sm font-bold transition-all border-b-2 -mb-[2px] cursor-pointer",
        isActive
          ? "border-primary-500 text-primary-600 dark:text-primary-400"
          : "border-transparent text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300",
        className
      )}
      onClick={() => { context.onValueChange(value); }}
      {...props}
    >
      {children}
    </button>
  );
}

export function TabsContent({ value, children }: { value: string; children: React.ReactNode }) {
  const context = useContext(TabsContext);
  if (!context) throw new Error("TabsContent must be used within Tabs");

  if (context.value !== value) return null;

  return <div className="mt-6 animate-in fade-in slide-in-from-bottom-2 duration-300">{children}</div>;
}
