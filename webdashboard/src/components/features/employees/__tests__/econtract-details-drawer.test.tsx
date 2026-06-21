import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import { saveEContractEnvelope, EContractEnvelope } from "@/lib/api/econtracts";
import EContractDetailsDrawer from "../econtract-details-drawer";

// Mock saveEContractEnvelope API call
vi.mock("@/lib/api/econtracts", () => ({
  saveEContractEnvelope: vi.fn(),
}));

const mockEnvelopePending: EContractEnvelope = {
  id: "env-1",
  contract_id: "contract-1",
  employer_id: "employer-1",
  provider_type: "smartca",
  external_envelope_id: "VNPT-ENV-887192",
  status: "sent",
  signer_statuses: [
    {
      signer_id: "rep-1",
      name: "HR Representative",
      email: "hr@domain.com",
      role: "representative",
      routing_order: 1,
      status: "signed",
      signed_at: "2025-01-01T08:00:00Z",
    },
    {
      signer_id: "emp-1",
      name: "Employee Name",
      email: "employee@domain.com",
      role: "employee",
      routing_order: 2,
      status: "pending",
    },
  ],
  audit_trail_metadata: {
    webhook_events: [],
  },
  created_at: "2025-01-01T07:00:00Z",
};

const mockEnvelopeCompleted: EContractEnvelope = {
  id: "env-1",
  contract_id: "contract-1",
  employer_id: "employer-1",
  provider_type: "smartca",
  external_envelope_id: "VNPT-ENV-887192",
  status: "completed",
  signer_statuses: [
    {
      signer_id: "rep-1",
      name: "HR Representative",
      email: "hr@domain.com",
      role: "representative",
      routing_order: 1,
      status: "signed",
      signed_at: "2025-01-01T08:00:00Z",
    },
    {
      signer_id: "emp-1",
      name: "Employee Name",
      email: "employee@domain.com",
      role: "employee",
      routing_order: 2,
      status: "signed",
      signed_at: "2025-01-01T09:00:00Z",
    },
  ],
  audit_trail_metadata: {
    webhook_events: [
      { event: "envelope.completed", timestamp: "2025-01-01T09:00:00Z" }
    ],
  },
  created_at: "2025-01-01T07:00:00Z",
};

describe("EContractDetailsDrawer Component", () => {
  let queryClient: QueryClient;

  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    wsId: "ws-1",
  };

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    (saveEContractEnvelope as Mock).mockResolvedValue(mockEnvelopeCompleted);
  });

  const renderDrawer = (envelope: EContractEnvelope | null, props: Partial<typeof defaultProps> = {}) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <EContractDetailsDrawer envelope={envelope} {...defaultProps} {...props} />
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders nothing when envelope is null", () => {
    renderDrawer(null);
    expect(screen.queryByText("Chi tiết Ký số (e-Sign Audit)")).toBeNull();
    expect(screen.queryByText(/Trình tự ký hợp đồng/i)).toBeNull();
  });

  it("renders drawer header and timeline in open state", () => {
    renderDrawer(mockEnvelopePending);
    expect(screen.getByText("Chi tiết Ký số (e-Sign Audit)")).toBeInTheDocument();
    expect(screen.getByText("VNPT-ENV-887192", { exact: false })).toBeInTheDocument();
    expect(screen.getByText("HR Representative")).toBeInTheDocument();
    expect(screen.getByText("Employee Name")).toBeInTheDocument();
  });

  it("shows resend notification action button and clicks it successfully", async () => {
    const user = userEvent.setup();
    renderDrawer(mockEnvelopePending);

    const resendBtn = screen.getByText(/Gửi lại yêu cầu/i);
    expect(resendBtn).toBeInTheDocument();

    await user.click(resendBtn);
    // Note: toast success message gets rendered in body
    expect(screen.getByText(/Đã gửi lại yêu cầu ký số thành công/i)).toBeInTheDocument();
  });

  it("shows Download PDF button when envelope is completed", async () => {
    const user = userEvent.setup();
    // Mock window.URL.createObjectURL and click triggers if needed, but HTML5 link click works in test env
    renderDrawer(mockEnvelopeCompleted);

    const downloadBtn = screen.getByText(/Tải chứng thư/i);
    expect(downloadBtn).toBeInTheDocument();
    expect(screen.queryByText(/Ký số ngay/i)).toBeNull();

    await user.click(downloadBtn);
    expect(screen.getByText(/Tải xuống tệp PDF hợp đồng thành công/i)).toBeInTheDocument();
  });

  it("shows Sign Now button and runs simulation portal workflow", async () => {
    const user = userEvent.setup();
    renderDrawer(mockEnvelopePending);

    const signBtn = screen.getByText(/Ký số ngay/i);
    expect(signBtn).toBeInTheDocument();

    await user.click(signBtn);

    // E-Sign modal should be open
    expect(screen.getByText("Ký Số Điện Tử (E-Sign Portal)")).toBeInTheDocument();

    // Fill signature sample
    const signatureInput = screen.getByPlaceholderText(/Nhập tên của bạn để tạo chữ ký mẫu/i);
    await user.type(signatureInput, "Nguyễn Văn A");

    // Click Get OTP
    const getOtpBtn = screen.getByText(/Lấy mã OTP/i);
    await user.click(getOtpBtn);
    expect(screen.getByText(/Đã gửi mã OTP đến Số điện thoại/i)).toBeInTheDocument();

    // Fill OTP
    const otpInput = screen.getByLabelText(/Mã OTP xác nhận/i);
    await user.type(otpInput, "1234");

    // Click complete sign
    const completeBtn = screen.getByText(/HOÀN TẤT KÝ SỐ/i);
    await user.click(completeBtn);

    await waitFor(() => {
      expect(saveEContractEnvelope).toHaveBeenCalledTimes(1);
      expect(saveEContractEnvelope).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          status: "completed", // Both representative and employee signed
        })
      );
    });
  });

  it("handles invalid OTP length and closes signature modal on Back click", async () => {
    const user = userEvent.setup();
    renderDrawer(mockEnvelopePending);

    const signBtn = screen.getByText(/Ký số ngay/i);
    await user.click(signBtn);

    // E-Sign modal should be open
    expect(screen.getByText("Ký Số Điện Tử (E-Sign Portal)")).toBeInTheDocument();

    // Click HOÀN TẤT KÝ SỐ without entering OTP (empty)
    const completeBtn = screen.getByText(/HOÀN TẤT KÝ SỐ/i);
    await user.click(completeBtn);

    // Verify error toast
    expect(screen.getByText(/Vui lòng nhập mã OTP hợp lệ/i)).toBeInTheDocument();

    // Enter short OTP (length < 4)
    const otpInput = screen.getByLabelText(/Mã OTP xác nhận/i);
    await user.type(otpInput, "12");
    await user.click(completeBtn);
    expect(screen.getAllByText(/Vui lòng nhập mã OTP hợp lệ/i).length).toBeGreaterThan(0);

    // Click QUAY LẠI button to close modal
    const backBtn = screen.getByText(/QUAY LẠI/i);
    await user.click(backBtn);

    // Modal should be closed
    expect(screen.queryByText("Ký Số Điện Tử (E-Sign Portal)")).toBeNull();
  });
});
