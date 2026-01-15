import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
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
      // @ts-expect-error - The mutation return might not be the full agent depending on API but useAuthorizedAgents refetches
      onSave(updated || agent);
      onClose();
    } catch (e) {
      alert(t("agents.edit_modal.messages.update_error"));
    }
  }

  return (
    <div className="modal-backdrop">
      <div className="modal-content max-w-lg">
        <div className="flex items-center justify-between p-8 border-b border-slate-50">
          <div className="flex items-center gap-4">
            <div className="bg-primary-50 p-3 rounded-2xl text-primary-600">
              <UserCheck className="w-6 h-6" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-slate-900 tracking-tight">
                {t("agents.edit_modal.title")}
              </h2>
              <p className="text-[10px] text-slate-400 font-bold uppercase tracking-widest mt-0.5">{t("agents.edit_modal.subtitle")}</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-100 rounded-full transition-colors group"
          >
            <X className="w-5 h-5 text-slate-300 group-hover:text-slate-600" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-8 space-y-8">
          <div className="space-y-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] ml-1">
                {t("agents.edit_modal.id_label")}
              </label>
              <input
                disabled
                className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 text-slate-400 font-mono font-bold cursor-not-allowed"
                value={agent.id}
              />
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] ml-1">
                {t("agents.edit_modal.name_label")}
              </label>
              <input
                required
                placeholder={t("agents.edit_modal.name_placeholder")}
                className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all font-bold text-slate-700"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] ml-1">
                {t("agents.edit_modal.token_label")}
              </label>
              <div className="relative">
                <input
                  required
                  placeholder={t("agents.edit_modal.token_placeholder")}
                  className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:border-primary-300 outline-none transition-all pr-12 font-mono font-bold text-slate-700"
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
                {isTrusted ? t("agents.status.trusted") : t("agents.status.pending")}
              </button>

              <button
                type="button"
                onClick={() => setIsActive(!isActive)}
                className={`flex items-center justify-center gap-2 py-4 rounded-2xl border transition-all font-black text-xs uppercase tracking-widest ${
                  isActive
                    ? "bg-primary-50 border-primary-100 text-primary-700 shadow-sm"
                    : "bg-rose-50 border-rose-100 text-rose-700 shadow-sm"
                }`}
              >
                <Power className="w-4 h-4" />
                {isActive ? t("common.status_active") : t("common.status_inactive")}
              </button>
            </div>
          </div>

          <div className="flex gap-4 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-8 py-4 rounded-2xl font-bold text-slate-500 hover:bg-slate-50 transition-all border border-slate-100"
            >
              {t("common.cancel")}
            </button>
            <button
              type="submit"
              disabled={isUpdating}
              className="flex-1 bg-primary-600 text-white px-8 py-4 rounded-2xl font-black uppercase tracking-widest text-xs hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 flex items-center justify-center gap-2 disabled:opacity-50 hover:scale-[1.02] active:scale-95"
            >
              <Save className="w-5 h-5" />
              {isUpdating ? t("common.saving") : t("common.save_changes")}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
