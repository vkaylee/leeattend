import { ShieldAlert, Sparkles } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

// @see TD-1 — eContracts backend not yet wired; signing is stub-only.
import Button from "@/components/ui/button";
import FormField from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";

interface EContractSigningModalProps {
  envelopeId: string;
  isOpen: boolean;
  onClose: () => void;
  onCompleteSign: (otpCode: string, signatureText: string) => void;
}

export default function EContractSigningModal({
  envelopeId,
  isOpen,
  onClose,
  onCompleteSign,
}: EContractSigningModalProps) {
  const { t } = useTranslation("employees");
  const toast = useToast();
  const [signatureText, setSignatureText] = useState("");
  const [otpCode, setOtpCode] = useState("");
  const [isOtpSent, setIsOtpSent] = useState(false);

  const handleCompleteSign = () => {
    if (!otpCode || otpCode.length < 4) {
      toast.error(t("econtract_drawer.error_otp"));
      return;
    }
    onCompleteSign(otpCode, signatureText);
  };

  return (
    <Modal
      icon={<Sparkles size={24} />}
      isOpen={isOpen}
      onClose={onClose}
      subtitle={`${t("econtract_drawer.session_for")} ${envelopeId}`}
      title={t("econtract_drawer.modal_title")}
    >
      <div className="space-y-6 pt-4">
        <div className="p-4 bg-amber-50/50 dark:bg-amber-950/20 border border-amber-100 dark:border-amber-900 rounded-2xl flex items-start gap-3">
          <ShieldAlert className="text-amber-500 shrink-0 mt-0.5" size={16} />
          <div>
            <Typography className="text-xs font-bold text-amber-800 dark:text-amber-400 uppercase tracking-wider" variant="h4">
              {t("econtract_drawer.mfa_title")}
            </Typography>
            <Typography className="text-[10px] text-amber-600 dark:text-amber-500 font-semibold uppercase tracking-wider leading-relaxed mt-0.5" variant="p">
              {t("econtract_drawer.mfa_desc")}
            </Typography>
          </div>
        </div>

        <div className="space-y-2">
          <label className="text-[11px] font-bold text-slate-500 uppercase tracking-widest" htmlFor="signature-input">
            {t("econtract_drawer.signature_label")}
          </label>
          <div className="h-32 border-2 border-dashed border-slate-200 dark:border-slate-800 rounded-3xl bg-slate-50/50 dark:bg-slate-900/50 flex flex-col items-center justify-center p-4 relative group">
            <input
              className="w-full text-center bg-transparent border-none text-2xl font-semibold italic text-primary-600 dark:text-primary-400 focus:outline-none placeholder:text-slate-300"
              id="signature-input"
              onChange={(e) => { setSignatureText(e.target.value); }}
              placeholder={t("econtract_drawer.signature_placeholder")}
              type="text"
              value={signatureText}
            />
            <span className="absolute bottom-2 text-[9px] text-slate-400 font-bold uppercase tracking-widest opacity-60">
              {t("econtract_drawer.signature_mock")}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-3 gap-4 items-end">
          <div className="col-span-2">
            <FormField
              id="otp-code-input"
              label={t("econtract_drawer.otp_label")}
              onChange={(e) => { setOtpCode(e.target.value); }}
              placeholder={t("econtract_drawer.otp_placeholder")}
              required
              value={otpCode}
            />
          </div>
          <Button
            className="h-10 mb-0.5 text-[10px] font-bold uppercase tracking-widest"
            onClick={() => {
              setIsOtpSent(true);
              toast.success(t("econtract_drawer.otp_sent_success"));
            }}
            type="button"
            variant="outline"
          >
            {isOtpSent ? t("econtract_drawer.resend_otp") : t("econtract_drawer.get_otp")}
          </Button>
        </div>

        <div className="flex items-center justify-end gap-3 pt-6 border-t border-slate-100 dark:border-slate-800">
          <Button onClick={onClose} type="button" variant="outline">
            {t("econtract_drawer.back_btn")}
          </Button>
          <Button onClick={handleCompleteSign} type="button" variant="secondary">
            {t("econtract_drawer.finish_btn")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
