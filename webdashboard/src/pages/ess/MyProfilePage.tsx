import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useToast } from "@/components/Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMyProfile, useEmergencyContacts } from "@/hooks/use-ess";
import {
  User as UserIcon,
  Phone,
  MapPin,
  Calendar,
  CreditCard,
  Building2,
  Users,
  Plus,
  Trash2,
  Save,
  AlertCircle,
} from "lucide-react";
import Modal from "@/components/ui/Modal";

type TabType = "personal" | "banking" | "emergency";

export default function MyProfilePage() {
  const { t } = useTranslation();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();
  const { employee, profile, isLoading, isError, updateProfile, isUpdating } = useMyProfile();
  const { contacts, addContact, deleteContact, isAdding, isDeleting } = useEmergencyContacts();

  const [activeTab, setActiveTab] = useState<TabType>("personal");
  const [formData, setFormData] = useState({
    phone: "", address: "", date_of_birth: "", gender: "", national_id: "",
    bank_name: "", bank_account_number: "", bank_branch: "",
  });

  const [isContactModalOpen, setIsContactModalOpen] = useState(false);
  const [newContact, setNewContact] = useState({ contact_name: "", relationship: "", phone: "", is_primary: false });

  useEffect(() => {
    if (profile) {
      setFormData({
        phone: profile.phone || "", address: profile.address || "", date_of_birth: profile.date_of_birth || "",
        gender: profile.gender || "", national_id: profile.national_id || "", bank_name: profile.bank_name || "",
        bank_account_number: profile.bank_account_number || "", bank_branch: profile.bank_branch || "",
      });
    }
  }, [profile]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("ess.profile.update_success"));
    } catch (err) { toast.error(t("ess.profile.update_error")); }
  }

  async function handleAddContact(e: React.FormEvent) {
    e.preventDefault();
    try {
      await addContact(newContact);
      setNewContact({ contact_name: "", relationship: "", phone: "", is_primary: false });
      setIsContactModalOpen(false);
      toast.success(t("ess.emergency_contact.add_success"));
    } catch (err) { toast.error(t("ess.emergency_contact.add_error")); }
  }

  async function handleDeleteContact(contactId: string) {
    if (!confirm(t("ess.emergency_contact.confirm_delete"))) return;
    try {
      await deleteContact(contactId);
      toast.success(t("ess.emergency_contact.delete_success"));
    } catch (err) { toast.error(t("ess.emergency_contact.delete_error")); }
  }

  if (isLoading) return <div className="p-8 text-center text-slate-500 font-bold italic">{t("common.loading")}</div>;

  if (isError || !employee) {
    return (
      <div className="p-8"><div className="bg-rose-50 border border-rose-100 rounded-[2rem] p-8 flex items-center gap-6 shadow-xl shadow-rose-100/50">
        <div className="bg-rose-100 p-4 rounded-2xl text-rose-600"><AlertCircle size={32} /></div>
        <div><h3 className="text-xl font-black text-rose-900 uppercase tracking-tight">{t("ess.no_employee_linked")}</h3><p className="font-bold text-rose-600 opacity-80 mt-1">{t("ess.contact_hr")}</p></div>
      </div></div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 px-2">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">{t("ess.my_profile")}</h1>
          <p className="text-slate-500 mt-1 font-medium">{t("ess.profile.subtitle")}</p>
        </div>

        {/* Tab Switcher */}
        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
          <button onClick={() => setActiveTab("personal")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'personal' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <UserIcon size={14} /> {t("ess.personal_info")}
          </button>
          <button onClick={() => setActiveTab("banking")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'banking' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <CreditCard size={14} /> {t("ess.bank_details")}
          </button>
          <button onClick={() => setActiveTab("emergency")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'emergency' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700"}`}>
            <Users size={14} /> {t("ess.emergency_contacts")}
          </button>
        </div>
      </div>

      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {/* Header Banner Section */}
        <div className="max-w-5xl bg-white rounded-[32px] border border-slate-100 shadow-sm overflow-hidden mb-8">
          <div className="h-32 bg-gradient-to-r from-primary-600 to-indigo-600 opacity-90"></div>
          <div className="px-10 pb-8">
            <div className="relative -mt-12 mb-6 flex items-end gap-6">
              <div className="w-28 h-28 rounded-[2rem] border-8 border-white shadow-xl bg-primary-50 flex items-center justify-center text-primary-600 font-black text-2xl uppercase shadow-inner">
                {employee.first_name[0]}{employee.last_name[0]}
              </div>
              <div className="mb-2">
                <h2 className="text-2xl font-black text-slate-900 tracking-tight leading-tight">{employee.first_name} {employee.last_name}</h2>
                <div className="flex flex-wrap gap-x-4 gap-y-1 mt-1.5">
                  <p className="text-[10px] font-black text-primary-600 uppercase tracking-widest">{t("common.employee_id")}: {employee.employee_id_number}</p>
                  <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("common.hire_date")}: {formatDate(employee.hire_date)}</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        {activeTab === "personal" && (
          <form onSubmit={handleSubmit} className="max-w-5xl bg-white rounded-[32px] border border-slate-100 shadow-sm p-10 space-y-10">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              {[
                { label: "phone", key: "phone", icon: Phone, type: "tel", placeholder: "+84 xxx xxx xxx" },
                { label: "date_of_birth", key: "date_of_birth", icon: Calendar, type: "date" },
                { label: "national_id", key: "national_id", icon: ShieldCheck, placeholder: "xxx xxx xxx xxx" }
              ].map(field => (
                <div key={field.key} className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"><field.icon size={12} className="inline mr-1" /> {t(`ess.${field.label}`)}</label>
                  <input type={field.type} value={(formData as any)[field.key]} onChange={(e) => setFormData({ ...formData, [field.key]: e.target.value })} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm" placeholder={field.placeholder} />
                </div>
              ))}
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.gender")}</label>
                <select value={formData.gender} onChange={(e) => setFormData({ ...formData, gender: e.target.value })} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm">
                  <option value="">{t("common.select")}</option>
                  <option value="male">{t("ess.gender_male")}</option>
                  <option value="female">{t("ess.gender_female")}</option>
                  <option value="other">{t("ess.gender_other")}</option>
                </select>
              </div>
              <div className="space-y-2 md:col-span-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"><MapPin size={12} className="inline mr-1" /> {t("ess.address")}</label>
                <textarea value={formData.address} onChange={(e) => setFormData({ ...formData, address: e.target.value })} rows={2} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-medium text-sm resize-none" placeholder={t("ess.address_placeholder")} />
              </div>
            </div>
            <div className="pt-6 border-t border-slate-50 flex justify-end">
              <button type="submit" disabled={isUpdating} className="bg-primary-600 text-white px-10 py-3.5 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg active:scale-95 disabled:opacity-50 flex items-center gap-3"><Save size={16} /> {isUpdating ? t("common.saving").toUpperCase() : t("common.save")}</button>
            </div>
          </form>
        )}

        {activeTab === "banking" && (
          <form onSubmit={handleSubmit} className="max-w-5xl bg-white rounded-[32px] border border-slate-100 shadow-sm p-10 space-y-10">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
              {[
                { label: "bank_name", key: "bank_name", icon: Building2, placeholder: "Vietcombank" },
                { label: "account_number", key: "bank_account_number", icon: CreditCard, placeholder: "xxxx xxxx xxxx" },
                { label: "bank_branch", key: "bank_branch", icon: MapPin, placeholder: t("ess.branch_placeholder") }
              ].map(field => (
                <div key={field.key} className="space-y-2">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"><field.icon size={12} className="inline mr-1" /> {t(`ess.${field.label}`)}</label>
                  <input type="text" value={(formData as any)[field.key]} onChange={(e) => setFormData({ ...formData, [field.key]: e.target.value })} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase" placeholder={field.placeholder} />
                </div>
              ))}
            </div>
            <div className="pt-6 border-t border-slate-50 flex justify-end">
              <button type="submit" disabled={isUpdating} className="bg-primary-600 text-white px-10 py-3.5 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg active:scale-95 disabled:opacity-50 flex items-center gap-3"><Save size={16} /> {isUpdating ? t("common.saving").toUpperCase() : t("common.save")}</button>
            </div>
          </form>
        )}

        {activeTab === "emergency" && (
          <div className="max-w-5xl bg-white rounded-[32px] border border-slate-100 shadow-sm p-10 space-y-8">
            <div className="flex items-center justify-between px-2">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-2xl bg-rose-50 flex items-center justify-center text-rose-600 shadow-inner border border-rose-100/50"><Users size={24} /></div>
                <div>
                  <h3 className="text-xl font-black text-slate-900 tracking-tight uppercase">{t("ess.emergency_contacts")}</h3>
                  <p className="text-xs font-medium text-slate-500">{t("ess.emergency_contact.subtitle", { defaultValue: "Manage your primary contacts for emergencies." })}</p>
                </div>
              </div>
              <button type="button" onClick={() => setIsContactModalOpen(true)} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-black text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95"><Plus size={14} /> {t("ess.add_contact")}</button>
            </div>

            {contacts.length === 0 ? (
              <div className="py-20 text-center border-2 border-dashed border-slate-100 rounded-[2.5rem] bg-slate-50/30 flex flex-col items-center">
                <Users size={32} className="text-slate-200 mb-4" />
                <p className="text-xs font-black text-slate-300 uppercase tracking-widest">{t("ess.no_emergency_contacts")}</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {contacts.map((contact) => (
                  <div key={contact.id} className="flex items-center justify-between p-6 bg-slate-50 border border-slate-100 rounded-[2rem] hover:bg-white hover:shadow-xl hover:shadow-primary-100/10 transition-all group">
                    <div className="flex items-center gap-4">
                      <div className="w-10 h-10 rounded-xl bg-white border border-slate-100 flex items-center justify-center text-slate-400 font-black shadow-sm group-hover:text-primary-600 transition-colors">{contact.contact_name[0]}</div>
                      <div>
                        <div className="font-black text-slate-900 text-sm tracking-tight flex items-center gap-2">
                          {contact.contact_name}
                          {contact.is_primary && <span className="text-[8px] bg-primary-600 text-white px-1.5 py-0.5 rounded font-black uppercase tracking-tighter">{t("ess.primary")}</span>}
                        </div>
                        <div className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mt-1">{contact.relationship} • {contact.phone}</div>
                      </div>
                    </div>
                    <button type="button" onClick={() => handleDeleteContact(contact.id)} disabled={isDeleting} className="p-2.5 text-slate-300 hover:text-rose-600 hover:bg-rose-50 rounded-xl transition-all opacity-0 group-hover:opacity-100"><Trash2 size={16} /></button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      <Modal isOpen={isContactModalOpen} onClose={() => setIsContactModalOpen(false)} title={t("ess.add_emergency_contact")} icon={<Users size={24} />} maxWidth="md">
        <form onSubmit={handleAddContact} className="space-y-6 pt-4">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.contact_name")}</label>
            <input type="text" required value={newContact.contact_name} onChange={(e) => setNewContact({ ...newContact, contact_name: e.target.value })} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm" />
          </div>
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.relationship")}</label>
              <select required value={newContact.relationship} onChange={(e) => setNewContact({ ...newContact, relationship: e.target.value })} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-bold text-sm">
                <option value="">{t("common.select")}</option>
                <option value="spouse">{t("ess.relationship_spouse")}</option>
                <option value="parent">{t("ess.relationship_parent")}</option>
                <option value="sibling">{t("ess.relationship_sibling")}</option>
                <option value="child">{t("ess.relationship_child")}</option>
                <option value="friend">{t("ess.relationship_friend")}</option>
                <option value="other">{t("ess.relationship_other")}</option>
              </select>
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.phone")}</label>
              <input type="tel" required value={newContact.phone} onChange={(e) => setNewContact({ ...newContact, phone: e.target.value })} className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm" />
            </div>
          </div>
          <div className="flex items-center gap-3 py-2">
            <input type="checkbox" id="is_primary" checked={newContact.is_primary} onChange={(e) => setNewContact({ ...newContact, is_primary: e.target.checked })} className="w-6 h-6 text-primary-600 rounded-xl border-slate-200 focus:ring-primary-500 transition-all cursor-pointer" />
            <label htmlFor="is_primary" className="text-sm font-black text-slate-700 uppercase tracking-widest cursor-pointer">{t("ess.set_as_primary")}</label>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-5">
            <button type="button" onClick={() => setIsContactModalOpen(false)} className="px-8 py-4 border border-slate-100 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button type="submit" disabled={isAdding} className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200">{isAdding ? t("common.adding").toUpperCase() : t("common.add")}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
