import { useRouter } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import Button from "@/components/ui/button";
import { useTranslation } from "react-i18next";

export function RouteErrorFallback({ error }: { error: Error }) {
  const router = useRouter();
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center justify-center p-8 text-center bg-destructive/5 rounded-xl border border-destructive/20 m-4">
      <div className="h-12 w-12 rounded-full bg-destructive/10 flex items-center justify-center mb-4">
        <AlertTriangle className="h-6 w-6 text-destructive" />
      </div>
      <h2 className="text-lg font-semibold text-destructive mb-2">
        {t("common.error.unexpected", "An unexpected error occurred")}
      </h2>
      <p className="text-sm text-muted-foreground mb-6 max-w-md">
        {error.message || t("common.error.description", "We're sorry, but something went wrong while loading this page.")}
      </p>
      <div className="flex gap-4">
        <Button
          variant="outline"
          onClick={() => {
            router.invalidate();
          }}
        >
          {t("common.action.try_again", "Try Again")}
        </Button>
      </div>
    </div>
  );
}
