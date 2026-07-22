import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import {
  fetchEContractConfigs,
  saveEContractConfig,
  fetchEContractRules,
  saveEContractRule,
  deleteEContractRule,
  fetchEContractEnvelopes,
} from "@/lib/api/econtracts";
import { fetchEmployers } from "@/lib/api/employers";

import EContractsSettingsPage from "../econtracts-settings-page";

// Mock i18next translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => options?.defaultValue || key,
  }),
}));

// Mock current workspace ID
let mockWorkspaceId = "ws-1";
vi.mock("@/lib/api/helpers", () => ({
  getCurrentWorkspaceId: () => mockWorkspaceId,
}));

// Mock API endpoints
vi.mock("@/lib/api/employers", () => ({
  fetchEmployers: vi.fn(),
}));

vi.mock("@/lib/api/econtracts", () => ({
  fetchEContractConfigs: vi.fn(),
  saveEContractConfig: vi.fn(),
  fetchEContractRules: vi.fn(),
  saveEContractRule: vi.fn(),
  deleteEContractRule: vi.fn(),
  fetchEContractEnvelopes: vi.fn(),
}));

const mockEmployers = [
  { id: "employer-1", name: "LeeAttend Solutions Inc." },
  { id: "employer-2", name: "Global HR Corp." },
];

const mockConfigs = [
  {
    id: "config-1",
    employer_id: "employer-1",
    provider_type: "smartca",
    api_url: "https://api.smartca.vn",
    client_id: "client-smartca-1",
    client_secret_encrypted: "••••••••",
    webhook_secret_encrypted: "whsec-1",
    default_auth_type: "smartca" as const,
    is_active: true,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  },
];

const mockRules = [
  {
    id: "rule-1",
    employer_id: "employer-1",
    signer_country: "VN",
    signing_method: "smartca" as const,
    target_provider_type: "smartca",
    priority: 10,
    created_at: "2025-01-01T00:00:00Z",
  },
  {
    id: "rule-2",
    employer_id: "employer-1",
    signer_country: "TH",
    signing_method: "sms" as const,
    target_provider_type: "smartca",
    priority: 20,
    created_at: "2025-01-01T00:00:00Z",
  },
];

const mockEnvelopes = [
  {
    id: "env-1",
    contract_id: "contract-1",
    employer_id: "employer-1",
    provider_type: "smartca",
    external_envelope_id: "VNPT-ENV-887192",
    status: "completed" as const,
    signer_statuses: [],
    audit_trail_metadata: { log: "audit-trail-mock" },
    created_at: "2025-01-01T07:00:00Z",
  },
  {
    id: "env-2",
    contract_id: "contract-2",
    employer_id: "employer-1",
    provider_type: "smartca",
    external_envelope_id: "VNPT-ENV-887193",
    status: "sent" as const,
    signer_statuses: [],
    audit_trail_metadata: null,
    created_at: "2025-01-02T07:00:00Z",
  },
  {
    id: "env-3",
    contract_id: "contract-3",
    employer_id: "employer-1",
    provider_type: "hsm",
    external_envelope_id: "HSM-ENV-998822",
    status: "draft" as const,
    signer_statuses: [],
    audit_trail_metadata: undefined,
    created_at: "2025-01-03T07:00:00Z",
  },
];

describe("EContractsSettingsPage Component", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    mockWorkspaceId = "ws-1";
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0 },
        mutations: { retry: false },
      },
    });

    (fetchEmployers as Mock).mockResolvedValue(mockEmployers);
    (fetchEContractConfigs as Mock).mockResolvedValue(mockConfigs);
    (fetchEContractRules as Mock).mockResolvedValue(mockRules);
    (fetchEContractEnvelopes as Mock).mockResolvedValue(mockEnvelopes);
    (saveEContractConfig as Mock).mockResolvedValue(mockConfigs[0]);
    (saveEContractRule as Mock).mockResolvedValue(mockRules[0]);
    (deleteEContractRule as Mock).mockResolvedValue(undefined);
  });

  const renderPage = () => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <EContractsSettingsPage />
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders page header and active legal entity Picker", async () => {
    renderPage();
    expect(screen.getByText("e-Contracts Integration")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getAllByText("LEEATTEND SOLUTIONS INC.").length).toBeGreaterThan(0);
    });
  });

  it("switches context when select employer changes", async () => {
    renderPage();

    await waitFor(() => {
      expect(screen.getAllByText("LEEATTEND SOLUTIONS INC.").length).toBeGreaterThan(0);
    });

    const select = screen.getByLabelText("Select Legal Entity") as HTMLSelectElement;
    expect(select.value).toBe("employer-1");
    fireEvent.change(select, { target: { value: "employer-2" } });
    expect(select.value).toBe("employer-2");

    await waitFor(() => {
      expect(screen.getAllByText("GLOBAL HR CORP.").length).toBeGreaterThan(0);
    });
  });

  // ── Credentials Manager Tab ────────────────────────────────────────

  it("allows filling and saving API credentials config", async () => {
    const user = userEvent.setup();
    renderPage();

    // Fields should auto-populate from config-1
    await waitFor(() => {
      const urlInput = screen.getByLabelText(/API Gateway Endpoint URL/i) as HTMLInputElement;
      expect(urlInput.value).toBe("https://api.smartca.vn");
    });

    // Modify fields and submit form
    const urlInput = screen.getByLabelText(/API Gateway Endpoint URL/i);
    await user.clear(urlInput);
    await user.type(urlInput, "https://api.new-smartca.vn");

    // Modify dropdown selects
    const providerSelect = screen.getByLabelText(/Provider Type/i);
    await user.selectOptions(providerSelect, "hsm");

    const defaultAuthSelect = screen.getByLabelText(/Default Authorization Verification/i);
    await user.selectOptions(defaultAuthSelect, "sms");

    const clientIdInput = screen.getByLabelText(/Integration Client ID/i);
    await user.clear(clientIdInput);
    await user.type(clientIdInput, "client-id-new");

    const secretInput = screen.getByLabelText(/Client Credentials Secret/i);
    await user.clear(secretInput);
    await user.type(secretInput, "secret-new");

    const webhookSecretInput = screen.getByLabelText(/Webhook Secret/i);
    await user.clear(webhookSecretInput);
    await user.type(webhookSecretInput, "whsec-new");

    const hsmAliasInput = screen.getByLabelText(/HSM Key Alias/i);
    await user.clear(hsmAliasInput);
    await user.type(hsmAliasInput, "hsm-alias-new");

    const submitBtn = screen.getByText("SAVE CONFIGURATIONS");
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(saveEContractConfig).toHaveBeenCalledTimes(1);
      expect(saveEContractConfig).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          api_url: "https://api.new-smartca.vn",
          client_id: "client-id-new",
          client_secret_encrypted: "secret-new",
          webhook_secret_encrypted: "whsec-new",
          hsm_key_alias: "hsm-alias-new",
          employer_id: "employer-1",
          provider_type: "hsm",
          default_auth_type: "sms",
        })
      );
    });
  });

  it("toggles active config switch correctly", async () => {
    renderPage();

    await waitFor(() => {
      const switchBtn = screen.getByRole("switch");
      expect(switchBtn).toBeInTheDocument();
      expect(switchBtn.getAttribute("aria-checked")).toBe("true");
    });

    const switchBtn = screen.getByRole("switch");
    fireEvent.click(switchBtn);

    // Switch should turn false
    expect(switchBtn.getAttribute("aria-checked")).toBe("false");

    // Keydown Space to turn back to true
    fireEvent.keyDown(switchBtn, { key: " " });
    expect(switchBtn.getAttribute("aria-checked")).toBe("true");

    // Keydown Enter to turn false again
    fireEvent.keyDown(switchBtn, { key: "Enter" });
    expect(switchBtn.getAttribute("aria-checked")).toBe("false");
  });

  // ── Routing Priority Rules Tab ──────────────────────────────────────

  it("navigates to routing rules tab and allows adding/canceling a rule", async () => {
    const user = userEvent.setup();
    renderPage();

    const rulesTabTrigger = screen.getByText(/Routing Priority Rules/i);
    await user.click(rulesTabTrigger);

    // Verify rules table is rendered
    await waitFor(() => {
      expect(screen.getAllByText("VN")[0]).toBeInTheDocument();
    });

    // 1. Open and cancel rule modal using CANCEL button
    const addBtn = screen.getByText("ADD ROUTING RULE");
    await user.click(addBtn);

    const cancelBtn = screen.getByText("CANCEL");
    await user.click(cancelBtn);

    // 2. Open and cancel rule modal using Close dialog icon
    await user.click(addBtn);
    const closeDialogIconBtn = screen.getByRole("button", { name: /Close dialog/i });
    await user.click(closeDialogIconBtn);

    // 3. Open again to fill and submit
    await user.click(addBtn);

    // Fill rule form
    const countryInput = screen.getByLabelText(/Signer Country Code/i);
    await user.clear(countryInput);
    await user.type(countryInput, "US");

    const authSelect = screen.getByLabelText(/Preferred Signing Auth/i);
    await user.selectOptions(authSelect, "hsm");

    const providerSelect = screen.getByLabelText(/Target Provider Route/i);
    await user.selectOptions(providerSelect, "hsm");

    const priorityInput = screen.getByLabelText(/Priority Weight/i);
    fireEvent.change(priorityInput, { target: { value: "20" } });

    const completeBtn = screen.getByText("ADD RULE");
    await user.click(completeBtn);

    await waitFor(() => {
      expect(saveEContractRule).toHaveBeenCalledTimes(1);
      expect(saveEContractRule).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          signer_country: "US",
          signing_method: "hsm",
          target_provider_type: "hsm",
          priority: 20,
          employer_id: "employer-1",
        })
      );
    });
  });

  it("allows deleting routing rule successfully", async () => {
    const user = userEvent.setup();
    renderPage();

    const rulesTabTrigger = screen.getByText(/Routing Priority Rules/i);
    await user.click(rulesTabTrigger);

    await waitFor(() => {
      expect(screen.getAllByText("VN")[0]).toBeInTheDocument();
    });

    const deleteBtn = screen.getAllByLabelText("Delete rule")[0];
    await user.click(deleteBtn);

    await waitFor(() => {
      expect(deleteEContractRule).toHaveBeenCalledTimes(1);
      expect(deleteEContractRule).toHaveBeenCalledWith("ws-1", "rule-2");
    });
  });

  // ── Envelope Audit Dashboard Tab ────────────────────────────────────

  it("navigates to envelope audit dashboard and tests filter + webhook modal", async () => {
    const user = userEvent.setup();
    renderPage();

    const envTabTrigger = screen.getByText(/Envelope Audit Dashboard/i);
    await user.click(envTabTrigger);

    // Verify envelope table renders
    await waitFor(() => {
      expect(screen.getAllByText("VNPT-ENV-887192")[0]).toBeInTheDocument();
    });

    // Test Search filter
    const searchInput = screen.getByPlaceholderText(/Search envelopes by ID/i);
    await user.type(searchInput, "NON-EXISTENT-ID");
    expect(screen.queryByText("VNPT-ENV-887192")).toBeNull();

    // Clear search
    await user.clear(searchInput);
    expect(screen.getAllByText("VNPT-ENV-887192")[0]).toBeInTheDocument();

    // Test select filters
    const providerFilter = screen.getByLabelText("Filter by Provider") as HTMLSelectElement;
    fireEvent.change(providerFilter, { target: { value: "smartca" } });
    expect(providerFilter.value).toBe("smartca");

    const statusFilter = screen.getByLabelText("Filter by Status") as HTMLSelectElement;
    fireEvent.change(statusFilter, { target: { value: "completed" } });
    expect(statusFilter.value).toBe("completed");

    // Open Webhook logs modal
    const logsBtns = screen.getAllByText("WEBHOOK LOGS");
    await user.click(logsBtns[0]);

    // Verify audit logs are displayed
    expect(screen.getByText("Webhooks & Audit Trail")).toBeInTheDocument();
    expect(screen.getByText(/audit-trail-mock/i)).toBeInTheDocument();

    // Close logs modal using inline CLOSE button
    const closeBtn = screen.getByText("CLOSE");
    await user.click(closeBtn);
    expect(screen.queryByText("Webhooks & Audit Trail")).toBeNull();

    // Open logs modal again
    const logsBtnsAgain = screen.getAllByText("WEBHOOK LOGS");
    await user.click(logsBtnsAgain[0]);

    // Close logs modal using standard close dialog icon
    const closeDialogIconBtn = screen.getByRole("button", { name: /Close dialog/i });
    await user.click(closeDialogIconBtn);
    expect(screen.queryByText("Webhooks & Audit Trail")).toBeNull();
  });

  it("navigates to routing rules tab for employer-2, displays EmptyState, and opens modal from there", async () => {
    const user = userEvent.setup();
    renderPage();

    await waitFor(() => {
      expect(screen.getAllByText("LEEATTEND SOLUTIONS INC.").length).toBeGreaterThan(0);
    });

    // Select employer-2 (has no rules)
    const select = screen.getByLabelText("Select Legal Entity") as HTMLSelectElement;
    fireEvent.change(select, { target: { value: "employer-2" } });

    // Navigate to routing rules tab
    const rulesTabTrigger = screen.getByText(/Routing Priority Rules/i);
    await user.click(rulesTabTrigger);

    // Verify Empty State is shown
    await waitFor(() => {
      expect(screen.getByText("No Precedence Rules Found")).toBeInTheDocument();
    });

    // Click "Create First Rule" button to cover EmptyState action handler
    const createFirstBtn = screen.getByText("Create First Rule");
    await user.click(createFirstBtn);

    // Modal should be open
    expect(screen.getByText("Add Routing Rule")).toBeInTheDocument();
  });

  it("shows error toast when saveEContractConfig fails", async () => {
    (saveEContractConfig as Mock).mockRejectedValueOnce(new Error("API Error"));
    renderPage();

    await waitFor(() => {
      const urlInput = screen.getByLabelText(/API Gateway Endpoint URL/i) as HTMLInputElement;
      expect(urlInput.value).toBe("https://api.smartca.vn");
    });

    const submitBtn = screen.getByText("SAVE CONFIGURATIONS");
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText("API Error")).toBeInTheDocument();
    });
  });

  it("shows generic error toast when saveEContractConfig fails without message", async () => {
    (saveEContractConfig as Mock).mockRejectedValueOnce({ foo: "bar" });
    renderPage();

    await waitFor(() => {
      const urlInput = screen.getByLabelText(/API Gateway Endpoint URL/i) as HTMLInputElement;
      expect(urlInput.value).toBe("https://api.smartca.vn");
    });

    const submitBtn = screen.getByText("SAVE CONFIGURATIONS");
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText("Failed to save configuration")).toBeInTheDocument();
    });
  });

  it("covers fallback defaults for empty configuration form", async () => {
    // Return empty configs so it falls back to default values
    (fetchEContractConfigs as Mock).mockResolvedValueOnce([]);
    renderPage();

    await waitFor(() => {
      // API Gateway URL should be empty
      const urlInput = screen.getByLabelText(/API Gateway Endpoint URL/i) as HTMLInputElement;
      expect(urlInput.value).toBe("");
    });

    // Provider select fallback value
    const providerSelect = screen.getByLabelText(/Provider Type/i) as HTMLSelectElement;
    expect(providerSelect.value).toBe("smartca");

    // Toggle active state switch keyboard handler
    const activeSwitch = screen.getByRole("switch");
    fireEvent.keyDown(activeSwitch, { key: "Enter" });
    expect(activeSwitch.getAttribute("aria-checked")).toBe("false");
    fireEvent.keyDown(activeSwitch, { key: " " });
    expect(activeSwitch.getAttribute("aria-checked")).toBe("true");
  });

  it("handles empty employers lists and saving without selected employer", async () => {
    (fetchEmployers as Mock).mockResolvedValueOnce([]);
    (fetchEContractConfigs as Mock).mockResolvedValueOnce([]);
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("No employers found")).toBeInTheDocument();
    });
  });

  it("covers envelope status badge styles and webhook logs modal metadata fallback", async () => {
    const user = userEvent.setup();
    renderPage();

    await waitFor(() => {
      expect(screen.getAllByText("LEEATTEND SOLUTIONS INC.").length).toBeGreaterThan(0);
    });

    // Switch to Audit tab
    const auditTabTrigger = screen.getByText(/Envelope Audit Dashboard/i);
    await user.click(auditTabTrigger);

    // Verify all 3 envelopes with different statuses are rendered
    await waitFor(() => {
      expect(screen.getAllByText("VNPT-ENV-887192")[0]).toBeInTheDocument(); // completed
      expect(screen.getAllByText("VNPT-ENV-887193")[0]).toBeInTheDocument(); // sent
      expect(screen.getAllByText("HSM-ENV-998822")[0]).toBeInTheDocument(); // draft (fallback badge class)
    });

    // Click Webhook logs for the second envelope (has null metadata)
    const webhookBtns = screen.getAllByText("WEBHOOK LOGS");
    await user.click(webhookBtns[1]); // VNPT-ENV-887193

    // Audit metadata fallback should be rendered
    await waitFor(() => {
      expect(screen.getByText(/"No callback metadata captured"/)).toBeInTheDocument();
    });

    // Close modal
    const closeBtn = screen.getByText("CLOSE");
    await user.click(closeBtn);
    expect(screen.queryByText("Webhooks & Audit Trail")).not.toBeInTheDocument();
  });

  it("handles rule creation when no legal entity is selected", async () => {
    (fetchEmployers as Mock).mockResolvedValueOnce([]);
    (fetchEContractRules as Mock).mockResolvedValueOnce([]);
    const user = userEvent.setup();
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("No employers found")).toBeInTheDocument();
    });

    // Switch to rules tab
    const rulesTabTrigger = screen.getByText(/Routing Priority Rules/i);
    await user.click(rulesTabTrigger);

    // Verify Empty State is shown
    await waitFor(() => {
      expect(screen.getByText("No Precedence Rules Found")).toBeInTheDocument();
    });

    // Click "Create First Rule" to open modal
    const createFirstBtn = screen.getByText("Create First Rule");
    await user.click(createFirstBtn);

    // Modal should be open
    expect(screen.getByText("Add Routing Rule")).toBeInTheDocument();

    // Click submit button in the modal
    const addRuleBtn = screen.getByText("ADD RULE");
    await user.click(addRuleBtn);

    // Expect saveEContractRule not to have been called because selectedEmployerId is empty
    expect(saveEContractRule).not.toHaveBeenCalled();
  });

  it("shows error toast when saving config without a selected employer", async () => {
    (fetchEmployers as Mock).mockResolvedValueOnce([]);
    (fetchEContractConfigs as Mock).mockResolvedValueOnce([]);
    renderPage();

    await waitFor(() => {
      expect(screen.getByText("No employers found")).toBeInTheDocument();
    });

    const submitBtn = screen.getByText("SAVE CONFIGURATIONS");
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText("Please select a Legal Entity.")).toBeInTheDocument();
    });
  });

  it("renders select workspace message when wsId is empty", () => {
    mockWorkspaceId = "";
    renderPage();
    expect(screen.getByText("common.select_workspace")).toBeInTheDocument();
  });
});
