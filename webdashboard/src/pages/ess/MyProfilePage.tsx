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
  ShieldCheck,
} from "lucide-react";
import Modal from "@/components/ui/Modal";
import { cn } from "@/lib/utils";

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
      // eslint-disable-next-line react-hooks/set-state-in-effect
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
    } catch (err) { console.error(err); }
  }

  async function handleAddContact(e: React.FormEvent) {
    e.preventDefault();
    try {
      await addContact(newContact);
      setNewContact({ contact_name: "", relationship: "", phone: "", is_primary: false });
      setIsContactModalOpen(false);
      toast.success(t("ess.emergency_contact.add_success"));
    } catch (err) { console.error(err); }
  }

  async function handleDeleteContact(contactId: string) {
    if (!confirm(t("ess.emergency_contact.confirm_delete"))) return;
    try {
      await deleteContact(contactId);
      toast.success(t("ess.emergency_contact.delete_success"));
    } catch (err) { console.error(err); }
  }

  if (isLoading) return <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>;

  if (isError || !employee) {
    return (
      <div className="p-8"><div className="bg-rose-50 border-2 border-rose-100 rounded-[3rem] p-12 flex items-center gap-8 shadow-2xl shadow-rose-100/50">
        <div className="bg-white p-5 rounded-[1.5rem] shadow-sm border border-rose-100 text-rose-500"><AlertCircle size={40} /></div>
        <div><h3 className="text-2xl font-black text-rose-900 uppercase tracking-tighter leading-none mb-2">{t("ess.no_employee_linked")}</h3><p className="font-bold text-rose-600 uppercase tracking-tight text-sm opacity-80">{t("ess.contact_hr")}</p></div>
      </div></div>
    );
  }

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="space-y-1">
          <h1 className="text-4xl font-black tracking-tighter text-slate-900 uppercase leading-none">{t("ess.my_profile")}</h1>
          <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-60">{t("ess.profile.subtitle")}</p>
        </div>

        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
          <button onClick={() => setActiveTab("personal")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'personal' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><UserIcon size={14} /> {t("ess.personal_info")}</button>
          <button onClick={() => setActiveTab("banking")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'banking' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><CreditCard size={14} /> {t("ess.bank_details")}</button>
          <button onClick={() => setActiveTab("emergency")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'emergency' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Users size={14} /> {t("ess.emergency_contacts")}</button>
        </div>
      </div>

      <div className="animate-in slide-in-from-bottom-8 duration-700">
        <div className="max-w-5xl bg-white rounded-[3rem] border border-slate-100 shadow-sm overflow-hidden mb-10 group/hero">
          <div className="h-40 bg-slate-900 relative overflow-hidden">
             <div className="absolute inset-0 bg-gradient-to-br from-primary-600/40 via-indigo-600/20 to-transparent" />
             <div className="absolute top-0 left-0 w-full h-full opacity-10 pointer-events-none bg-[radial-gradient(circle_at_50%_50%,rgba(255,255,255,0.1)_0,transparent_50%)]" />
          </div>
          <div className="px-12 pb-10">
            <div className="relative -mt-16 mb-8 flex items-end gap-8">
              <div className="w-32 h-32 rounded-[2.5rem] border-[10px] border-white shadow-2xl bg-primary-50 flex items-center justify-center text-primary-600 font-black text-3xl uppercase shadow-inner rotate-3 group-hover/hero:rotate-0 transition-transform duration-700">
                {employee.full_name[0]}
              </div>
              <div className="mb-2">
                <h2 className="text-3xl font-black text-slate-900 tracking-tighter uppercase leading-none mb-2">{employee.full_name}</h2>
                <div className="flex flex-wrap gap-4 items-center">
                  <span className="text-[10px] font-black text-primary-600 bg-primary-50 px-3 py-1 rounded-full border border-primary-100 shadow-sm uppercase tracking-widest">{t("common.employee_id")}: {employee.employee_id_number}</span>
                  <div className="w-1.5 h-1.5 bg-slate-200 rounded-full" />
                  <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest opacity-70">{t("common.hire_date")}: {formatDate(employee.hire_date)}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {activeTab === "personal" && (
          <form onSubmit={handleSubmit} className="max-w-5xl bg-white rounded-[3rem] border border-slate-100 shadow-sm p-12 space-y-12 animate-in fade-in duration-500">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
              {[
                { label: "phone", key: "phone", icon: Phone, type: "tel", placeholder: "+84 xxx xxx xxx" },
                { label: "date_of_birth", key: "date_of_birth", icon: Calendar, type: "date" },
                { label: "national_id", key: "national_id", icon: ShieldCheck, placeholder: "xxx xxx xxx xxx" }
              ].map(field => (
                <div key={field.key} className="space-y-3 px-1">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"><field.icon size={12} className="inline mr-2" /> {t(`ess.${field.label}`)}</label>
                  <input type={field.type} value={(formData as any)[field.key]} onChange={(e) => setFormData({ ...formData, [field.key]: e.target.value })} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" placeholder={field.placeholder} />
                </div>
              ))}
              <div className="space-y-3 px-1">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.gender")}</label>
                <select value={formData.gender} onChange={(e) => setFormData({ ...formData, gender: e.target.value })} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm appearance-none cursor-pointer">
                  <option value="">{t("common.select").toUpperCase()}</option>
                  <option value="male">{t("ess.gender_male").toUpperCase()}</option>
                  <option value="female">{t("ess.gender_female").toUpperCase()}</option>
                  <option value="other">{t("ess.gender_other").toUpperCase()}</option>
                </select>
              </div>
              <div className="space-y-3 md:col-span-2 px-1">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"><MapPin size={12} className="inline mr-2" /> {t("ess.address")}</label>
                <textarea value={formData.address} onChange={(e) => setFormData({ ...formData, address: e.target.value })} rows={2} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-medium text-sm resize-none shadow-sm" placeholder={t("ess.address_placeholder")} />
              </div>
            </div>
            <div className="pt-8 border-t border-slate-50 flex justify-end">
              <button type="submit" disabled={isUpdating} className="bg-primary-600 text-white px-12 py-4 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50 flex items-center gap-4"><Save size={18} /> {isUpdating ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</button>
            </div>
          </form>
        )}

        {activeTab === "banking" && (
          <form onSubmit={handleSubmit} className="max-w-5xl bg-white rounded-[3rem] border border-slate-100 shadow-sm p-12 space-y-12 animate-in fade-in duration-500">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-10">
              {[
                { label: "bank_name", key: "bank_name", icon: Building2, placeholder: "Vietcombank" },
                { label: "account_number", key: "bank_account_number", icon: CreditCard, placeholder: "xxxx xxxx xxxx" },
                { label: "bank_branch", key: "bank_branch", icon: MapPin, placeholder: t("ess.branch_placeholder") }
              ].map(field => (
                <div key={field.key} className="space-y-3 px-1">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1"><field.icon size={12} className="inline mr-2" /> {t(`ess.${field.label}`)}</label>
                  <input type="text" value={(formData as any)[field.key]} onChange={(e) => setFormData({ ...formData, [field.key]: e.target.value })} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm font-mono" placeholder={field.placeholder} />
                </div>
              ))}
            </div>
            <div className="pt-8 border-t border-slate-50 flex justify-end">
              <button type="submit" disabled={isUpdating} className="bg-primary-600 text-white px-12 py-4 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50 flex items-center gap-4"><Save size={18} /> {isUpdating ? t("common.saving").toUpperCase() : t("common.save").toUpperCase()}</button>
            </div>
          </form>
        )}

        {activeTab === "emergency" && (
          <div className="max-w-5xl bg-white rounded-[3rem] border border-slate-100 shadow-sm p-12 space-y-10 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
              <div className="flex items-center gap-5">
                <div className="w-14 h-14 rounded-2xl bg-rose-50 flex items-center justify-center text-rose-600 shadow-inner border border-rose-100/50"><Users size={28} /></div>
                <div>
                  <h3 className="text-2xl font-black text-slate-900 tracking-tighter uppercase leading-none mb-1">{t("ess.emergency_contacts")}</h3>
                  <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-70">{t("ess.emergency_contact.subtitle")}</p>
                </div>
              </div>
              <button type="button" onClick={() => setIsContactModalOpen(true)} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95"><Plus size={18} /> {t("ess.add_contact").toUpperCase()}</button>
            </div>

            {contacts.length === 0 ? (
              <div className="py-32 text-center border-2 border-dashed border-slate-100 rounded-[3rem] bg-slate-50/30 flex flex-col items-center">
                <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100"><Users size={32} className="text-slate-200" /></div>
                <p className="text-[10px] font-black text-slate-300 uppercase tracking-widest italic">{t("ess.no_emergency_contacts")}</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {contacts.map((contact) => (
                  <div key={contact.id} className="flex items-center justify-between p-8 bg-slate-50 border border-slate-100 rounded-[2.5rem] hover:bg-white hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden shadow-inner">
                    <div className="flex items-center gap-6">
                      <div className="w-12 h-12 rounded-2xl bg-white border border-slate-100 flex items-center justify-center text-slate-400 font-black text-lg shadow-sm group-hover:bg-primary-50 group-hover:text-primary-600 group-hover:border-primary-100 transition-all uppercase">{contact.contact_name[0]}</div>
                      <div>
                        <div className="font-black text-slate-900 text-sm tracking-tight flex items-center gap-3 uppercase leading-none mb-2">
                          {contact.contact_name}
                          {contact.is_primary && <span className="text-[9px] bg-primary-600 text-white px-2 py-0.5 rounded-full font-black uppercase tracking-widest shadow-lg shadow-primary-500/20">{t("ess.primary")}</span>}
                        </div>
                        <div className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] opacity-70">{contact.relationship} • <span className="font-mono text-slate-500 font-bold">{contact.phone}</span></div>
                      </div>
                    </div>
                    <button type="button" onClick={() => handleDeleteContact(contact.id)} disabled={isDeleting} className="p-3 bg-white text-slate-300 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all opacity-0 group-hover:opacity-100 active:scale-90"><Trash2 size={18} /></button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      <Modal isOpen={isContactModalOpen} onClose={() => setIsContactModalOpen(false)} title={t("ess.add_emergency_contact")} icon={<Users size={24} />} maxWidth="md">
        <form onSubmit={handleAddContact} className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.contact_name")}</label>
            <input type="text" required value={newContact.contact_name} onChange={(e) => setNewContact({ ...newContact, contact_name: e.target.value })} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" />
          </div>
          <div className="grid grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.relationship")}</label>
              <select required value={newContact.relationship} onChange={(e) => setNewContact({ ...newContact, relationship: e.target.value })} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm appearance-none cursor-pointer">
                <option value="">{t("common.select").toUpperCase()}</option>
                <option value="spouse">{t("ess.relationship_spouse").toUpperCase()}</option>
                <option value="parent">{t("ess.relationship_parent").toUpperCase()}</option>
                <option value="sibling">{t("ess.relationship_sibling").toUpperCase()}</option>
                <option value="child">{t("ess.relationship_child").toUpperCase()}</option>
                <option value="friend">{t("ess.relationship_friend").toUpperCase()}</option>
                <option value="other">{t("ess.relationship_other").toUpperCase()}</option>
              </select>
            </div>
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("ess.phone")}</label>
              <input type="tel" required value={newContact.phone} onChange={(e) => setNewContact({ ...newContact, phone: e.target.value })} className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-mono font-black text-sm tracking-tighter shadow-sm" />
            </div>
          </div>
          <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
            <input type="checkbox" id="is_primary" checked={newContact.is_primary} onChange={(e) => setNewContact({ ...newContact, is_primary: e.target.checked })} className="w-7 h-7 text-primary-600 rounded-xl border-slate-200 focus:ring-primary-500 transition-all cursor-pointer shadow-inner" />
            <label htmlFor="is_primary" className="text-sm font-black text-slate-700 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 transition-colors">{t("ess.set_as_primary")}</label>
          </div>
          <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50">
            <button type="button" onClick={() => setIsContactModalOpen(false)} className="px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button type="submit" disabled={isAdding} className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95 disabled:opacity-50">{isAdding ? t("common.adding").toUpperCase() : t("common.add").toUpperCase()}</button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
