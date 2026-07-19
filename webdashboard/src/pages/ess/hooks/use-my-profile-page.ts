import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import { useEmergencyContacts, useMyProfile } from "@/hooks/use-ess";
import { logError } from "@/lib/logger";

import { NewContactData } from "../components/add-contact-modal";
import { BankDetailsFormData } from "../components/bank-details-section";
import { PersonalInfoFormData } from "../components/personal-info-section";

export type TabType = "personal" | "banking" | "emergency";
export type FullProfileFormData = BankDetailsFormData & PersonalInfoFormData;

const EMPTY_CONTACT: NewContactData = {
  contact_name: "",
  is_primary: false,
  phone: "",
  relationship: "",
};

export function useMyProfilePage() {
  const { t } = useTranslation();
  const toast = useToast();
  const { ConfirmationDialog, confirm } = useConfirm();
  const { employee, isError, isLoading, isUpdating, profile, updateProfile } = useMyProfile();
  const { addContact, contacts, deleteContact, isAdding, isDeleting } = useEmergencyContacts();

  const [activeTab, setActiveTab] = useState<TabType>("personal");
  const [isContactModalOpen, setIsContactModalOpen] = useState(false);
  const [newContact, setNewContact] = useState<NewContactData>(EMPTY_CONTACT);

  // Compute external form data — stabilized via useMemo.
  const externalFormData = useMemo((): FullProfileFormData => {
    if (profile) {
      return {
        address: profile.address ?? "",
        bank_account_number: profile.bank_account_number ?? "",
        bank_branch: profile.bank_branch ?? "",
        bank_name: profile.bank_name ?? "",
        date_of_birth: profile.date_of_birth ?? "",
        gender: profile.gender ?? "",
        national_id: profile.national_id ?? "",
        phone: profile.phone ?? "",
      };
    }
    return {
      address: "", bank_account_number: "", bank_branch: "", bank_name: "",
      date_of_birth: "", gender: "", national_id: "", phone: "",
    };
  }, [profile]);

  const [formData, setFormData] = useDerivedStateFrom(externalFormData);

  async function handleSubmit(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      await updateProfile(formData);
      toast.success(t("ess.profile.update_success"));
    } catch (error) {
      logError(error, { component: "useMyProfilePage" });
    }
  }

  async function handleAddContact(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      await addContact(newContact);
      setNewContact(EMPTY_CONTACT);
      setIsContactModalOpen(false);
      toast.success(t("ess.emergency_contact.add_success"));
    } catch (error) {
      logError(error, { component: "useMyProfilePage" });
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
      logError(error, { component: "useMyProfilePage" });
    }
  }

  return {
    t,
    employee,
    isError,
    isLoading,
    isUpdating,
    contacts,
    isAdding,
    isDeleting,
    activeTab,
    setActiveTab,
    formData,
    setFormData,
    isContactModalOpen,
    setIsContactModalOpen,
    newContact,
    setNewContact,
    handleSubmit,
    handleAddContact,
    handleDeleteContact,
    ConfirmationDialog,
  };
}
