import { Users } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";

export interface NewContactData {
  contact_name: string;
  is_primary: boolean;
  phone: string;
  relationship: string;
}

interface AddContactModalProps {
  handleAddContact: (e: React.SyntheticEvent) => Promise<void>;
  isAdding: boolean;
  isOpen: boolean;
  newContact: NewContactData;
  onClose: () => void;
  setNewContact: (contact: NewContactData) => void;
}

export function AddContactModal({
  handleAddContact,
  isAdding,
  isOpen,
  newContact,
  onClose,
  setNewContact,
}: AddContactModalProps) {
  const { t } = useTranslation();

  return (
    <Modal
      icon={<Users size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      title={t("ess.add_emergency_contact")}
    >
      <form
        className="space-y-8 pt-4 animate-in fade-in duration-500"
        onSubmit={(e) => {
          void handleAddContact(e);
        }}
      >
        <div className="space-y-3 px-1">
          <label className={LABEL_BASE}>{t("ess.contact_name")}</label>
          <input
            className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
            onChange={(e) => {
              setNewContact({ ...newContact, contact_name: e.target.value });
            }}
            required
            type="text"
            value={newContact.contact_name}
          />
        </div>
        <div className="grid grid-cols-2 gap-8">
          <div className="space-y-3 px-1">
            <label className={LABEL_BASE}>{t("ess.relationship")}</label>
            <Combobox
              className="w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm"
              onChange={(value) => {
                setNewContact({ ...newContact, relationship: value });
              }}
              options={[
                { value: "spouse", label: t("ess.relationship_spouse").toUpperCase() },
                { value: "parent", label: t("ess.relationship_parent").toUpperCase() },
                { value: "sibling", label: t("ess.relationship_sibling").toUpperCase() },
                { value: "child", label: t("ess.relationship_child").toUpperCase() },
                { value: "friend", label: t("ess.relationship_friend").toUpperCase() },
                { value: "other", label: t("ess.relationship_other").toUpperCase() },
              ]}
              placeholder={t("common.select").toUpperCase()}
              value={newContact.relationship}
            />
          </div>
          <div className="space-y-3 px-1">
            <label className={LABEL_BASE}>{t("ess.phone")}</label>
            <input
              className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm tracking-tighter shadow-sm"
              onChange={(e) => {
                setNewContact({ ...newContact, phone: e.target.value });
              }}
              required
              type="tel"
              value={newContact.phone}
            />
          </div>
        </div>
        <div className="flex items-center gap-4 py-2 px-2 group cursor-pointer">
          <input
            checked={newContact.is_primary}
            className="w-7 h-7 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer shadow-inner"
            id="is_primary"
            onChange={(e) => {
              setNewContact({ ...newContact, is_primary: e.target.checked });
            }}
            type="checkbox"
          />
          <label
            className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors"
            htmlFor="is_primary"
          >
            {t("ess.set_as_primary")}
          </label>
        </div>
        <div className="pt-6 grid grid-cols-2 gap-6 border-t border-slate-50 dark:border-slate-800">
          <Button onClick={onClose} size="lg" variant="outline">
            {t("common.cancel").toUpperCase()}
          </Button>
          <Button disabled={isAdding} loading={isAdding} size="lg" type="submit" variant="secondary">
            {isAdding ? t("common.adding").toUpperCase() : t("common.add").toUpperCase()}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
