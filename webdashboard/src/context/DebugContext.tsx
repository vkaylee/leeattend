import React, { createContext, useContext, useState, useEffect } from "react";

interface DebugState {
  impersonateId: string | null;
  mockDate: string | null;
  isEnabled: boolean;
}

interface DebugContextType extends DebugState {
  setImpersonateId: (id: string | null) => void;
  setMockDate: (date: string | null) => void;
  setIsEnabled: (enabled: boolean) => void;
  reset: () => void;
}

const DebugContext = createContext<DebugContextType | undefined>(undefined);

export const DebugProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [isEnabled, setIsEnabled] = useState(() => {
    return localStorage.getItem("debug_enabled") === "true";
  });
  
  const [impersonateId, setImpersonateId] = useState(() => {
    return localStorage.getItem("debug_impersonate_id");
  });
  
  const [mockDate, setMockDate] = useState(() => {
    return localStorage.getItem("debug_mock_date");
  });

  useEffect(() => {
    localStorage.setItem("debug_enabled", String(isEnabled));
  }, [isEnabled]);

  useEffect(() => {
    if (impersonateId) {
      localStorage.setItem("debug_impersonate_id", impersonateId);
    } else {
      localStorage.removeItem("debug_impersonate_id");
    }
  }, [impersonateId]);

  useEffect(() => {
    if (mockDate) {
      localStorage.setItem("debug_mock_date", mockDate);
    } else {
      localStorage.removeItem("debug_mock_date");
    }
  }, [mockDate]);

  const reset = () => {
    setImpersonateId(null);
    setMockDate(null);
    setIsEnabled(false);
  };

  // Only enable in development mode
  const isDev = import.meta.env.DEV;

  return (
    <DebugContext.Provider value={{ 
      isEnabled: isDev && isEnabled, 
      impersonateId, 
      mockDate, 
      setIsEnabled, 
      setImpersonateId, 
      setMockDate, 
      reset 
    }}>
      {children}
    </DebugContext.Provider>
  );
};

// eslint-disable-next-line react-refresh/only-export-components
export const useDebug = () => {
  const context = useContext(DebugContext);
  if (!context) {
    throw new Error("useDebug must be used within a DebugProvider");
  }
  return context;
};
