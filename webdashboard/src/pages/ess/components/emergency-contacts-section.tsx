import { Plus, Trash2, Users } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import StatusBadge from "@/components/ui/status-badge";

interface Contact {
  contact_name: string;
  id: string;
  is_primary: boolean;
  phone: string;
  relationship: string;
}

interface EmergencyContactsSectionProps {
  contacts: Contact[];
  handleDeleteContact: (id: string) => Promise<void>;
  isDeleting: boolean;
  setIsContactModalOpen: (open: boolean) => void;
}

export function EmergencyContactsSection({
  contacts,
  handleDeleteContact,
  isDeleting,
  setIsContactModalOpen,
}: EmergencyContactsSectionProps) {
  const { t } = useTranslation();

  return (
    <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm p-12 space-y-10 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 rounded-2xl bg-rose-50 dark:bg-rose-950 flex items-center justify-center text-rose-600 dark:text-rose-300 shadow-inner border border-rose-100 dark:border-rose-800/50">
            <Users size={28} />
          </div>
          <div>
            <h3 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-1">
              {t("ess.emergency_contacts")}
            </h3>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-70">
              {t("ess.emergency_contact.subtitle")}
            </p>
          </div>
        </div>
        <Button
          icon={<Plus size={18} />}
          onClick={() => {
            setIsContactModalOpen(true);
          }}
        >
          {t("ess.add_contact").toUpperCase()}
        </Button>
      </div>

      {contacts.length === 0 ? (
        <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
          <div className="w-20 h-20 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100 dark:border-slate-800">
            <Users className="text-slate-200 dark:text-slate-700" size={32} />
          </div>
          <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic">
            {t("ess.no_emergency_contacts")}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {contacts.map((contact) => (
            <div
              className="flex items-center justify-between p-8 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] hover:bg-white dark:hover:bg-slate-900 hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden shadow-inner"
              key={contact.id}
            >
              <div className="flex items-center gap-6">
                <div className="w-12 h-12 rounded-2xl bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 font-bold text-lg shadow-sm group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100 dark:border-primary-800 transition-all uppercase">
                  {contact.contact_name[0]}
                </div>
                <div>
                  <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight flex items-center gap-3 uppercase leading-none mb-2">
                    {contact.contact_name}
                    {contact.is_primary ? (
                      <StatusBadge variant="primary">{t("ess.primary")}</StatusBadge>
                    ) : null}
                  </div>
                  <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] opacity-70">
                    {contact.relationship} •{" "}
                    <span className="font-mono text-slate-500 dark:text-slate-400 font-bold">
                      {contact.phone}
                    </span>
                  </div>
                </div>
              </div>
              <button
                className="p-3 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 active:scale-90"
                disabled={isDeleting}
                onClick={() => {
                  void handleDeleteContact(contact.id);
                }}
                type="button"
              >
                <Trash2 size={18} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
