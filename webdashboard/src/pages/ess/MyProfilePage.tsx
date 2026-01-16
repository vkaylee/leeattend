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

export default function MyProfilePage() {
  const { t } = useTranslation();
  const toast = useToast();
  const { formatDate } = useDateTimeFormatter();
  const {
    employee,
    profile,
    isLoading,
    isError,
    updateProfile,
    isUpdating,
  } = useMyProfile();
  const {
    contacts,
    addContact,
    deleteContact,
    isAdding,
    isDeleting,
  } = useEmergencyContacts();

  const [formData, setFormData] = useState({
    phone: "",
    address: "",
    date_of_birth: "",
    gender: "",
    national_id: "",
    bank_name: "",
    bank_account_number: "",
    bank_branch: "",
  });

  const [isContactModalOpen, setIsContactModalOpen] = useState(false);
  const [newContact, setNewContact] = useState({
    contact_name: "",
    relationship: "",
    phone: "",
    is_primary: false,
  });

  useEffect(() => {
    if (profile) {
      const timer = setTimeout(() => {
        setFormData({
          phone: profile.phone || "",
          address: profile.address || "",
          date_of_birth: profile.date_of_birth || "",
          gender: profile.gender || "",
          national_id: profile.national_id || "",
          bank_name: profile.bank_name || "",
          bank_account_number: profile.bank_account_number || "",
          bank_branch: profile.bank_branch || "",
        });
      }, 0);
      return () => clearTimeout(timer);
    }
  }, [profile]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("ess.profile.update_success"));
    } catch (err) {
      toast.error(t("ess.profile.update_error"));
    }
  }

  async function handleAddContact(e: React.FormEvent) {
    e.preventDefault();
    try {
      await addContact(newContact);
      setNewContact({ contact_name: "", relationship: "", phone: "", is_primary: false });
      setIsContactModalOpen(false);
      toast.success(t("ess.emergency_contact.add_success"));
    } catch (err) {
      toast.error(t("ess.emergency_contact.add_error"));
    }
  }

  async function handleDeleteContact(contactId: string) {
    if (!confirm(t("ess.emergency_contact.confirm_delete"))) return;
    try {
      await deleteContact(contactId);
      toast.success(t("ess.emergency_contact.delete_success"));
    } catch (err) {
      toast.error(t("ess.emergency_contact.delete_error"));
    }
  }

  if (isLoading) {
    return (
      <div className="p-8 text-center text-slate-500 font-bold">
        {t("common.loading")}
      </div>
    );
  }

  if (isError || !employee) {
    return (
      <div className="p-8">
        <div className="bg-red-50 border border-red-200 rounded-2xl p-6 flex items-center gap-4">
          <AlertCircle className="w-6 h-6 text-red-500" />
          <div>
            <h3 className="font-bold text-red-800">{t("ess.no_employee_linked")}</h3>
            <p className="text-sm text-red-600">{t("ess.contact_hr")}</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="p-8 animate-in fade-in duration-500">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-slate-900 tracking-tight">
            {t("ess.my_profile")}
          </h1>
          <p className="text-slate-500 mt-1">{t("ess.profile.subtitle")}</p>
        </div>

        {/* Employee Info Card */}
        <div className="bg-white rounded-3xl border border-slate-100 shadow-sm overflow-hidden mb-6">
          <div className="h-24 bg-gradient-to-r from-primary-600 to-primary-500"></div>
          <div className="px-8 pb-6">
            <div className="relative -mt-10 mb-4">
              <div className="w-20 h-20 rounded-2xl border-4 border-white shadow-lg bg-primary-100 flex items-center justify-center text-primary-600 font-bold text-xl">
                {employee.first_name[0]}
                {employee.last_name[0]}
              </div>
            </div>
            <h2 className="text-xl font-bold text-slate-900">
              {employee.first_name} {employee.last_name}
            </h2>
            <p className="text-slate-500 text-sm">
              {t("common.employee_id")}: {employee.employee_id_number}
            </p>
            <p className="text-slate-400 text-sm">
              {t("common.hire_date")}: {formatDate(employee.hire_date)}
            </p>
          </div>
        </div>

        {/* Profile Form */}
        <form onSubmit={handleSubmit}>
          <div className="bg-white rounded-3xl border border-slate-100 shadow-sm p-8 mb-6">
            <h3 className="text-lg font-bold text-slate-900 mb-6 flex items-center gap-2">
              <UserIcon className="w-5 h-5 text-primary-500" />
              {t("ess.personal_info")}
            </h3>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  <Phone className="w-3 h-3 inline mr-1" />
                  {t("ess.phone")}
                </label>
                <input
                  type="tel"
                  value={formData.phone}
                  onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
                  placeholder="+84 xxx xxx xxx"
                />
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  <Calendar className="w-3 h-3 inline mr-1" />
                  {t("ess.date_of_birth")}
                </label>
                <input
                  type="date"
                  value={formData.date_of_birth}
                  onChange={(e) => setFormData({ ...formData, date_of_birth: e.target.value })}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  {t("ess.gender")}
                </label>
                <select
                  value={formData.gender}
                  onChange={(e) => setFormData({ ...formData, gender: e.target.value })}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
                >
                  <option value="">{t("common.select")}</option>
                  <option value="male">{t("ess.gender_male")}</option>
                  <option value="female">{t("ess.gender_female")}</option>
                  <option value="other">{t("ess.gender_other")}</option>
                </select>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  {t("ess.national_id")}
                </label>
                <input
                  type="text"
                  value={formData.national_id}
                  onChange={(e) => setFormData({ ...formData, national_id: e.target.value })}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
                  placeholder="xxx xxx xxx xxx"
                />
              </div>

              <div className="space-y-2 md:col-span-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  <MapPin className="w-3 h-3 inline mr-1" />
                  {t("ess.address")}
                </label>
                <textarea
                  value={formData.address}
                  onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                  rows={2}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all resize-none"
                  placeholder={t("ess.address_placeholder")}
                />
              </div>
            </div>
          </div>

          {/* Bank Details */}
          <div className="bg-white rounded-3xl border border-slate-100 shadow-sm p-8 mb-6">
            <h3 className="text-lg font-bold text-slate-900 mb-6 flex items-center gap-2">
              <CreditCard className="w-5 h-5 text-primary-500" />
              {t("ess.bank_details")}
            </h3>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  <Building2 className="w-3 h-3 inline mr-1" />
                  {t("ess.bank_name")}
                </label>
                <input
                  type="text"
                  value={formData.bank_name}
                  onChange={(e) => setFormData({ ...formData, bank_name: e.target.value })}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
                  placeholder="Vietcombank"
                />
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  {t("ess.account_number")}
                </label>
                <input
                  type="text"
                  value={formData.bank_account_number}
                  onChange={(e) => setFormData({ ...formData, bank_account_number: e.target.value })}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
                  placeholder="xxxx xxxx xxxx"
                />
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold text-slate-400 uppercase tracking-widest ml-1">
                  {t("ess.bank_branch")}
                </label>
                <input
                  type="text"
                  value={formData.bank_branch}
                  onChange={(e) => setFormData({ ...formData, bank_branch: e.target.value })}
                  className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
                  placeholder={t("ess.branch_placeholder")}
                />
              </div>
            </div>
          </div>

          <div className="flex justify-end">
            <button
              type="submit"
              disabled={isUpdating}
              className="px-6 py-3 bg-primary-600 text-white font-bold rounded-xl hover:bg-primary-700 transition-colors disabled:opacity-50 flex items-center gap-2"
            >
              <Save className="w-4 h-4" />
              {isUpdating ? t("common.saving") : t("common.save")}
            </button>
          </div>
        </form>

        {/* Emergency Contacts */}
        <div className="bg-white rounded-3xl border border-slate-100 shadow-sm p-8 mt-6">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-bold text-slate-900 flex items-center gap-2">
              <Users className="w-5 h-5 text-primary-500" />
              {t("ess.emergency_contacts")}
            </h3>
            <button
              type="button"
              onClick={() => setIsContactModalOpen(true)}
              className="px-4 py-2 bg-primary-50 text-primary-600 font-bold rounded-xl hover:bg-primary-100 transition-colors flex items-center gap-2 text-sm"
            >
              <Plus className="w-4 h-4" />
              {t("ess.add_contact")}
            </button>
          </div>

          {contacts.length === 0 ? (
            <div className="text-center py-8 text-slate-400">
              {t("ess.no_emergency_contacts")}
            </div>
          ) : (
            <div className="space-y-3">
              {contacts.map((contact) => (
                <div
                  key={contact.id}
                  className="flex items-center justify-between p-4 bg-slate-50 rounded-xl"
                >
                  <div>
                    <div className="font-bold text-slate-900 flex items-center gap-2">
                      {contact.contact_name}
                      {contact.is_primary && (
                        <span className="text-xs bg-primary-100 text-primary-600 px-2 py-0.5 rounded-lg">
                          {t("ess.primary")}
                        </span>
                      )}
                    </div>
                    <div className="text-sm text-slate-500">
                      {contact.relationship} • {contact.phone}
                    </div>
                  </div>
                  <button
                    type="button"
                    onClick={() => handleDeleteContact(contact.id)}
                    disabled={isDeleting}
                    className="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Add Contact Modal */}
        <Modal
          isOpen={isContactModalOpen}
          onClose={() => setIsContactModalOpen(false)}
          title={t("ess.add_emergency_contact")}
          icon={<Users className="w-6 h-6" />}
        >
          <form onSubmit={handleAddContact} className="p-6 space-y-4">
            <div className="space-y-2">
              <label className="text-xs font-bold text-slate-400 uppercase tracking-widest">
                {t("ess.contact_name")}
              </label>
              <input
                type="text"
                required
                value={newContact.contact_name}
                onChange={(e) => setNewContact({ ...newContact, contact_name: e.target.value })}
                className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>

            <div className="space-y-2">
              <label className="text-xs font-bold text-slate-400 uppercase tracking-widest">
                {t("ess.relationship")}
              </label>
              <select
                required
                value={newContact.relationship}
                onChange={(e) => setNewContact({ ...newContact, relationship: e.target.value })}
                className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
              >
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
              <label className="text-xs font-bold text-slate-400 uppercase tracking-widest">
                {t("ess.phone")}
              </label>
              <input
                type="tel"
                required
                value={newContact.phone}
                onChange={(e) => setNewContact({ ...newContact, phone: e.target.value })}
                className="w-full px-4 py-3 bg-slate-50 border-0 rounded-xl focus:bg-white focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>

            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="is_primary"
                checked={newContact.is_primary}
                onChange={(e) => setNewContact({ ...newContact, is_primary: e.target.checked })}
                className="rounded border-slate-300 text-primary-600 focus:ring-primary-500"
              />
              <label htmlFor="is_primary" className="text-sm text-slate-600">
                {t("ess.set_as_primary")}
              </label>
            </div>

            <div className="flex justify-end gap-3 pt-4">
              <button
                type="button"
                onClick={() => setIsContactModalOpen(false)}
                className="px-4 py-2 text-slate-600 font-bold rounded-xl hover:bg-slate-100 transition-colors"
              >
                {t("common.cancel")}
              </button>
              <button
                type="submit"
                disabled={isAdding}
                className="px-4 py-2 bg-primary-600 text-white font-bold rounded-xl hover:bg-primary-700 transition-colors disabled:opacity-50"
              >
                {isAdding ? t("common.adding") : t("common.add")}
              </button>
            </div>
          </form>
        </Modal>
      </div>
    </div>
  );
}
