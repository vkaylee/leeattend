import React, { useState, useEffect } from "react";
import {
  X,
  Save,
  Key,
  UserCheck,
  ShieldCheck,
  ShieldAlert,
  Power,
} from "lucide-react";
import { AuthorizedAgent } from "@/lib/api";
import { useAuthorizedAgents } from "@/hooks/use-agents";

interface AgentEditModalProps {
  agent: AuthorizedAgent | null;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedAgent: AuthorizedAgent) => void;
}

export default function AgentEditModal({
  agent,
  isOpen,
  onClose,
  onSave,
}: AgentEditModalProps) {
  const { updateAgent, isUpdating } = useAuthorizedAgents();
  const [name, setName] = useState("");
  const [token, setToken] = useState("");
  const [isTrusted, setIsTrusted] = useState(false);
  const [isActive, setIsActive] = useState(true);

  useEffect(() => {
    if (agent) {
      setName(agent.name);
      setToken(agent.token);
      setIsTrusted(agent.is_trusted);
      setIsActive(agent.is_active);
    }
  }, [agent]);

  if (!isOpen || !agent) return null;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      const updated = await updateAgent({
        id: agent!.id,
        agent: {
          name,
          token,
          is_trusted: isTrusted,
          is_active: isActive,
        },
      });
      // @ts-ignore - The mutation return might not be the full agent depending on API but useAuthorizedAgents refetches
      onSave(updated || agent);
      onClose();
    } catch (e) {
      alert("Lỗi khi cập nhật Agent.");
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-md animate-in fade-in duration-300">
      <div className="bg-white w-full max-w-lg rounded-[2.5rem] shadow-2xl border border-white/20 overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="flex items-center justify-between p-8 border-b border-slate-50 bg-slate-50/50">
          <div className="flex items-center gap-4">
            <div className="bg-violet-100 p-3 rounded-2xl text-violet-600 shadow-inner">
              <UserCheck className="w-6 h-6" />
            </div>
            <h2 className="text-2xl font-black text-slate-900 tracking-tight">
              Chỉnh sửa Agent
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-3 hover:bg-white rounded-2xl transition-all shadow-sm active:scale-90"
          >
            <X className="w-6 h-6 text-slate-400" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-10 space-y-8">
          <div className="space-y-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] ml-1">
                Agent ID (Read-only)
              </label>
              <input
                disabled
                className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 text-slate-400 font-mono font-bold cursor-not-allowed"
                value={agent.id}
              />
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] ml-1">
                Tên Chi nhánh / Vị trí
              </label>
              <input
                required
                placeholder="Ví dụ: Văn phòng Hà Nội"
                className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-violet-500/10 focus:border-violet-300 outline-none transition-all font-bold text-slate-700"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] ml-1">
                Secret Token
              </label>
              <div className="relative">
                <input
                  required
                  placeholder="Mật mã kết nối"
                  className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-violet-500/10 focus:border-violet-300 outline-none transition-all pr-12 font-mono font-bold text-slate-700"
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                />
                <Key className="absolute right-4 top-4 w-5 h-5 text-slate-300" />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 pt-2">
              <button
                type="button"
                onClick={() => setIsTrusted(!isTrusted)}
                className={`flex items-center justify-center gap-2 py-4 rounded-2xl border transition-all font-black text-xs uppercase tracking-widest ${
                  isTrusted
                    ? "bg-emerald-50 border-emerald-100 text-emerald-700 shadow-sm"
                    : "bg-slate-50 border-slate-100 text-slate-400"
                }`}
              >
                {isTrusted ? (
                  <ShieldCheck className="w-4 h-4" />
                ) : (
                  <ShieldAlert className="w-4 h-4" />
                )}
                {isTrusted ? "Đã duyệt" : "Chờ duyệt"}
              </button>

              <button
                type="button"
                onClick={() => setIsActive(!isActive)}
                className={`flex items-center justify-center gap-2 py-4 rounded-2xl border transition-all font-black text-xs uppercase tracking-widest ${
                  isActive
                    ? "bg-violet-50 border-violet-100 text-violet-700 shadow-sm"
                    : "bg-rose-50 border-rose-100 text-rose-700 shadow-sm"
                }`}
              >
                <Power className="w-4 h-4" />
                {isActive ? "Hoạt động" : "Tạm dừng"}
              </button>
            </div>
          </div>

          <div className="flex gap-4 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-8 py-4 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all border border-slate-100"
            >
              Hủy bỏ
            </button>
            <button
              type="submit"
              disabled={isUpdating}
              className="flex-1 bg-violet-600 text-white px-8 py-4 rounded-2xl font-black uppercase tracking-widest text-xs hover:bg-violet-700 transition-all shadow-xl shadow-violet-200 flex items-center justify-center gap-2 disabled:opacity-50 hover:scale-[1.02] active:scale-95"
            >
              <Save className="w-5 h-5" />
              {isUpdating ? "Đang lưu..." : "Lưu thay đổi"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
