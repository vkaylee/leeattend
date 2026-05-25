import { createFileRoute } from "@tanstack/react-router";
import { Skeleton } from "@/components/ui/skeleton";
import { RouteErrorFallback } from "@/components/common/route-error-fallback";

function PageSkeleton() {
  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-40 rounded-full" />
        <Skeleton className="h-10 w-32 rounded-lg" />
      </div>
      <Skeleton className="h-12 rounded-xl" />
      <div className="space-y-3">
        {[1, 2, 3, 4, 5].map((index) => (
          <Skeleton className="h-16 rounded-xl" key={index} />
        ))}
      </div>
    </div>
  );
}

export const Route = createFileRoute("/_authenticated/_workspace/employees")({
  pendingComponent: PageSkeleton,
  errorComponent: RouteErrorFallback,
});
