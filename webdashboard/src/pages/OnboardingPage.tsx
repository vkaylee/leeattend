import React, { useState } from "react";
import { LayoutDashboard, Plus } from "lucide-react";
import { useWorkspaces } from "@/hooks/use-workspace";
import { setCurrentWorkspaceId } from "@/lib/api";
import { useNavigate } from "react-router-dom";

export default function OnboardingPage() {
  const [name, setName] = useState("");
  const { createWorkspace } = useWorkspaces();
  const navigate = useNavigate();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    setIsSubmitting(true);
    try {
      const ws = await createWorkspace(name);
      setCurrentWorkspaceId(ws.id);
      navigate("/", { replace: true });
    } catch (error) {
      console.error("Failed to create workspace:", error);
      alert("Không thể tạo Workspace. Vui lòng thử lại.");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="min-h-screen bg-slate-50 flex flex-col items-center justify-center p-6">
      <div className="w-full max-w-md bg-white rounded-3xl shadow-xl shadow-slate-200 border border-slate-100 p-8">
        <div className="flex flex-col items-center gap-4 mb-8">
          <div className="bg-amber-500 p-3 rounded-2xl shadow-lg shadow-amber-200">
            <LayoutDashboard className="text-white w-8 h-8" />
          </div>
          <div className="text-center">
            <h1 className="text-2xl font-bold text-slate-900">
              Chào mừng bạn!
            </h1>
            <p className="text-slate-500 font-medium">
              Hãy tạo tổ chức (Workspace) đầu tiên của bạn
            </p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="space-y-2">
            <label className="text-sm font-semibold text-slate-700 ml-1">
              Tên tổ chức
            </label>
            <input
              autoFocus
              type="text"
              placeholder="Ví dụ: Công ty ABC, LeeAttend Team..."
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-4 py-3 bg-slate-50 border border-slate-200 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-amber-500 transition-all"
              disabled={isSubmitting}
            />
          </div>

          <button
            type="submit"
            disabled={isSubmitting || !name.trim()}
            className="w-full py-3 bg-slate-900 text-white rounded-xl font-bold hover:bg-slate-800 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
          >
            {isSubmitting ? (
              "Đang tạo..."
            ) : (
              <>
                <Plus className="w-5 h-5" />
                Bắt đầu ngay
              </>
            )}
          </button>
        </form>

        <p className="mt-8 text-[10px] text-slate-400 text-center uppercase tracking-widest leading-relaxed">
          Workspace là không gian làm việc riêng biệt giúp bạn quản lý thiết bị
          và thành viên một cách tập trung.
        </p>
      </div>
    </div>
  );
}
