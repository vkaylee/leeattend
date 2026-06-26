import React, { createContext, useContext, useState, useMemo } from "react";

interface HeaderActionsContextValue {
  actions: React.ReactNode;
  setActions: (node: React.ReactNode) => void;
}

const HeaderActionsContext = createContext<HeaderActionsContextValue>({
  actions: null,
  setActions: () => {},
});

export function HeaderActionsProvider({ children }: { children: React.ReactNode }) {
  const [actions, setActions] = useState<React.ReactNode>(null);
  const value = useMemo(() => ({ actions, setActions }), [actions]);
  return (
    <HeaderActionsContext.Provider value={value}>
      {children}
    </HeaderActionsContext.Provider>
  );
}

export function useHeaderActions(): HeaderActionsContextValue {
  return useContext(HeaderActionsContext);
}
