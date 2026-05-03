import { TFunction } from "i18next";
import { AlertCircle, CreditCard, User as UserIcon, Users } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useEmergencyContacts, useMyProfile } from "@/hooks/use-ess";
import { cn } from "@/lib/utils";

import { AddContactModal, NewContactData } from "./components/add-contact-modal";
import { BankDetailsFormData, BankDetailsSection } from "./components/bank-details-section";
import { EmergencyContactsSection } from "./components/emergency-contacts-section";
import { PersonalInfoFormData, PersonalInfoSection } from "./components/personal-info-section";
import { ProfileHero } from "./components/profile-hero";

type TabType = "personal" | "banking" | "emergency";

type FullProfileFormData = BankDetailsFormData & PersonalInfoFormData;

export default function MyProfilePage() {
  const { t } = useTranslation();
  const toast = useToast();
  const { ConfirmationDialog, confirm } = useConfirm();
  const { employee, isError, isLoading, isUpdating, profile, updateProfile } = useMyProfile();
  const { addContact, contacts, deleteContact, isAdding, isDeleting } = useEmergencyContacts();

  const [activeTab, setActiveTab] = useState<TabType>("personal");
  const [formData, setFormData] = useState<FullProfileFormData>({
    address: "",
    bank_account_number: "",
    bank_branch: "",
    bank_name: "",
    date_of_birth: "",
    gender: "",
    national_id: "",
    phone: "",
  });

  const [isContactModalOpen, setIsContactModalOpen] = useState(false);
  const [newContact, setNewContact] = useState<NewContactData>({
    contact_name: "",
    is_primary: false,
    phone: "",
    relationship: "",
  });

  useEffect(() => {
    if (profile) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- Syncing form data with profile from API
      setFormData({
        address: profile.address ?? "",
        bank_account_number: profile.bank_account_number ?? "",
        bank_branch: profile.bank_branch ?? "",
        bank_name: profile.bank_name ?? "",
        date_of_birth: profile.date_of_birth ?? "",
        gender: profile.gender ?? "",
        national_id: profile.national_id ?? "",
        phone: profile.phone ?? "",
      });
    }
  }, [profile]);

  async function handleSubmit(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("ess.profile.update_success"));
    } catch (error) {
      console.error(error);
    }
  }

  async function handleAddContact(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      await addContact(newContact);
      setNewContact({ contact_name: "", is_primary: false, phone: "", relationship: "" });
      setIsContactModalOpen(false);
      toast.success(t("ess.emergency_contact.add_success"));
    } catch (error) {
      console.error(error);
    }
  }

  async function handleDeleteContact(contactId: string) {
    const ok = await confirm({
      message: t("ess.emergency_contact.confirm_delete"),
      title: t("common.confirm_delete", "Confirm Delete"),
      variant: "danger",
    });
    if (!ok) return;
    try {
      await deleteContact(contactId);
      toast.success(t("ess.emergency_contact.delete_success"));
    } catch (error) {
      console.error(error);
    }
  }

  if (isLoading) return <LoadingState />;

  if (isError || !employee) {
    return <ErrorState t={t} />;
  }

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <Header activeTab={activeTab} setActiveTab={setActiveTab} t={t} />

      <div className="animate-in slide-in-from-bottom-8 duration-700">
        <ProfileHero employee={employee} />

        {activeTab === "personal" && (
          <PersonalInfoSection
            formData={formData}
            handleSubmit={handleSubmit}
            isUpdating={isUpdating}
            setFormData={(data) => {
              setFormData((prev) => ({ ...prev, ...data }));
            }}
          />
        )}

        {activeTab === "banking" && (
          <BankDetailsSection
            formData={formData}
            handleSubmit={handleSubmit}
            isUpdating={isUpdating}
            setFormData={(data) => {
              setFormData((prev) => ({ ...prev, ...data }));
            }}
          />
        )}

        {activeTab === "emergency" && (
          <EmergencyContactsSection
            contacts={contacts}
            handleDeleteContact={handleDeleteContact}
            isDeleting={isDeleting}
            setIsContactModalOpen={setIsContactModalOpen}
          />
        )}
      </div>

      <AddContactModal
        handleAddContact={handleAddContact}
        isAdding={isAdding}
        isOpen={isContactModalOpen}
        newContact={newContact}
        onClose={() => {
          setIsContactModalOpen(false);
        }}
        setNewContact={setNewContact}
      />
      <ConfirmationDialog />
    </div>
  );
}

function Header({
  activeTab,
  setActiveTab,
  t,
}: {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  t: TFunction;
}) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div className="space-y-1">
        <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">
          {t("ess.my_profile")}
        </h1>
        <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">
          {t("ess.profile.subtitle")}
        </p>
      </div>

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <TabButton
          active={activeTab === "personal"}
          icon={<UserIcon size={14} />}
          label={t("ess.personal_info")}
          onClick={() => {
            setActiveTab("personal");
          }}
        />
        <TabButton
          active={activeTab === "banking"}
          icon={<CreditCard size={14} />}
          label={t("ess.bank_details")}
          onClick={() => {
            setActiveTab("banking");
          }}
        />
        <TabButton
          active={activeTab === "emergency"}
          icon={<Users size={14} />}
          label={t("ess.emergency_contacts")}
          onClick={() => {
            setActiveTab("emergency");
          }}
        />
      </div>
    </div>
  );
}

function LoadingState() {
  return (
    <div className="p-20 text-center">
      <div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div>
    </div>
  );
}

function ErrorState({ t }: { t: TFunction }) {
  return (
    <div className="p-8">
      <div className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 rounded-[3rem] p-12 flex items-center gap-8 shadow-2xl shadow-rose-100/50">
        <div className="bg-white dark:bg-slate-900 p-5 rounded-[1.5rem] shadow-sm border border-rose-100 dark:border-rose-800 text-rose-500">
          <AlertCircle size={40} />
        </div>
        <div>
          <h3 className="text-2xl font-bold text-rose-900 uppercase tracking-tighter leading-none mb-2">
            {t("ess.no_employee_linked")}
          </h3>
          <p className="font-bold text-rose-600 dark:text-rose-300 uppercase tracking-tight text-sm opacity-80">
            {t("ess.contact_hr")}
          </p>
        </div>
      </div>
    </div>
  );
}

function TabButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      className={cn(
        "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
        active
          ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
          : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
      )}
      onClick={onClick}
    >
      {icon} {label}
    </button>
  );
}
