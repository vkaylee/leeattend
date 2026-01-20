import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, LayoutDashboard, Loader2 } from "lucide-react";
import { useWorkspaces } from "@/hooks/use-workspace";
import Modal from "./ui/Modal";

interface CreateWorkspaceDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function CreateWorkspaceDialog({
  isOpen,
  onClose,
}: CreateWorkspaceDialogProps) {
  const { t } = useTranslation();
  const { createWorkspace } = useWorkspaces();
  const [name, setName] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    setIsSubmitting(true); setError(null);
    try {
      await createWorkspace({ name });
      setName(""); onClose();
    } catch (e) {
      setError(t("common.create_workspace_error"));
    } finally { setIsSubmitting(false); }
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("common.create_workspace")}
      subtitle={t("common.leeattend_system")}
      icon={<LayoutDashboard size={24} />}
      maxWidth="md"
    >
      <form onSubmit={handleSubmit} className="space-y-8 pt-4">
        {error && (
          <div className="p-4 text-[11px] font-black uppercase tracking-widest text-rose-600 bg-rose-50 border border-rose-100 rounded-2xl animate-in slide-in-from-top-2">
            {error}
          </div>
        )}

        <div className="space-y-3">
          <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
            {t("common.org_name_label")}
          </label>
          <input
            autoFocus
            type="text"
            required
            className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight"
            placeholder={t("common.org_name_placeholder")}
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={isSubmitting}
          />
        </div>

        <div className="pt-2">
          <button
            type="submit"
            disabled={isSubmitting || !name.trim()}
            className="w-full flex items-center justify-center gap-3 bg-slate-900 text-white py-4 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95 disabled:opacity-50"
          >
            {isSubmitting ? (
              <Loader2 size={18} className="animate-spin" />
            ) : (
              <Plus size={18} />
            )}
            {t("common.create_workspace").toUpperCase()}
          </button>
        </div>
      </form>
    </Modal>
  );
}
