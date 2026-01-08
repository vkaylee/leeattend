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
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/50 backdrop-blur-sm animate-in fade-in duration-200">
      <div className="bg-white w-full max-w-lg rounded-3xl shadow-2xl border border-slate-100 overflow-hidden animate-in zoom-in-95 duration-200">
        <div className="flex items-center justify-between p-6 border-b border-slate-50">
          <div className="flex items-center gap-3">
            <div className="bg-blue-50 p-2 rounded-xl text-blue-600">
              <UserCheck className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-bold text-slate-900">
              Chỉnh sửa Agent
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-100 rounded-full transition-colors"
          >
            <X className="w-5 h-5 text-slate-400" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-6">
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                Agent ID (Read-only)
              </label>
              <input
                disabled
                className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 text-slate-400 font-mono cursor-not-allowed"
                value={agent.id}
              />
            </div>

            <div className="space-y-2">
              <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                Tên Chi nhánh/Vị trí
              </label>
              <input
                required
                placeholder="Ví dụ: Văn phòng Hà Nội"
                className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                Secret Token
              </label>
              <div className="relative">
                <input
                  required
                  placeholder="Mật mã kết nối"
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all pr-10"
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                />
                <Key className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 pt-2">
              <button
                type="button"
                onClick={() => setIsTrusted(!isTrusted)}
                className={`flex items-center justify-center gap-2 p-3 rounded-2xl border transition-all font-bold text-sm ${
                  isTrusted
                    ? "bg-emerald-50 border-emerald-100 text-emerald-700"
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
                className={`flex items-center justify-center gap-2 p-3 rounded-2xl border transition-all font-bold text-sm ${
                  isActive
                    ? "bg-blue-50 border-blue-100 text-blue-700"
                    : "bg-rose-50 border-rose-100 text-rose-700"
                }`}
              >
                <Power className="w-4 h-4" />
                {isActive ? "Hoạt động" : "Tạm dừng"}
              </button>
            </div>
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-6 py-3 rounded-2xl font-bold text-slate-600 hover:bg-slate-50 transition-all border border-slate-100"
            >
              Hủy
            </button>
            <button
              type="submit"
              disabled={isUpdating}
              className="flex-1 bg-blue-600 text-white px-6 py-3 rounded-2xl font-bold hover:bg-blue-700 transition-all shadow-lg shadow-blue-100 flex items-center justify-center gap-2 disabled:opacity-50"
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
