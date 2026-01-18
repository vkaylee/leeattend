import React, { useState } from "react";
import { useDebug } from "../../context/DebugContext";
import { Terminal, User, Calendar, X, RefreshCcw, Power } from "lucide-react";

export const DebugToolbar: React.FC = () => {
  const { isEnabled, impersonateId, mockDate, setIsEnabled, setImpersonateId, setMockDate, reset } = useDebug();
  const [isOpen, setIsOpen] = useState(false);
  const [tempId, setTempId] = useState(impersonateId || "");
  const [tempDate, setTempDate] = useState(mockDate || "");
  const [debugUsers, setDebugUsers] = useState<any[]>([]);

  React.useEffect(() => {
    if (isOpen && isEnabled) {
      fetch("/api/debug/users", {
        headers: {
            "Authorization": `Bearer ${localStorage.getItem("access_token")}`
        }
      })
        .then(res => res.json())
        .then(setDebugUsers)
        .catch(console.error);
    }
  }, [isOpen, isEnabled]);

  // Only show in DEV
  if (!import.meta.env.DEV) return null;

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
          <span className="text-xs font-bold tracking-wider uppercase">Advanced Debug Suite</span>
        </div>
        <button onClick={() => setIsOpen(false)} className="hover:text-slate-400">
          <X size={16} />
        </button>
      </div>

      <div className="p-4 space-y-4">
        {/* Enable Toggle */}
        <div className="flex items-center justify-between p-2 bg-slate-800/50 rounded-lg">
          <div className="flex items-center gap-2">
            <Power size={14} className={isEnabled ? "text-green-400" : "text-slate-500"} />
            <span className="text-xs font-medium">Debug Mode</span>
          </div>
          <button
            onClick={() => setIsEnabled(!isEnabled)}
            className={`w-10 h-5 rounded-full relative transition-colors ${
              isEnabled ? "bg-green-600" : "bg-slate-600"
            }`}
          >
            <div className={`absolute top-1 w-3 h-3 bg-white rounded-full transition-all ${
              isEnabled ? "left-6" : "left-1"
            }`} />
          </button>
        </div>

        <div className={!isEnabled ? "opacity-40 pointer-events-none" : ""}>
          {/* Impersonation */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-slate-500 uppercase flex items-center gap-1">
              <User size={10} /> Impersonate User
            </label>
            <select 
              className="w-full bg-slate-800 border border-slate-700 rounded px-2 py-1 text-[11px] mb-2 focus:outline-none"
              onChange={(e) => setTempId(e.target.value)}
              value={tempId}
            >
              <option value="">-- Select User --</option>
              {debugUsers.map(u => (
                <option key={u.id} value={u.id}>{u.name} ({u.role})</option>
              ))}
            </select>
            <div className="flex gap-2">
              <input
                type="text"
                value={tempId}
                onChange={(e) => setTempId(e.target.value)}
                placeholder="Or paste ID..."
                className="flex-1 bg-slate-800 border border-slate-700 rounded px-2 py-1 text-[11px] font-mono focus:outline-none focus:border-blue-500"
              />
              <button
                onClick={() => setImpersonateId(tempId)}
                className="bg-blue-600 hover:bg-blue-500 px-2 py-1 rounded text-[10px] font-bold"
              >
                Apply
              </button>
            </div>
          </div>

          {/* Time Travel */}
          <div className="space-y-1.5 mt-4">
            <label className="text-[10px] font-bold text-slate-500 uppercase flex items-center gap-1">
              <Calendar size={10} /> System Mock Date
            </label>
            <div className="flex gap-2">
              <input
                type="date"
                value={tempDate}
                onChange={(e) => setTempDate(e.target.value)}
                className="flex-1 bg-slate-800 border border-slate-700 rounded px-2 py-1 text-[11px] focus:outline-none focus:border-blue-500 [color-scheme:dark]"
              />
              <button
                onClick={() => setMockDate(tempDate)}
                className="bg-purple-600 hover:bg-purple-500 px-2 py-1 rounded text-[10px] font-bold"
              >
                Apply
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="p-3 bg-slate-950 border-t border-slate-800 flex justify-between gap-2">
        <button
          onClick={() => {
            reset();
            setTempId("");
            setTempDate("");
            window.location.reload();
          }}
          className="flex-1 flex items-center justify-center gap-1.5 bg-slate-800 hover:bg-slate-700 py-1.5 rounded text-[10px] font-bold text-slate-300"
        >
          <RefreshCcw size={12} /> Full Reset
        </button>
        {isEnabled && (
          <button
            onClick={() => window.location.reload()}
            className="flex-1 flex items-center justify-center gap-1.5 bg-green-600 hover:bg-green-500 py-1.5 rounded text-[10px] font-bold"
          >
            Reload to Apply
          </button>
        )}
      </div>
    </div>
  );
};
