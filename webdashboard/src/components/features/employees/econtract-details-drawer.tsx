import { X, KeyRound, CheckCircle, Clock, Send, Download, FileSignature } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

// @see TD-1 — eContracts backend not yet wired; UI is stub-only.
import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { EContractEnvelope } from "@/lib/api/econtracts";
import { cn } from "@/lib/utils";

import EContractSigningModal from "./econtract-signing-modal";

interface EContractDetailsDrawerProps {
  envelope: EContractEnvelope | null;
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
}

  // eslint-disable-next-line max-lines-per-function
export default function EContractDetailsDrawer({
  envelope,
  isOpen,
  onClose,
  wsId: _wsId,
}: EContractDetailsDrawerProps) {
  const { t } = useTranslation("employees");
  const toast = useToast();

  const [isSigningModalOpen, setIsSigningModalOpen] = useState(false);

  if (!envelope) return null;

  const handleResendNotification = () => {
    toast.success(t("econtract_drawer.resend_success"));
  };

  const handleDownloadPDF = () => {
    // Generate dummy download
    const link = document.createElement("a");
    link.href = "data:application/pdf;base64,JVBERi0xLjQKJd...[MOCK SIGNED PDF]...";
    link.download = `Signed_Contract_${envelope.external_envelope_id}.pdf`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    toast.success(t("econtract_drawer.download_success"));
  };

  const handleCompleteSign = (_otpCode: string, _signatureText: string) => {
    // e-signature backend is not yet wired — signing is a no-op for now.
    setIsSigningModalOpen(false);
    toast.success(t("econtract_drawer.sign_success"));
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
                {t("econtract_drawer.title")}
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
              <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest block mb-0.5">{t("econtract_drawer.provider")}</span>
              <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wider">
                {envelope.provider_type}
              </span>
            </div>
            <div>
              <span className="text-[9px] font-bold text-slate-400 uppercase tracking-widest block mb-0.5">{t("econtract_drawer.transaction_status")}</span>
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
            <Typography className="text-[10px] font-bold text-slate-400 uppercase tracking-[0.2em]" variant="h3">{t("econtract_drawer.signing_order")}</Typography>
            <div className="relative pl-6 space-y-6 before:absolute before:left-[10px] before:top-2 before:bottom-2 before:w-0.5 before:bg-slate-100 dark:before:bg-slate-800">
              {envelope.signer_statuses.map((signer) => {
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
                          <span className="text-[9px] text-slate-400 font-semibold block uppercase tracking-wider mt-0.5 animate-none">
                            {t("econtract_drawer.role_label")}: {signer.role === "representative" ? t("econtract_drawer.role_rep") : t("econtract_drawer.role_emp")} • {t("econtract_drawer.routing_order")}: {signer.routing_order}
                          </span>
                        </div>
                        <span
                          className={cn(
                            "text-[8px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border",
                            isSigned
                              ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-355"
                              : "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950/40 dark:text-amber-355"
                          )}
                        >
                          {signer.status}
                        </span>
                      </div>
                      {!!signer.signed_at && (
                        <div className="mt-2 text-[8px] font-mono text-slate-400 block uppercase tracking-wider">
                          {t("econtract_drawer.signed_at")}: {new Date(signer.signed_at).toLocaleString()}
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
              {t("econtract_drawer.resend_notif")}
            </Button>

            {import.meta.env.DEV && envelope.status === "completed" ? (
              <Button
                className="flex items-center gap-1 text-[11px] font-bold uppercase tracking-widest h-11"
                icon={<Download size={14} />}
                onClick={handleDownloadPDF}
                variant="outline"
              >
                {t("econtract_drawer.download_pdf")}
              </Button>
            ) : null}
            {isEmployeePending ? (
              <Button
                className="flex items-center gap-1 text-[11px] font-bold uppercase tracking-widest h-11 bg-primary-600 hover:bg-primary-700 text-white"
                icon={<FileSignature size={14} />}
                onClick={() => { setIsSigningModalOpen(true); }}
                variant="secondary"
              >
                {t("econtract_drawer.sign_now")}
              </Button>
            ) : null}
          </div>
        </div>
      </Sheet>

      <EContractSigningModal
        envelopeId={envelope.external_envelope_id}
        isOpen={isSigningModalOpen}
        onClose={() => { setIsSigningModalOpen(false); }}
        onCompleteSign={handleCompleteSign}
      />
    </>
  );
}
