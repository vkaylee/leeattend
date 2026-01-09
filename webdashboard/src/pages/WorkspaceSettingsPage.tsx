import { useState } from "react";
import { Users, UserPlus, Shield, Mail, Trash2, Settings2 } from "lucide-react";
import { useWorkspaceMembers } from "@/hooks/use-workspace";
import RoleManagementModal from "@/components/RoleManagementModal";
import { useToast } from "../components/Toast";

export default function WorkspaceSettingsPage() {
  const { members, isLoading, addMember, isAdding } = useWorkspaceMembers();
  const toast = useToast();
  const [newMemberEmail, setNewMemberEmail] = useState("");
  const [newMemberRole, setNewMemberRole] = useState<"admin" | "member">(
    "member",
  );

  // State for role management
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [isRoleModalOpen, setIsRoleModalOpen] = useState(false);

  async function handleInvite(e: React.FormEvent) {
    e.preventDefault();
    try {
      await addMember({ email: newMemberEmail, role: newMemberRole });
      toast.success(`Đã gửi lời mời tới ${newMemberEmail} tham gia Workspace.`);
      setNewMemberEmail("");
    } catch (e) {
      toast.error("Lỗi khi mời thành viên. Người dùng có thể chưa đăng ký tài khoản LeeAttend.");
    }
  }

  return (
    <div className="p-8 animate-in fade-in duration-500">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
            Cài đặt Workspace
          </h1>
          <p className="text-slate-500 mt-1">
            Quản lý thành viên và quyền truy cập
          </p>
        </div>

        <div className="grid grid-cols-1 gap-8">
          {/* Invite Section */}
          <div className="bg-white p-8 rounded-3xl border border-slate-100 shadow-sm">
            <h2 className="text-xl font-bold mb-6 flex items-center gap-2 text-slate-800">
              <UserPlus className="w-5 h-5 text-blue-500" />
              Mời thành viên
            </h2>
            <form onSubmit={handleInvite} className="flex flex-wrap gap-4">
              <div className="flex-1 min-w-[240px] space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  Email
                </label>
                <div className="relative">
                  <input
                    required
                    type="email"
                    placeholder="dong-nghiep@congty.com"
                    className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all pr-10"
                    value={newMemberEmail}
                    onChange={(e) => setNewMemberEmail(e.target.value)}
                  />
                  <Mail className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
                </div>
              </div>
              <div className="w-40 space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  Vai trò
                </label>
                <select
                  className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all appearance-none font-bold text-slate-700"
                  value={newMemberRole}
                  onChange={(e) => setNewMemberRole(e.target.value as any)}
                >
                  <option value="member">Thành viên</option>
                  <option value="admin">Quản trị viên</option>
                </select>
              </div>
              <div className="flex items-end">
                <button
                  type="submit"
                  disabled={isAdding}
                  className="bg-blue-600 text-white px-8 py-3 rounded-xl font-bold hover:bg-blue-700 transition-all shadow-lg shadow-blue-100 h-[50px] disabled:opacity-50"
                >
                  {isAdding ? "Đang mời..." : "Gửi lời mời"}
                </button>
              </div>
            </form>
          </div>

          {/* Members List */}
          <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
            <div className="p-6 border-b border-slate-50 flex items-center justify-between">
              <h2 className="text-lg font-bold flex items-center gap-2 text-slate-800">
                <Users className="w-5 h-5 text-slate-400" />
                Danh sách thành viên
              </h2>
              <span className="text-xs font-black uppercase tracking-widest bg-slate-100 text-slate-500 px-3 py-1 rounded-full">
                {members.length} NGƯỜI
              </span>
            </div>

            <div className="divide-y divide-slate-50">
              {isLoading ? (
                <div className="p-12 text-center text-slate-400 font-medium italic">
                  Đang tải danh sách thành viên...
                </div>
              ) : (
                members.map((member) => (
                  <div
                    key={member.user_id}
                    className="p-6 flex items-center justify-between hover:bg-slate-50/50 transition-colors group"
                  >
                    <div className="flex items-center gap-4">
                      <div className={`w-12 h-12 rounded-2xl flex items-center justify-center font-bold shadow-sm transition-all ${
                        member.role === 'owner' ? 'bg-indigo-600 text-white shadow-indigo-100' : 
                        member.role === 'admin' ? 'bg-blue-600 text-white shadow-blue-100' : 'bg-slate-100 text-slate-500'
                      }`}>
                        {member.full_name ? member.full_name[0] : <Users className="w-5 h-5" />}
                      </div>
                      <div>
                        <p className="font-bold text-slate-900 flex items-center gap-2">
                          {member.full_name || member.email}
                          {member.role === "owner" && <span className="text-[10px] bg-indigo-50 text-indigo-600 px-1.5 py-0.5 rounded font-black uppercase tracking-tighter border border-indigo-100">Sáng lập</span>}
                        </p>
                        <p className="text-[10px] font-black uppercase tracking-widest text-slate-400">
                          {member.role === 'admin' ? 'Quản trị viên' : member.role === 'owner' ? 'Chủ sở hữu' : 'Thành viên'}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onClick={() => {
                          setSelectedUserId(member.user_id);
                          setIsRoleModalOpen(true);
                        }}
                        className="p-2.5 text-slate-400 hover:text-blue-600 hover:bg-blue-50 rounded-xl transition-all"
                        title="Quản lý quyền"
                      >
                        <Settings2 className="w-5 h-5" />
                      </button>

                      {member.role !== "owner" && (
                        <button className="p-2.5 text-slate-300 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all">
                          <Trash2 className="w-5 h-5" />
                        </button>
                      )}
                    </div>
                  </div>
                ))
              )}

              {members.length === 0 && !isLoading && (
                <div className="py-20 text-center text-slate-400">
                  <p className="font-medium italic">Không có thành viên nào.</p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      <RoleManagementModal
        userId={selectedUserId}
        isOpen={isRoleModalOpen}
        onClose={() => setIsRoleModalOpen(false)}
      />
    </div>
  );
}