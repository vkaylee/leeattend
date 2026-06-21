import { useMutation, useQueryClient } from "@tanstack/react-query";
import { X, KeyRound, CheckCircle, Clock, Send, Download, ShieldAlert, Sparkles, FileSignature } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { EContractEnvelope, saveEContractEnvelope, SignerStatus } from "@/lib/api/econtracts";
import { cn } from "@/lib/utils";

interface EContractDetailsDrawerProps {
  envelope: EContractEnvelope | null;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

export default function EContractDetailsDrawer({
  envelope,
  isOpen,
  onClose,
  wsId,
}: EContractDetailsDrawerProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const queryClient = useQueryClient();

  const [isSigningModalOpen, setIsSigningModalOpen] = useState(false);
  const [signatureText, setSignatureText] = useState("");
  const [otpCode, setOtpCode] = useState("");
  const [isOtpSent, setIsOtpSent] = useState(false);

  const envelopeMutation = useMutation({
    mutationFn: (env: EContractEnvelope) => saveEContractEnvelope(wsId, env),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["econtract-envelopes", wsId] });
    },
  });

  if (!envelope) return null;

  const handleResendNotification = () => {
    toast.success("Đã gửi lại yêu cầu ký số thành công đến toàn bộ người ký!");
  };

  const handleDownloadPDF = () => {
    // Generate dummy download
    const link = document.createElement("a");
    link.href = "data:application/pdf;base64,JVBERi0xLjQKJd...[MOCK SIGNED PDF]...";
    link.download = `Signed_Contract_${envelope.external_envelope_id}.pdf`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    toast.success("Tải xuống tệp PDF hợp đồng thành công!");
  };

  const handleCompleteSign = () => {
    if (!otpCode || otpCode.length < 4) {
      toast.error("Vui lòng nhập mã OTP hợp lệ!");
      return;
    }

    // Update signer status to signed
    const updatedSigners: SignerStatus[] = envelope.signer_statuses.map((signer) => {
      if (signer.role === "employee" && signer.status === "pending") {
        return {
          ...signer,
          status: "signed",
          signed_at: new Date().toISOString(),
        };
      }
      return signer;
    });

    const isAllSigned = updatedSigners.every((s) => s.status === "signed");

    const updatedEnv: EContractEnvelope = {
      ...envelope,
      status: isAllSigned ? "completed" : "sent",
      signer_statuses: updatedSigners,
      audit_trail_metadata: {
        webhook_events: [
          ...(envelope.audit_trail_metadata?.webhook_events || []),
          { event: "signer.employee.signed", timestamp: new Date().toISOString() },
          ...(isAllSigned ? [{ event: "envelope.completed", timestamp: new Date().toISOString() }] : []),
        ],
      },
    };

    envelopeMutation.mutate(updatedEnv);
    setIsSigningModalOpen(false);
    toast.success("Bạn đã ký số thành công!");
    onClose();
  };

  const isEmployeePending = envelope.signer_statuses.some(
    (s) => s.role === "employee" && s.status === "pending"
  );

  return (
    <>
      <Sheet
        ariaDescribedBy="e-sign-audit-description"
        ariaLabelledBy="e-sign-audit-title"
        className="!max-w-2xl"
        contentClassName="!p-0"
        hideHeader
        isOpen={isOpen}
        onClose={onClose}
      >
        {/* Header */}
        <div className="px-8 py-6 border-b border-slate-100 dark:border-slate-800 flex items-center justify-between bg-slate-50/50 dark:bg-slate-900/50">
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 border border-primary-100 dark:border-primary-900 shadow-sm">
              <KeyRound size={20} />
            </div>
            <div>
              <Typography className="text-sm font-bold uppercase tracking-widest text-slate-800 dark:text-slate-100" id="e-sign-audit-title" variant="h2">
                Chi tiết Ký số (e-Sign Audit)
              </Typography>
              <span className="text-[10px] font-mono text-slate-400 uppercase tracking-widest block" id="e-sign-audit-description">
                ID: {envelope.external_envelope_id}
              </span>
            </div>
          </div>
          <Button
            aria-label="Close drawer"
            className="p-2 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-full transition-colors h-auto w-auto shadow-none border-none"
            onClick={onClose}
            variant="ghost"
          >
            <X className="w-5 h-5 text-slate-400" />
          </Button>
        </div>

        {/* Content */}
        <div className="p-8 space-y-8">
          {/* General Metadata Card */}
          <div className="p-6 border border-slate-100 dark:border-slate-800 bg-slate-50/30 dark:bg-slate-900/30 rounded-3xl grid grid-cols-2 gap-4">
            <div>
              <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest block mb-0.5">Nhà Cung cấp</span>
              <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wider">
                {envelope.provider_type}
              </span>
            </div>
            <div>
              <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest block mb-0.5">Trạng thái Giao dịch</span>
              <span
                className={cn(
                  "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 rounded-full border shadow-sm inline-block",
                  envelope.status === "completed"
                    ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800"
                    : "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/40 dark:text-amber-300 dark:border-amber-800"
                )}
              >
                {envelope.status}
              </span>
            </div>
          </div>

          {/* Timeline Process steps */}
          <div className="space-y-4">
            <Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-[0.2em]" variant="h3">Trình tự ký hợp đồng (Signing Order)</Typography>
            <div className="relative pl-6 space-y-6 before:absolute before:left-[10px] before:top-2 before:bottom-2 before:w-0.5 before:bg-slate-100 dark:before:bg-slate-800">
              {envelope.signer_statuses.map((signer, index) => {
                const isSigned = signer.status === "signed";
                return (
                  <div className="relative flex items-start gap-4" key={signer.signer_id}>
                    <div
                      className={cn(
                        "absolute -left-7 top-1 w-4 h-4 rounded-full border-2 border-white dark:border-slate-950 shadow-sm z-10 flex items-center justify-center",
                        isSigned
                          ? "bg-emerald-500 border-emerald-500 text-white"
                          : "bg-amber-500 border-amber-500 text-white"
                      )}
                    >
                      {isSigned ? <CheckCircle size={10} /> : <Clock size={10} />}
                    </div>
                    <div className="flex-1 bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-850 p-4 rounded-2xl shadow-sm">
                      <div className="flex justify-between items-start">
                        <div>
                          <span className="text-xs font-bold text-slate-850 dark:text-slate-100 block">
                            {signer.name}
                          </span>
                          <span className="text-[9px] text-slate-400 font-semibold block uppercase tracking-wider mt-0.5">
                            Vai trò: {signer.role === "representative" ? "Đại diện công ty" : "Nhân sự"} • Thứ tự: {signer.routing_order}
                          </span>
                        </div>
                        <span
                          className={cn(
                            "text-[8px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border",
                            isSigned
                              ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-350"
                              : "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/40 dark:text-amber-350"
                          )}
                        >
                          {signer.status}
                        </span>
                      </div>
                      {!!signer.signed_at && (
                        <div className="mt-2 text-[8px] font-mono text-slate-400 block uppercase tracking-wider">
                          Đã ký lúc: {new Date(signer.signed_at).toLocaleString()}
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Action buttons footer drawer */}
          <div className="pt-6 border-t border-slate-100 dark:border-slate-800 flex flex-wrap gap-3">
            <Button
              className="flex items-center gap-1 text-[11px] font-bold uppercase tracking-widest h-11"
              icon={<Send size={14} />}
              onClick={handleResendNotification}
              variant="outline"
            >
              Gửi lại yêu cầu (Resend Notification)
            </Button>

            {envelope.status === "completed" ? (
              <Button
                className="flex items-center gap-1 text-[11px] font-bold uppercase tracking-widest h-11"
                icon={<Download size={14} />}
                onClick={handleDownloadPDF}
                variant="outline"
              >
                Tải chứng thư (Download PDF)
              </Button>
            ) : isEmployeePending ? (
              <Button
                className="flex items-center gap-1 text-[11px] font-bold uppercase tracking-widest h-11 bg-primary-600 hover:bg-primary-700 text-white"
                icon={<FileSignature size={14} />}
                onClick={() => { setIsSigningModalOpen(true); }}
                variant="secondary"
              >
                Ký số ngay (Sign Now)
              </Button>
            ) : null}
          </div>
        </div>
      </Sheet>

      {/* Signature Iframe simulation overlay modal */}
      <Modal
        icon={<Sparkles size={24} />}
        isOpen={isSigningModalOpen}
        onClose={() => { setIsSigningModalOpen(false); }}
        subtitle={`SmartCA/OTP Signing Session for ${envelope.external_envelope_id}`}
        title="Ký Số Điện Tử (E-Sign Portal)"
      >
        <div className="space-y-6 pt-4">
          <div className="p-4 bg-amber-50/50 dark:bg-amber-950/20 border border-amber-100 dark:border-amber-900 rounded-2xl flex items-start gap-3">
            <ShieldAlert className="text-amber-500 shrink-0 mt-0.5" size={16} />
            <div>
              <Typography className="text-xs font-bold text-amber-800 dark:text-amber-400 uppercase tracking-wider" variant="h4">
                Xác thực bảo mật đa nhân tố
              </Typography>
              <Typography className="text-[10px] text-amber-600 dark:text-amber-500 font-semibold uppercase tracking-wider leading-relaxed mt-0.5" variant="p">
                Vui lòng vẽ chữ ký số và nhập mã xác thực OTP gửi đến Số điện thoại/Email đăng ký để hoàn tất.
              </Typography>
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="signature-input">
              Chữ ký số (Draw/Type Signature)
            </label>
            <div className="h-32 border-2 border-dashed border-slate-200 dark:border-slate-800 rounded-3xl bg-slate-50/50 dark:bg-slate-900/50 flex flex-col items-center justify-center p-4 relative group">
              <input
                className="w-full text-center bg-transparent border-none text-2xl font-semibold italic text-primary-600 dark:text-primary-400 focus:outline-none placeholder:text-slate-300"
                id="signature-input"
                onChange={(e) => { setSignatureText(e.target.value); }}
                placeholder="Nhập tên của bạn để tạo chữ ký mẫu"
                type="text"
                value={signatureText}
              />
              <span className="absolute bottom-2 text-[9px] text-slate-400 font-bold uppercase tracking-widest opacity-60">
                Signature Canvas Pad Mock
              </span>
            </div>
          </div>

          <div className="grid grid-cols-3 gap-4 items-end">
            <div className="col-span-2">
              <FormField
                id="otp-code-input"
                label="Mã OTP xác nhận (Verification OTP)"
                onChange={(e) => { setOtpCode(e.target.value); }}
                placeholder="VD: 123456"
                required
                value={otpCode}
              />
            </div>
            <Button
              className="h-10 mb-0.5 text-[10px] font-bold uppercase tracking-widest"
              onClick={() => {
                setIsOtpSent(true);
                toast.success("Đã gửi mã OTP đến Số điện thoại của bạn!");
              }}
              type="button"
              variant="outline"
            >
              {isOtpSent ? "Gửi lại OTP" : "Lấy mã OTP"}
            </Button>
          </div>

          <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
            <Button onClick={() => { setIsSigningModalOpen(false); }} type="button" variant="outline">
              QUAY LẠI
            </Button>
            <Button onClick={handleCompleteSign} type="button" variant="secondary">
              HOÀN TẤT KÝ SỐ
            </Button>
          </div>
        </div>
      </Modal>
    </>
  );
}
