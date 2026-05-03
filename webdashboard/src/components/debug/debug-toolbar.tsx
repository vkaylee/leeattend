import { useQueryClient } from "@tanstack/react-query";
import { Terminal, User, Calendar, X, RefreshCcw, Power, Database, Trash2, UserPlus, UserMinus } from "lucide-react";
import React, { useState, useEffect } from "react";

import { useToast } from "../../components/ui/toast";
import { useDebug } from "../../context/debug-context";
import { useConfirm } from "../../hooks/use-confirm";
import { debugApi, ClearDbGroup as ClearDatabaseGroup } from "../../lib/api/debug";
import { getCurrentWorkspaceId } from "../../lib/api/helpers";

type DebugTab = "identity" | "time" | "database";

export interface DebugUser {
  id: string;
  name: string;
  role: string;
}

interface DebugIdentityTabProps {
  debugUsers: DebugUser[];
  setImpersonateId: (id: string) => void;
  setTemporaryId: (id: string) => void;
  temporaryId: string;
}

function DebugIdentityTab({ debugUsers, temporaryId, setTemporaryId, setImpersonateId }: DebugIdentityTabProps) {
  return (
    <div className="space-y-3 animate-in fade-in duration-300">
      <div className="space-y-1.5">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase flex items-center gap-1" htmlFor="debug-user-select">
          Select User
        </label>
        <select 
          className="w-full bg-slate-800 border border-slate-700 rounded px-2 py-1.5 text-[11px] mb-2 focus:outline-none focus:border-blue-500"
          id="debug-user-select"
          onChange={(e) => { setTemporaryId(e.target.value); }}
          value={temporaryId}
        >
          <option value="">-- All Workspace Users --</option>
          {Array.isArray(debugUsers) && debugUsers.map((u) => (
            <option key={u.id} value={u.id}>{u.name} ({u.role})</option>
          ))}
        </select>
        <div className="flex gap-2">
          <input
            className="flex-1 bg-slate-800 border border-slate-700 rounded px-2 py-1.5 text-[11px] font-mono focus:outline-none focus:border-blue-500"
            onChange={(e) => { setTemporaryId(e.target.value); }}
            placeholder="Paste UUID..."
            type="text"
            value={temporaryId}
          />
          <button
            className="bg-blue-600 hover:bg-blue-50 dark:bg-blue-950 px-3 py-1.5 rounded text-[10px] font-bold"
            onClick={() => { setImpersonateId(temporaryId); }}
          >
            Apply
          </button>
        </div>
      </div>
    </div>
  );
}

interface DebugTimeTabProps {
  setMockDate: (date: string) => void;
  setTemporaryDate: (date: string) => void;
  temporaryDate: string;
}

function DebugTimeTab({ temporaryDate, setTemporaryDate, setMockDate }: DebugTimeTabProps) {
  return (
    <div className="space-y-3 animate-in fade-in duration-300">
      <div className="space-y-1.5">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase flex items-center gap-1" htmlFor="debug-date-input">
          Travel to Date
        </label>
        <div className="flex gap-2">
          <input
            className="flex-1 bg-slate-800 border border-slate-700 rounded px-2 py-1.5 text-[11px] focus:outline-none focus:border-teal-500 [color-scheme:dark]"
            id="debug-date-input"
            onChange={(e) => { setTemporaryDate(e.target.value); }}
            type="date"
            value={temporaryDate}
          />
          <button
            className="bg-teal-600 hover:bg-teal-50 dark:bg-teal-950 px-3 py-1.5 rounded text-[10px] font-bold"
            onClick={() => { setMockDate(temporaryDate); }}
          >
            Apply
          </button>
        </div>
        <p className="text-[9px] text-slate-500 dark:text-slate-400 italic mt-2">
          Note: Mock date affects payroll segments and policy resolution.
        </p>
      </div>
    </div>
  );
}

interface DebugDatabaseTabProps {
  handleClearDatabase: (group: ClearDatabaseGroup) => Promise<void>;
  isWiping: boolean;
}

function DebugDatabaseTab({ isWiping, handleClearDatabase }: DebugDatabaseTabProps) {
  return (
    <div className="space-y-3 animate-in fade-in duration-300">
      <div className="grid grid-cols-3 gap-2">
        <button
          className="flex flex-col items-center gap-1 bg-slate-800 hover:bg-slate-700 border border-slate-700 p-2 rounded transition-colors group"
          disabled={isWiping}
          onClick={() => handleClearDatabase("attendance")}
          title="Clear attendance logs and time entries"
        >
          <Trash2 className="text-slate-500 dark:text-slate-400 group-hover:text-amber-400" size={12} />
          <span className="text-[8px] font-bold uppercase">Logs</span>
        </button>
        <button
          className="flex flex-col items-center gap-1 bg-slate-800 hover:bg-slate-700 border border-slate-700 p-2 rounded transition-colors group"
          disabled={isWiping}
          onClick={() => handleClearDatabase("payroll")}
          title="Clear payroll history"
        >
          <Trash2 className="text-slate-500 dark:text-slate-400 group-hover:text-amber-400" size={12} />
          <span className="text-[8px] font-bold uppercase">Payroll</span>
        </button>
        <button
          className="flex flex-col items-center gap-1 bg-slate-800 hover:bg-slate-700 border border-slate-700 p-2 rounded transition-colors group"
          disabled={isWiping}
          onClick={() => handleClearDatabase("rosters")}
          title="Clear schedules and rosters"
        >
          <Trash2 className="text-slate-500 dark:text-slate-400 group-hover:text-amber-400" size={12} />
          <span className="text-[8px] font-bold uppercase">Rosters</span>
        </button>
      </div>

      <div className="grid grid-cols-2 gap-2 pt-2 border-t border-slate-800">
        <button
          className="flex flex-col items-center gap-1 bg-red-900/10 hover:bg-red-900/30 border border-red-900/30 p-2 rounded transition-colors group"
          disabled={isWiping}
          onClick={() => handleClearDatabase("employees")}
          title="Delete all employees and their data (keeps settings)"
        >
          <UserMinus className="text-red-400" size={14} />
          <span className="text-[9px] font-bold uppercase text-red-200">Wipe Staff</span>
        </button>
        <button
          className="flex flex-col items-center gap-1 bg-red-900/20 hover:bg-red-900/40 border border-red-900/50 p-2 rounded transition-colors group"
          disabled={isWiping}
          onClick={() => handleClearDatabase("full_reset")}
          title="Delete EVERYTHING (Factory Reset)"
        >
          <Trash2 className="text-red-400" size={14} />
          <span className="text-[9px] font-bold uppercase text-red-200">Factory Reset</span>
        </button>
      </div>
      
      <div className="pt-2 border-t border-slate-800">
        <button
          className="w-full flex items-center justify-center gap-2 bg-green-900/20 hover:bg-green-900/40 border border-green-900/50 p-2 rounded text-[10px] font-bold text-green-200 transition-colors"
          disabled={isWiping}
          onClick={() => handleClearDatabase("seed")}
          title="Create default departments, shifts, and employees"
        >
          <UserPlus size={14} /> Quick Seed Demo Team
        </button>
      </div>
    </div>
  );
}

interface DebugToolbarTabsProps {
  activeTab: DebugTab;
  setActiveTab: (tab: DebugTab) => void;
}

function DebugToolbarTabs({ activeTab, setActiveTab }: DebugToolbarTabsProps) {
  return (
    <div className="flex border-b border-slate-800 mt-2">
      <button 
        className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-tight flex items-center justify-center gap-1.5 transition-colors ${
          activeTab === "identity" ? "bg-slate-800 text-blue-400 border-b-2 border-blue-400" : "text-slate-500 dark:text-slate-400 hover:text-slate-300 dark:text-slate-600"
        }`}
        onClick={() => { setActiveTab("identity"); }}
      >
        <User size={12} /> Identity
      </button>
      <button 
        className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-tight flex items-center justify-center gap-1.5 transition-colors ${
          activeTab === "time" ? "bg-slate-800 text-teal-400 border-b-2 border-teal-400" : "text-slate-500 dark:text-slate-400 hover:text-slate-300 dark:text-slate-600"
        }`}
        onClick={() => { setActiveTab("time"); }}
      >
        <Calendar size={12} /> Time
      </button>
      <button 
        className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-tight flex items-center justify-center gap-1.5 transition-colors ${
          activeTab === "database" ? "bg-slate-800 text-red-400 border-b-2 border-red-400" : "text-slate-500 dark:text-slate-400 hover:text-slate-300 dark:text-slate-600"
        }`}
        onClick={() => { setActiveTab("database"); }}
      >
        <Database size={12} /> Database
      </button>
    </div>
  );
}

interface DebugToolbarHeaderProps {
  isEnabled: boolean;
  setIsEnabled: (enabled: boolean) => void;
  setIsOpen: (open: boolean) => void;
}

function DebugToolbarHeader({ isEnabled, setIsEnabled, setIsOpen }: DebugToolbarHeaderProps) {
  return (
    <>
      <div className="flex items-center justify-between p-3 border-b border-slate-800 bg-slate-950">
        <div className="flex items-center gap-2">
          <Terminal className="text-blue-400" size={16} />
          <span className="text-xs font-bold tracking-wider uppercase">Debug Suite v2.0</span>
        </div>
        <button className="hover:text-slate-500 dark:text-slate-400" onClick={() => { setIsOpen(false); }}>
          <X size={16} />
        </button>
      </div>

      <div className="px-4 pt-4">
        <div className="flex items-center justify-between p-2 bg-slate-800/50 rounded-lg">
          <div className="flex items-center gap-2">
            <Power className={isEnabled ? "text-green-400" : "text-slate-500 dark:text-slate-400"} size={14} />
            <span className="text-xs font-medium">Global Debug Mode</span>
          </div>
          <button
            className={`w-10 h-5 rounded-full relative transition-colors ${
              isEnabled ? "bg-green-600" : "bg-slate-600"
            }`}
            onClick={() => { setIsEnabled(!isEnabled); }}
          >
            <div className={`absolute top-1 w-3 h-3 bg-white dark:bg-slate-900 rounded-full transition-all ${
              isEnabled ? "left-6" : "left-1"
            }`} />
          </button>
        </div>
      </div>
    </>
  );
}

interface DebugToolbarFooterProps {
  isEnabled: boolean;
  reset: () => void;
  setTemporaryDate: (date: string) => void;
  setTemporaryId: (id: string) => void;
}

function DebugToolbarFooter({ isEnabled, reset, setTemporaryId, setTemporaryDate }: DebugToolbarFooterProps) {
  return (
    <div className="p-3 bg-slate-950 border-t border-slate-800 flex justify-between gap-2">
      <button
        className="flex-1 flex items-center justify-center gap-1.5 bg-slate-800 hover:bg-slate-700 py-2 rounded text-[10px] font-bold text-slate-300 dark:text-slate-600"
        onClick={() => {
          reset();
          setTemporaryId("");
          setTemporaryDate("");
          window.location.reload();
        }}
      >
        <RefreshCcw size={12} /> Clear Settings
      </button>
      {isEnabled ? (
        <button
          className="flex-1 flex items-center justify-center gap-1.5 bg-blue-600 hover:bg-blue-50 dark:bg-blue-950 py-2 rounded text-[10px] font-bold"
          onClick={() => { window.location.reload(); }}
        >
          Reload UI
        </button>
      ) : null}
    </div>
  );
}

export const DebugToolbar: React.FC = () => {
  const queryClient = useQueryClient();
  const toast = useToast();
  const { isEnabled, impersonateId, mockDate, setIsEnabled, setImpersonateId, setMockDate, reset } = useDebug();
  const { confirm, ConfirmationDialog } = useConfirm();
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<DebugTab>("identity");
  const [temporaryId, setTemporaryId] = useState(impersonateId ?? "");
  const [temporaryDate, setTemporaryDate] = useState(mockDate ?? "");
 
  const [debugUsers, setDebugUsers] = useState<DebugUser[]>([]);
  const [isWiping, setIsWiping] = useState(false);

  useEffect(() => {
    if (isOpen && isEnabled && activeTab === "identity") {
      debugApi.getUsers()
        .then(res => {
          if (Array.isArray(res)) {
            setDebugUsers(res as DebugUser[]);
          } else {
            console.warn("Debug users API did not return an array:", res);
            setDebugUsers([]);
          }
        })
        .catch((error: unknown) => {
          console.error("Failed to fetch debug users:", error);
          setDebugUsers([]);
        });
    }
  }, [isOpen, isEnabled, activeTab]);

  if (!import.meta.env.DEV) return null;

  const currentWorkspaceId = getCurrentWorkspaceId();

  const handleClearDatabase = async (group: ClearDatabaseGroup) => {
    if (!currentWorkspaceId) { toast.error("No workspace selected"); return; }
    
    const actionVerb = group === "seed" ? "seed demo" : "wipe";
    const warning = group === "full_reset" ? " This will delete EVERYTHING." : group === "employees" ? " This will delete all employees and their history." : "";
    
    const ok = await confirm({
      title: "Confirm " + actionVerb,
      message: `Are you sure you want to ${actionVerb} ${group === "seed" ? "data" : group + " data"}?${warning}`,
      variant: group === "seed" ? "warning" : "danger"
    });
    if (!ok) return;
    
    setIsWiping(true);
    try {
      await debugApi.clearDb(currentWorkspaceId, group);
      void queryClient.invalidateQueries();
      toast.success(`${group === "seed" ? "Demo data seeded" : group + " data wiped"} successfully.`);
    } catch (error) {
      toast.error("Operation failed: " + (error as Error).message);
    } finally {
      setIsWiping(false);
    }
  };

  if (!isOpen) {
    return (
      <button
        className="fixed bottom-4 right-4 bg-slate-900 text-white p-3 rounded-full shadow-2xl z-[9999] hover:bg-slate-800 transition-colors border border-slate-700"
        onClick={() => { setIsOpen(true); }}
        title="Open Debug Tools"
      >
        <Terminal className={isEnabled ? "text-green-400" : "text-white"} size={20} />
      </button>
    );
  }

  return (
    <div className="fixed bottom-4 right-4 bg-slate-900 text-white rounded-xl shadow-2xl z-[9999] border border-slate-700 w-80 overflow-hidden animate-in slide-in-from-bottom-4 duration-200">
      <DebugToolbarHeader isEnabled={isEnabled} setIsEnabled={setIsEnabled} setIsOpen={setIsOpen} />
      <DebugToolbarTabs activeTab={activeTab} setActiveTab={setActiveTab} />

      <div className={`p-4 min-h-[160px] ${!isEnabled ? "opacity-40 pointer-events-none" : ""}`}>
        {activeTab === "identity" && (
          <DebugIdentityTab debugUsers={debugUsers} setImpersonateId={setImpersonateId} setTemporaryId={setTemporaryId} temporaryId={temporaryId} />
        )}

        {activeTab === "time" && (
          <DebugTimeTab setMockDate={setMockDate} setTemporaryDate={setTemporaryDate} temporaryDate={temporaryDate} />
        )}

        {activeTab === "database" && (
          <DebugDatabaseTab handleClearDatabase={handleClearDatabase} isWiping={isWiping} />
        )}
      </div>

      <DebugToolbarFooter isEnabled={isEnabled} reset={reset} setTemporaryDate={setTemporaryDate} setTemporaryId={setTemporaryId} />
      <ConfirmationDialog />
    </div>
  );
}
