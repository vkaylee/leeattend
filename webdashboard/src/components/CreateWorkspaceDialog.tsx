import React, { useState } from "react";
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
  const { createWorkspace } = useWorkspaces();
  const [name, setName] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;

    setIsSubmitting(true);
    setError(null);
    try {
      await createWorkspace({ name });
      setName("");
      onClose();
    } catch (e) {
      setError("Không thể tạo Workspace. Vui lòng thử lại.");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Tạo Workspace"
      subtitle="Hệ thống LeeAttend"
      icon={<LayoutDashboard className="w-6 h-6" />}
    >
      <form onSubmit={handleSubmit} className="space-y-8">
        {error && (
          <div className="p-3 text-sm text-red-600 bg-red-50 rounded-lg">
            {error}
          </div>
        )}

        <div className="space-y-2">
          <label className="text-xs font-bold text-slate-500 uppercase tracking-widest ml-1">
            Tên tổ chức / Dự án
          </label>
          <input
            autoFocus
            type="text"
            required
            className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl focus:ring-2 focus:ring-primary-500 outline-none transition-all text-sm"
            placeholder="Ví dụ: Công ty ABC, Team Freelance..."
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={isSubmitting}
          />
        </div>

        <div className="pt-2">
          <button
            type="submit"
            disabled={isSubmitting || !name.trim()}
            className="w-full flex items-center justify-center gap-2 bg-slate-900 text-white py-3 rounded-xl font-bold hover:bg-slate-800 transition-all shadow-lg shadow-slate-200 text-sm disabled:opacity-50"
          >
            {isSubmitting ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Plus className="w-4 h-4" />
            )}
            Tạo Workspace
          </button>
        </div>
      </form>
    </Modal>
  );
}
