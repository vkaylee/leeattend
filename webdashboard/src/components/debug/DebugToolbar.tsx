import React, { useState } from "react";
import { useDebug } from "../../context/DebugContext";
import { Terminal, User, Calendar, X, RefreshCcw, Power, Database, Trash2, UserPlus, UserMinus } from "lucide-react";
import { debugApi, ClearDbGroup } from "../../lib/api/debug";
import { getCurrentWorkspaceId } from "../../lib/api/helpers";
import { useQueryClient } from "@tanstack/react-query";

type DebugTab = "identity" | "time" | "database";

export const DebugToolbar: React.FC = () => {
  const queryClient = useQueryClient();
  const { isEnabled, impersonateId, mockDate, setIsEnabled, setImpersonateId, setMockDate, reset } = useDebug();
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<DebugTab>("identity");
  const [tempId, setTempId] = useState(impersonateId || "");
  const [tempDate, setTempDate] = useState(mockDate || "");
  const [debugUsers, setDebugUsers] = useState<any[]>([]);
  const [isWiping, setIsWiping] = useState(false);

  React.useEffect(() => {
    if (isOpen && isEnabled && activeTab === "identity") {
      debugApi.getUsers()
        .then(res => {
          if (Array.isArray(res)) {
            setDebugUsers(res);
          } else {
            console.warn("Debug users API did not return an array:", res);
            setDebugUsers([]);
          }
        })
        .catch(err => {
          console.error("Failed to fetch debug users:", err);
          setDebugUsers([]);
        });
    }
  }, [isOpen, isEnabled, activeTab]);

  // Only show in DEV
  if (!import.meta.env.DEV) return null;

  const currentWorkspaceId = getCurrentWorkspaceId();

  const handleClearDb = async (group: ClearDbGroup) => {
    if (!currentWorkspaceId) return alert("No workspace selected");
    
    const actionVerb = group === "seed" ? "seed demo" : "wipe";
    const warning = group === "full_reset" ? " This will delete EVERYTHING." : group === "employees" ? " This will delete all employees and their history." : "";
    
    if (!confirm(`Are you sure you want to ${actionVerb} ${group === "seed" ? "data" : group + " data"}?${warning}`)) return;
    
    setIsWiping(true);
    try {
      await debugApi.clearDb(currentWorkspaceId, group);
      // Invalidate all queries to refresh the UI
      queryClient.invalidateQueries();
      alert(`${group === "seed" ? "Demo data seeded" : group + " data wiped"} successfully.`);
    } catch (err) {
      alert("Operation failed: " + (err as Error).message);
    } finally {
      setIsWiping(false);
    }
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="fixed bottom-4 right-4 bg-slate-900 text-white p-3 rounded-full shadow-2xl z-[9999] hover:bg-slate-800 transition-colors border border-slate-700"
        title="Open Debug Tools"
      >
        <Terminal size={20} className={isEnabled ? "text-green-400" : "text-white"} />
      </button>
    );
  }

  return (
    <div className="fixed bottom-4 right-4 bg-slate-900 text-white rounded-xl shadow-2xl z-[9999] border border-slate-700 w-80 overflow-hidden animate-in slide-in-from-bottom-4 duration-200">
      <div className="flex items-center justify-between p-3 border-b border-slate-800 bg-slate-950">
        <div className="flex items-center gap-2">
          <Terminal size={16} className="text-blue-400" />
          <span className="text-xs font-bold tracking-wider uppercase">Debug Suite v2.0</span>
        </div>
        <button onClick={() => setIsOpen(false)} className="hover:text-slate-500 dark:text-slate-400">
          <X size={16} />
        </button>
      </div>

      {/* Mode Toggle */}
      <div className="px-4 pt-4">
        <div className="flex items-center justify-between p-2 bg-slate-800/50 rounded-lg">
          <div className="flex items-center gap-2">
            <Power size={14} className={isEnabled ? "text-green-400" : "text-slate-500 dark:text-slate-400"} />
            <span className="text-xs font-medium">Global Debug Mode</span>
          </div>
          <button
            onClick={() => setIsEnabled(!isEnabled)}
            className={`w-10 h-5 rounded-full relative transition-colors ${
              isEnabled ? "bg-green-600" : "bg-slate-600"
            }`}
          >
            <div className={`absolute top-1 w-3 h-3 bg-white dark:bg-slate-900 rounded-full transition-all ${
              isEnabled ? "left-6" : "left-1"
            }`} />
          </button>
        </div>
      </div>

      {/* Tabs Header */}
      <div className="flex border-b border-slate-800 mt-2">
        <button 
          onClick={() => setActiveTab("identity")}
          className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-tight flex items-center justify-center gap-1.5 transition-colors ${
            activeTab === "identity" ? "bg-slate-800 text-blue-400 border-b-2 border-blue-400" : "text-slate-500 dark:text-slate-400 hover:text-slate-300 dark:text-slate-600"
          }`}
        >
          <User size={12} /> Identity
        </button>
        <button 
          onClick={() => setActiveTab("time")}
          className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-tight flex items-center justify-center gap-1.5 transition-colors ${
            activeTab === "time" ? "bg-slate-800 text-purple-400 border-b-2 border-purple-400" : "text-slate-500 dark:text-slate-400 hover:text-slate-300 dark:text-slate-600"
          }`}
        >
          <Calendar size={12} /> Time
        </button>
        <button 
          onClick={() => setActiveTab("database")}
          className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-tight flex items-center justify-center gap-1.5 transition-colors ${
            activeTab === "database" ? "bg-slate-800 text-red-400 border-b-2 border-red-400" : "text-slate-500 dark:text-slate-400 hover:text-slate-300 dark:text-slate-600"
          }`}
        >
          <Database size={12} /> Database
        </button>
      </div>

      <div className={`p-4 min-h-[160px] ${!isEnabled ? "opacity-40 pointer-events-none" : ""}`}>
        {activeTab === "identity" && (
          <div className="space-y-3 animate-in fade-in duration-300">
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase flex items-center gap-1">
                Select User
              </label>
              <select 
                className="w-full bg-slate-800 border border-slate-700 rounded px-2 py-1.5 text-[11px] mb-2 focus:outline-none focus:border-blue-500"
                onChange={(e) => setTempId(e.target.value)}
                value={tempId}
              >
                <option value="">-- All Workspace Users --</option>
                {Array.isArray(debugUsers) && debugUsers.map(u => (
                  <option key={u.id} value={u.id}>{u.name} ({u.role})</option>
                ))}
              </select>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={tempId}
                  onChange={(e) => setTempId(e.target.value)}
                  placeholder="Paste UUID..."
                  className="flex-1 bg-slate-800 border border-slate-700 rounded px-2 py-1.5 text-[11px] font-mono focus:outline-none focus:border-blue-500"
                />
                <button
                  onClick={() => setImpersonateId(tempId)}
                  className="bg-blue-600 hover:bg-blue-50 dark:bg-blue-950 px-3 py-1.5 rounded text-[10px] font-bold"
                >
                  Apply
                </button>
              </div>
            </div>
          </div>
        )}

        {activeTab === "time" && (
          <div className="space-y-3 animate-in fade-in duration-300">
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase flex items-center gap-1">
                Travel to Date
              </label>
              <div className="flex gap-2">
                <input
                  type="date"
                  value={tempDate}
                  onChange={(e) => setTempDate(e.target.value)}
                  className="flex-1 bg-slate-800 border border-slate-700 rounded px-2 py-1.5 text-[11px] focus:outline-none focus:border-purple-500 [color-scheme:dark]"
                />
                <button
                  onClick={() => setMockDate(tempDate)}
                  className="bg-purple-600 hover:bg-purple-50 dark:bg-purple-950 px-3 py-1.5 rounded text-[10px] font-bold"
                >
                  Apply
                </button>
              </div>
              <p className="text-[9px] text-slate-500 dark:text-slate-400 italic mt-2">
                Note: Mock date affects payroll segments and policy resolution.
              </p>
            </div>
          </div>
        )}

        {activeTab === "database" && (
          <div className="space-y-3 animate-in fade-in duration-300">
             {/* Data Operations */}
            <div className="grid grid-cols-3 gap-2">
              <button
                onClick={() => handleClearDb("attendance")}
                disabled={isWiping}
                className="flex flex-col items-center gap-1 bg-slate-800 hover:bg-slate-700 border border-slate-700 p-2 rounded transition-colors group"
                title="Clear attendance logs and time entries"
              >
                <Trash2 size={12} className="text-slate-500 dark:text-slate-400 group-hover:text-amber-400" />
                <span className="text-[8px] font-bold uppercase">Logs</span>
              </button>
              <button
                onClick={() => handleClearDb("payroll")}
                disabled={isWiping}
                className="flex flex-col items-center gap-1 bg-slate-800 hover:bg-slate-700 border border-slate-700 p-2 rounded transition-colors group"
                title="Clear payroll history"
              >
                <Trash2 size={12} className="text-slate-500 dark:text-slate-400 group-hover:text-amber-400" />
                <span className="text-[8px] font-bold uppercase">Payroll</span>
              </button>
              <button
                onClick={() => handleClearDb("rosters")}
                disabled={isWiping}
                className="flex flex-col items-center gap-1 bg-slate-800 hover:bg-slate-700 border border-slate-700 p-2 rounded transition-colors group"
                title="Clear schedules and rosters"
              >
                <Trash2 size={12} className="text-slate-500 dark:text-slate-400 group-hover:text-amber-400" />
                <span className="text-[8px] font-bold uppercase">Rosters</span>
              </button>
            </div>

            {/* Destructive Zone */}
            <div className="grid grid-cols-2 gap-2 pt-2 border-t border-slate-800">
                <button
                onClick={() => handleClearDb("employees")}
                disabled={isWiping}
                className="flex flex-col items-center gap-1 bg-red-900/10 hover:bg-red-900/30 border border-red-900/30 p-2 rounded transition-colors group"
                title="Delete all employees and their data (keeps settings)"
              >
                <UserMinus size={14} className="text-red-400" />
                <span className="text-[9px] font-bold uppercase text-red-200">Wipe Staff</span>
              </button>
              <button
                onClick={() => handleClearDb("full_reset")}
                disabled={isWiping}
                className="flex flex-col items-center gap-1 bg-red-900/20 hover:bg-red-900/40 border border-red-900/50 p-2 rounded transition-colors group"
                title="Delete EVERYTHING (Factory Reset)"
              >
                <Trash2 size={14} className="text-red-400" />
                <span className="text-[9px] font-bold uppercase text-red-200">Factory Reset</span>
              </button>
            </div>
            
            {/* Seeding */}
            <div className="pt-2 border-t border-slate-800">
               <button
                onClick={() => handleClearDb("seed")}
                disabled={isWiping}
                className="w-full flex items-center justify-center gap-2 bg-green-900/20 hover:bg-green-900/40 border border-green-900/50 p-2 rounded text-[10px] font-bold text-green-200 transition-colors"
                title="Create default departments, shifts, and employees"
              >
                <UserPlus size={14} /> Quick Seed Demo Team
              </button>
            </div>
          </div>
        )}
      </div>

      <div className="p-3 bg-slate-950 border-t border-slate-800 flex justify-between gap-2">
        <button
          onClick={() => {
            reset();
            setTempId("");
            setTempDate("");
            window.location.reload();
          }}
          className="flex-1 flex items-center justify-center gap-1.5 bg-slate-800 hover:bg-slate-700 py-2 rounded text-[10px] font-bold text-slate-300 dark:text-slate-600"
        >
          <RefreshCcw size={12} /> Clear Settings
        </button>
        {isEnabled && (
          <button
            onClick={() => window.location.reload()}
            className="flex-1 flex items-center justify-center gap-1.5 bg-blue-600 hover:bg-blue-50 dark:bg-blue-950 py-2 rounded text-[10px] font-bold"
          >
            Reload UI
          </button>
        )}
      </div>
    </div>
  );
};

