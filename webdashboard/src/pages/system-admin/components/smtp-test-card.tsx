import { Send, CheckCircle, XCircle, Loader2, AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface SmtpTestCardProps {
  onTest: () => void;
  onTestEmailChange: (email: string) => void;
  testEmail: string;
  testing: boolean;
  testResult: { success: boolean; message: string } | null;
  verifiedAt: Date | null;
}

export function SmtpTestCard({
  testEmail,
  testing,
  testResult,
  verifiedAt,
  onTestEmailChange,
  onTest,
}: SmtpTestCardProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl p-6 space-y-4 shadow-sm">
      <Typography className="font-semibold text-slate-700 dark:text-slate-200" variant="h3">
        {t("smtp.test_section", "Test Email Delivery")}
      </Typography>
      {!verifiedAt ? (
        <div className="flex items-start gap-2 p-3 rounded-xl bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800 text-amber-700 dark:text-amber-300 text-sm">
          <AlertTriangle className="w-4 h-4 mt-0.5 flex-shrink-0" />
          <span>{t("smtp.untested_reminder", "You haven't tested this configuration yet. We strongly recommend sending a test email before relying on this for password resets.")}</span>
        </div>
      ) : null}
      <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
        {t("smtp.test_desc", "Send a test email to verify your SMTP configuration works.")}
      </Typography>

      <div className="flex gap-3 items-end">
        <div className="flex-1 space-y-1.5">
          <label className="text-sm font-medium text-slate-600 dark:text-slate-400">
            {t("smtp.test_email_label", "Send test to")}
          </label>
          <input
            className="block w-full px-4 py-2.5 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-400 transition-all text-sm placeholder:text-slate-300 dark:placeholder:text-slate-600"
            data-testid="smtp-test-email-input"
            onChange={(e) => { onTestEmailChange(e.target.value); }}
            placeholder="admin@company.com"
            type="email"
            value={testEmail}
          />
        </div>
        <Button
          className="rounded-xl px-4 py-2 text-xs font-semibold normal-case tracking-normal shadow-xs"
          data-testid="smtp-test-btn"
          disabled={!testEmail.trim() || testing}
          loading={testing}
          onClick={onTest}
          variant="outline"
        >
          <Send className="w-4 h-4 mr-1.5" />
          {t("smtp.test_btn", "Test Now")}
        </Button>
      </div>

      {testing ? (
        <div className="flex items-center gap-2 text-sm text-slate-500">
          <Loader2 className="w-4 h-4 animate-spin" />
          {t("smtp.testing", "Sending test email...")}
        </div>
      ) : testResult ? (
        <div
          className={`flex items-start gap-2 p-4 rounded-xl text-sm font-medium animate-in fade-in slide-in-from-top-2 duration-200 ${
            testResult.success
              ? "bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300"
              : "bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300"
          }`}
          data-testid="smtp-test-result"
        >
          {testResult.success ? (
            <CheckCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
          ) : (
            <XCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
          )}
          <span className="whitespace-pre-wrap">{testResult.message}</span>
        </div>
      ) : null}
    </div>
  );
}
