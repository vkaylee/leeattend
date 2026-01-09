import { useState, useEffect } from "react";
import {
  User as UserIcon,
  Mail,
  UserCircle,
  Save,
  Calendar,
  Lock,
  ShieldCheck,
} from "lucide-react";
import { useProfile } from "@/hooks/use-profile";
import { changePassword, clearAuthSession } from "@/lib/api";
import { useNavigate } from "react-router-dom";
import { useToast } from "../components/Toast";

export default function ProfilePage() {
  const navigate = useNavigate();
  const toast = useToast();
  const { profile, isLoading, updateProfile, isUpdating } = useProfile();
  const [formData, setFormData] = useState({
    full_name: "",
    avatar_url: "",
    bio: "",
  });

  useEffect(() => {
    if (profile) {
      setFormData({
        full_name: profile.full_name || "",
        avatar_url: profile.avatar_url || "",
        bio: profile.bio || "",
      });
    }
  }, [profile]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success("Thông tin hồ sơ cá nhân đã được lưu.");
    } catch (e) {
      toast.error("Không thể cập nhật thông tin. Vui lòng thử lại sau.");
    }
  }

  if (isLoading) {
    return (
      <div className="p-8 text-center text-slate-500 font-bold">Đang tải hồ sơ...</div>
    );
  }

  return (
    <div className="p-8 animate-in fade-in duration-500">
      <div className="max-w-2xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
            Hồ sơ cá nhân
          </h1>
          <p className="text-slate-500 mt-1">
            Quản lý thông tin định danh toàn cầu của bạn
          </p>
        </div>

        <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
          <div className="h-32 bg-gradient-to-r from-blue-600 to-indigo-600"></div>
          <div className="px-8 pb-8">
            <div className="relative -mt-12 mb-6">
              {formData.avatar_url ? (
                <img
                  src={formData.avatar_url}
                  alt="Avatar"
                  className="w-24 h-24 rounded-2xl border-4 border-white shadow-lg object-cover"
                />
              ) : (
                <div className="w-24 h-24 rounded-2xl border-4 border-white shadow-lg bg-slate-100 flex items-center justify-center text-slate-400">
                  <UserCircle className="w-12 h-12" />
                </div>
              )}
            </div>

            <form onSubmit={handleSubmit} className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    Email (Cố định)
                  </label>
                  <div className="flex items-center gap-3 px-4 py-3 bg-slate-50 border border-slate-100 rounded-xl text-slate-500">
                    <Mail className="w-4 h-4" />
                    <span className="text-sm font-medium">
                      {profile?.email}
                    </span>
                  </div>
                </div>
                <div className="space-y-2">
                  <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                    Ngày tham gia
                  </label>
                  <div className="flex items-center gap-3 px-4 py-3 bg-slate-50 border border-slate-100 rounded-xl text-slate-500">
                    <Calendar className="w-4 h-4" />
                    <span className="text-sm font-medium">
                      {profile ? new Date(profile.created_at).toLocaleDateString("vi-VN") : '---'}
                    </span>
                  </div>
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  Họ và tên
                </label>
                <div className="relative">
                  <input
                    placeholder="Ví dụ: Nguyễn Văn A"
                    className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all pr-10"
                    value={formData.full_name}
                    onChange={(e) =>
                      setFormData({ ...formData, full_name: e.target.value })
                    }
                  />
                  <UserIcon className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  URL Ảnh đại diện
                </label>
                <input
                  placeholder="https://example.com/avatar.jpg"
                  className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                  value={formData.avatar_url}
                  onChange={(e) =>
                    setFormData({ ...formData, avatar_url: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  Tiểu sử
                </label>
                <div className="relative">
                  <textarea
                    rows={3}
                    placeholder="Một chút về bản thân bạn..."
                    className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                    value={formData.bio}
                    onChange={(e) =>
                      setFormData({ ...formData, bio: e.target.value })
                    }
                  />
                </div>
              </div>

              <div className="pt-4 border-t border-slate-50 flex justify-end">
                <button
                  type="submit"
                  disabled={isUpdating}
                  className="bg-blue-600 text-white px-8 py-3 rounded-xl font-bold hover:bg-blue-700 transition-all shadow-lg shadow-blue-100 flex items-center gap-2 disabled:opacity-50"
                >
                  <Save className="w-5 h-5" />
                  {isUpdating ? "Đang lưu..." : "Lưu hồ sơ"}
                </button>
              </div>
            </form>
          </div>
        </div>

        {/* Security Section */}
        <div className="mt-8 bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden">
          <div className="p-8">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-10 h-10 rounded-xl bg-orange-50 flex items-center justify-center text-orange-600">
                <ShieldCheck className="w-6 h-6" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-slate-900">Bảo mật</h2>
                <p className="text-sm text-slate-500">
                  Thay đổi mật khẩu đăng nhập của bạn
                </p>
              </div>
            </div>

            <ChangePasswordForm />
          </div>
        </div>
      </div>
    </div>
  );
}

function ChangePasswordForm() {
  const navigate = useNavigate();
  const toast = useToast();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [formData, setFormData] = useState({
    old_password: "",
    new_password: "",
    confirm_password: "",
  });

  async function handlePasswordSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (formData.new_password !== formData.confirm_password) {
      toast.warning("Xác nhận mật khẩu mới không khớp.");
      return;
    }

    setIsSubmitting(true);
    try {
      await changePassword({
        old_password: formData.old_password,
        new_password: formData.new_password,
      });
      toast.success("Mật khẩu đã được thay đổi. Vui lòng đăng nhập lại.");
      setTimeout(() => {
        clearAuthSession();
        navigate("/login");
      }, 2000);
    } catch (e: any) {
      toast.error(e.message || "Lỗi khi đổi mật khẩu.");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <form onSubmit={handlePasswordSubmit} className="space-y-4 max-w-md">
      <div className="space-y-2">
        <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
          Mật khẩu hiện tại
        </label>
        <div className="relative">
          <input
            type="password"
            required
            className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-orange-500 outline-none transition-all pr-10"
            value={formData.old_password}
            onChange={(e) =>
              setFormData({ ...formData, old_password: e.target.value })
            }
          />
          <Lock className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
          Mật khẩu mới
        </label>
        <div className="relative">
          <input
            type="password"
            required
            className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-orange-500 outline-none transition-all pr-10"
            value={formData.new_password}
            onChange={(e) =>
              setFormData({ ...formData, new_password: e.target.value })
            }
          />
          <Lock className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
          Xác nhận mật khẩu mới
        </label>
        <div className="relative">
          <input
            type="password"
            required
            className="w-full bg-white border border-slate-200 rounded-xl px-4 py-3 focus:ring-2 focus:ring-orange-500 outline-none transition-all pr-10"
            value={formData.confirm_password}
            onChange={(e) =>
              setFormData({ ...formData, confirm_password: e.target.value })
            }
          />
          <Lock className="absolute right-3 top-3.5 w-4 h-4 text-slate-300" />
        </div>
      </div>

      <div className="pt-2">
        <button
          type="submit"
          disabled={isSubmitting}
          className="bg-slate-900 text-white px-6 py-3 rounded-xl font-bold hover:bg-slate-800 transition-all flex items-center gap-2 disabled:opacity-50"
        >
          {isSubmitting ? "Đang xử lý..." : "Cập nhật mật khẩu"}
        </button>
      </div>
    </form>
  );
}