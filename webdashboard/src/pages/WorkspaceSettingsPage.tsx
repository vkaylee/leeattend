import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Users, UserPlus, Shield, Mail, Trash2, Settings2, Calculator, Save, Clock, Palette } from "lucide-react";
import { useWorkspaceMembers } from "@/hooks/use-workspace";
import RoleManagementModal from "@/components/RoleManagementModal";
import DebugWorkspaceTools from "@/components/DebugWorkspaceTools";
import { useToast } from "../components/Toast";
import { fetchWorkspace, updateWorkspace, getCurrentWorkspaceId, Workspace } from "../lib/api";
import { Combobox } from "@/components/ui/Combobox";
import { Globe, Calendar, CheckCircle2, ShieldAlert } from "lucide-react";
import { useTheme } from "@/hooks/use-theme";

export default function WorkspaceSettingsPage() {
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const { members, isLoading, addMember, isAdding } = useWorkspaceMembers();
  const toast = useToast();
  const { setPreviewTheme } = useTheme();
  const [newMemberEmail, setNewMemberEmail] = useState("");
  const [newMemberRole, setNewMemberRole] = useState<"admin" | "member">(
    "member",
  );

  const { data: workspace, isLoading: isLoadingWs } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  const [workUnit, setWorkUnit] = useState(1);
  const [roundingPolicy, setRoundingPolicy] = useState("none");
  const [dateFormat, setDateFormat] = useState("DD/MM/YYYY");
  const [timeFormat, setTimeFormat] = useState("24h");
  const [timezone, setTimezone] = useState("Asia/Ho_Chi_Minh");
  const [themePrimaryColor, setThemePrimaryColor] = useState("#7c3aed");

  useEffect(() => {
    if (workspace) {
      setWorkUnit(workspace.work_unit_minutes);
      setRoundingPolicy(workspace.rounding_policy);
      setDateFormat(workspace.date_format);
      setTimeFormat(workspace.time_format);
      setTimezone(workspace.timezone || "Asia/Ho_Chi_Minh");
      setThemePrimaryColor(workspace.theme_primary_color || "#7c3aed");
    }
  }, [workspace]);

  // Handle live preview
  useEffect(() => {
    setPreviewTheme(themePrimaryColor);
    return () => setPreviewTheme(null);
  }, [themePrimaryColor, setPreviewTheme]);

  const updateWsMutation = useMutation({
    mutationFn: (data: Partial<Workspace>) => updateWorkspace(wsId!, data),
    onSuccess: () => {
      toast.success("Cập nhật cấu hình thành công.");
      queryClient.invalidateQueries({ queryKey: ["workspace", wsId] });
    },
    onError: () => {
      toast.error("Không thể cập nhật cấu hình.");
    }
  });

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

  function handleSaveConfig() {
    updateWsMutation.mutate({
      work_unit_minutes: workUnit,
      rounding_policy: roundingPolicy,
      date_format: dateFormat,
      time_format: timeFormat,
      timezone: timezone,
      theme_primary_color: themePrimaryColor,
    });
  }

  return (
    <div className="p-8 animate-in fade-in duration-500">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
            Cài đặt Workspace
          </h1>
          <p className="text-slate-500 mt-1">
            Quản lý thành viên và cấu hình tính toán
          </p>
        </div>

        <div className="grid grid-cols-1 gap-8">
          {/* Payroll Config Section */}
          <div className="bg-white p-8 rounded-3xl border border-slate-100 shadow-sm">
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-xl font-bold flex items-center gap-2 text-slate-800">
                <Calculator className="w-5 h-5 text-primary-500" />
                Cấu hình tính công
              </h2>
              <button
                onClick={handleSaveConfig}
                disabled={updateWsMutation.isPending || isLoadingWs}
                className="flex items-center gap-2 bg-primary-600 text-white px-4 py-2 rounded-xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 disabled:opacity-50 text-sm"
              >
                <Save className="w-4 h-4" />
                Lưu cấu hình
              </button>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  Đơn vị tính công (Phút)
                </label>
                <Combobox
                  data-testid="work-unit-select"
                  options={[
                    { value: "1", label: "1 phút", description: "Mặc định, độ chính xác cao nhất" },
                    { value: "15", label: "15 phút", description: "Khối 0.25 giờ" },
                    { value: "30", label: "30 phút", description: "Khối 0.5 giờ" },
                    { value: "60", label: "60 phút", description: "Khối 1 giờ" },
                  ]}
                  value={workUnit.toString()}
                  onChange={(val) => setWorkUnit(parseInt(val))}
                  className="bg-slate-50 border-slate-200"
                />
                <p className="text-[10px] text-slate-400 italic mt-1 ml-1">
                  Hệ thống sẽ gom các phút làm việc thành từng khối (block) này.
                </p>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  Quy tắc làm tròn
                </label>
                <Combobox
                  data-testid="rounding-policy-select"
                  options={[
                    { value: "none", label: "Không làm tròn", icon: <CheckCircle2 className="w-4 h-4" /> },
                    { value: "floor", label: "Làm tròn xuống (Floor)", icon: <ShieldAlert className="w-4 h-4" /> },
                    { value: "ceil", label: "Làm tròn lên (Ceil)", icon: <ShieldAlert className="w-4 h-4" /> },
                    { value: "round", label: "Làm tròn gần nhất (Round)", icon: <ShieldAlert className="w-4 h-4" /> },
                  ]}
                  value={roundingPolicy}
                  onChange={setRoundingPolicy}
                  className="bg-slate-50 border-slate-200"
                />
                <p className="text-[10px] text-slate-400 italic mt-1 ml-1">
                  Cách xử lý khi tổng giờ làm trong ngày không chia hết cho đơn vị tính.
                </p>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  Định dạng ngày tháng ưu tiên
                </label>
                <Combobox
                  data-testid="date-format-select"
                  options={[
                    { value: "DD/MM/YYYY", label: "DD/MM/YYYY", description: "Kiểu Việt Nam (Ví dụ: 31/12/2025)", icon: <Calendar className="w-4 h-4" /> },
                    { value: "MM/DD/YYYY", label: "MM/DD/YYYY", description: "Kiểu Mỹ (Ví dụ: 12/31/2025)", icon: <Calendar className="w-4 h-4" /> },
                    { value: "YYYY-MM-DD", label: "YYYY-MM-DD", description: "Kiểu ISO (Ví dụ: 2025-12-31)", icon: <Calendar className="w-4 h-4" /> },
                  ]}
                  value={dateFormat}
                  onChange={setDateFormat}
                  className="bg-slate-50 border-slate-200"
                />
                <p className="text-[10px] text-slate-400 italic mt-1 ml-1">
                  Dùng để ưu tiên xử lý khi file Excel/CSV không rõ ràng (ví dụ 01/02).
                </p>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  Định dạng giờ ưu tiên
                </label>
                <Combobox
                  data-testid="time-format-select"
                  options={[
                    { value: "24h", label: "24 Giờ", description: "Ví dụ: 14:30", icon: <Clock className="w-4 h-4" /> },
                    { value: "12h", label: "12 Giờ", description: "Ví dụ: 02:30 PM", icon: <Clock className="w-4 h-4" /> },
                  ]}
                  value={timeFormat}
                  onChange={setTimeFormat}
                  className="bg-slate-50 border-slate-200"
                />
                <p className="text-[10px] text-slate-400 italic mt-1 ml-1">
                  Cách hiển thị thời gian trên toàn hệ thống.
                </p>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  Múi giờ hệ thống
                </label>
                <Combobox
                  data-testid="timezone-select"
                  options={[
                    { value: "Asia/Ho_Chi_Minh", label: "Asia/Ho_Chi_Minh", description: "Việt Nam (UTC+7)", icon: <Globe className="w-4 h-4" /> },
                    { value: "Asia/Bangkok", label: "Asia/Bangkok", description: "Thailand (UTC+7)", icon: <Globe className="w-4 h-4" /> },
                    { value: "UTC", label: "Giờ quốc tế (UTC)", icon: <Globe className="w-4 h-4" /> },
                  ]}
                  value={timezone}
                  onChange={setTimezone}
                  className="bg-slate-50 border-slate-200"
                />
                <p className="text-[10px] text-slate-400 italic mt-1 ml-1">
                  Quyết định múi giờ khi import dữ liệu không có offset và hiển thị báo cáo.
                </p>
              </div>
            </div>
          </div>

          {/* Theme Config Section */}
          <div className="bg-white p-8 rounded-3xl border border-slate-100 shadow-sm">
            <h2 className="text-xl font-bold mb-6 flex items-center gap-2 text-slate-800">
              <Palette className="w-5 h-5 text-pink-500" />
              Cấu hình giao diện
            </h2>
            <div className="space-y-6">
              <div className="space-y-3">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  Màu chủ đạo (Primary Color)
                </label>
                <div className="flex flex-wrap gap-4 items-center">
                  {[
                    { name: 'Violet', color: '#7c3aed' },
                    { name: 'Indigo', color: '#4f46e5' },
                    { name: 'Blue', color: '#2563eb' },
                    { name: 'Emerald', color: '#059669' },
                    { name: 'Amber', color: '#d97706' },
                    { name: 'Rose', color: '#e11d48' },
                    { name: 'Slate', color: '#475569' },
                  ].map((preset) => (
                    <button
                      key={preset.color}
                      onClick={() => setThemePrimaryColor(preset.color)}
                      className={`w-10 h-10 rounded-xl transition-all border-2 flex items-center justify-center ${
                        themePrimaryColor === preset.color ? 'border-slate-900 scale-110 shadow-lg' : 'border-transparent hover:scale-105'
                      }`}
                      style={{ backgroundColor: preset.color }}
                      title={preset.name}
                    >
                      {themePrimaryColor === preset.color && <div className="w-2 h-2 bg-white rounded-full shadow-sm" />}
                    </button>
                  ))}
                  
                  <div className="h-8 w-[1px] bg-slate-100 mx-2" />
                  
                  <div className="flex items-center gap-3 bg-slate-50 p-1.5 rounded-xl border border-slate-100">
                    <input 
                      type="color" 
                      value={themePrimaryColor} 
                      onChange={(e) => setThemePrimaryColor(e.target.value)}
                      className="w-8 h-8 rounded-lg cursor-pointer bg-transparent border-none"
                    />
                    <input 
                      type="text" 
                      value={themePrimaryColor} 
                      onChange={(e) => setThemePrimaryColor(e.target.value)}
                      className="bg-transparent border-none text-xs font-mono font-bold text-slate-600 w-20 outline-none uppercase"
                    />
                  </div>
                </div>
                <p className="text-[10px] text-slate-400 italic mt-2 ml-1">
                  Thay đổi màu sắc này sẽ áp dụng lên toàn bộ giao diện (Nút, Icons, Sidebar,...) dành cho tất cả thành viên.
                </p>
              </div>
            </div>
          </div>

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
                    className="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-primary-500 outline-none transition-all pr-10"
                    value={newMemberEmail}
                    onChange={(e) => setNewMemberEmail(e.target.value)}
                  />
                  <Mail className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
                </div>
              </div>
              <div className="w-56 space-y-2">
                <label
                  htmlFor="new-member-role"
                  className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1"
                >
                  Vai trò
                </label>
                <Combobox
                  id="new-member-role"
                  options={[
                    { value: "member", label: "Thành viên", description: "Chỉ xem dữ liệu cá nhân" },
                    { value: "admin", label: "Quản trị viên", description: "Toàn quyền quản lý hệ thống" },
                  ]}
                  value={newMemberRole}
                  onChange={(val) => setNewMemberRole(val as any)}
                  className="bg-slate-50 border-slate-200"
                />
              </div>
              <div className="flex items-end">
                <button
                  type="submit"
                  disabled={isAdding}
                  className="bg-primary-600 text-white px-8 py-3 rounded-xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 h-[50px] disabled:opacity-50"
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
                        member.role === 'owner' ? 'bg-primary-600 text-white shadow-primary-100' : 
                        member.role === 'admin' ? 'bg-primary-600 text-white shadow-primary-100' : 'bg-slate-100 text-slate-500'
                      }`}>
                        {member.full_name ? member.full_name[0] : <Users className="w-5 h-5" />}
                      </div>
                      <div>
                        <p className="font-bold text-slate-900 flex items-center gap-2">
                          {member.full_name || member.email}
                          {member.role === "owner" && <span className="text-[10px] bg-primary-50 text-primary-600 px-1.5 py-0.5 rounded font-black uppercase tracking-tighter border border-primary-100">Sáng lập</span>}
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
                        className="p-2.5 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-xl transition-all"
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

        {import.meta.env.DEV && (
          <div className="mt-8">
            <DebugWorkspaceTools />
          </div>
        )}
      </div>

      <RoleManagementModal
        userId={selectedUserId}
        isOpen={isRoleModalOpen}
        onClose={() => setIsRoleModalOpen(false)}
      />
    </div>
  );
}