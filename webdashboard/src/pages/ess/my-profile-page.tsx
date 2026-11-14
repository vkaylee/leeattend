
import { useQuery } from "@tanstack/react-query";

import { PageContainer } from "@/components/ui/page-container";
import { fetchEmployeeEmployments } from "@/lib/api";

import { AddContactModal } from "./components/add-contact-modal";
import { BankDetailsSection } from "./components/bank-details-section";
import { EmergencyContactsSection } from "./components/emergency-contacts-section";
import { PersonalInfoSection } from "./components/personal-info-section";
import { ErrorState, Header, LoadingState } from "./components/profile-common";
import { ProfileHero } from "./components/profile-hero";
import { useMyProfilePage } from "./hooks/use-my-profile-page";
import { createQueryKey } from "@/lib/query-keys";


  // eslint-disable-next-line complexity
export default function MyProfilePage() {
  const {
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
  } = useMyProfilePage();


  const { data: employments = [] } = useQuery({
    queryKey: createQueryKey("employments", employee?.id),
    queryFn: () => employee ? fetchEmployeeEmployments(employee.id) : Promise.resolve([]),
    enabled: !!employee,
  });

  if (isLoading) return <LoadingState />;

  if (isError || !employee) {
    return <ErrorState t={t} />;
  }

  const activeEmployment = employments.find(emp => emp.id === employee.active_employment_id);
  const hireDate = activeEmployment?.start_date ?? "—";

  return (
    <PageContainer>
      <Header activeTab={activeTab} setActiveTab={setActiveTab} t={t} />

      <div className="animate-in slide-in-from-bottom-8 duration-700">
        <ProfileHero employee={{
          employee_id_number: employee.employee_id_number,
          full_name: employee.full_name,
          hire_date: hireDate,
        }} />

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
    </PageContainer>
  );
}
