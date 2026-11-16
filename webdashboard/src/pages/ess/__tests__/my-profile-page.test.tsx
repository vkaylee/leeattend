import type { ReactNode } from "react";

import { render, screen } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import MyProfilePage from "../my-profile-page";

vi.mock("../hooks/use-my-profile-page", () => ({
  useMyProfilePage: () => ({
    t: (key: string) => key,
    employee: {
      id: "employee-1",
      employee_id_number: "EMP001",
      full_name: "Test Employee",
      hire_date: "2026-01-15",
    },
    isError: false,
    isLoading: false,
    isUpdating: false,
    contacts: [],
    isAdding: false,
    isDeleting: false,
    activeTab: "personal",
    setActiveTab: vi.fn(),
    formData: {},
    setFormData: vi.fn(),
    isContactModalOpen: false,
    setIsContactModalOpen: vi.fn(),
    newContact: {},
    setNewContact: vi.fn(),
    handleSubmit: vi.fn(),
    handleAddContact: vi.fn(),
    handleDeleteContact: vi.fn(),
    ConfirmationDialog: () => null,
  }),
}));

vi.mock("../components/profile-hero", () => ({
  ProfileHero: ({ employee }: { employee: { hire_date: string } }) => <div>{employee.hire_date}</div>,
}));
vi.mock("../components/profile-common", () => ({
  ErrorState: () => <div>error</div>,
  Header: () => <div>header</div>,
  LoadingState: () => <div>loading</div>,
}));
vi.mock("../components/personal-info-section", () => ({ PersonalInfoSection: () => <div>personal</div> }));
vi.mock("../components/bank-details-section", () => ({ BankDetailsSection: () => <div>banking</div> }));
vi.mock("../components/emergency-contacts-section", () => ({ EmergencyContactsSection: () => <div>emergency</div> }));
vi.mock("../components/add-contact-modal", () => ({ AddContactModal: () => null }));

vi.mock("@/components/ui/page-container", () => ({
  PageContainer: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));

describe("MyProfilePage", () => {
  it("renders hire date from the ESS profile without management employment data", () => {
    render(<MyProfilePage />);

    expect(screen.getByText("2026-01-15")).toBeInTheDocument();
  });
});
