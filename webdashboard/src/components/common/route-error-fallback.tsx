import { useRouter } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

export function RouteErrorFallback({ error }: { error: Error }) {
  const router = useRouter();
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center justify-center p-8 text-center bg-destructive/5 rounded-xl border border-destructive/20 m-4">
      <div className="h-12 w-12 rounded-full bg-destructive/10 flex items-center justify-center mb-4">
        <AlertTriangle className="h-6 w-6 text-destructive" />
      </div>
      <h2 className="text-lg font-semibold text-destructive mb-2">
        {t("common.error_occurred", "An error occurred")}
      </h2>
      <p className="text-sm text-muted-foreground mb-6 max-w-md">
        {error.message || t("common.system_connection_error", "System connection error")}
      </p>
      <div className="flex gap-4">
        <Button
          onClick={() => {
            void router.invalidate();
          }}
          variant="outline"
        >
          {t("common.retry", "Retry")}
        </Button>
      </div>
    </div>
  );
}
