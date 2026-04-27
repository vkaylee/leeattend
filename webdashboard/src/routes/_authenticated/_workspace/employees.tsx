import { createFileRoute } from "@tanstack/react-router";

function PageSkeleton() {
  return (
    <div className="p-6 space-y-6 animate-pulse">
      <div className="flex items-center justify-between">
        <div className="h-8 w-40 bg-slate-200 dark:bg-slate-700 rounded-full" />
        <div className="h-10 w-32 bg-slate-200 dark:bg-slate-700 rounded-lg" />
      </div>
      <div className="h-12 bg-slate-100 dark:bg-slate-800 rounded-xl" />
      <div className="space-y-3">
        {[1, 2, 3, 4, 5].map((i) => (
          <div key={i} className="h-16 bg-slate-50 dark:bg-slate-800/50 rounded-xl" />
        ))}
      </div>
    </div>
  );
}

export const Route = createFileRoute("/_authenticated/_workspace/employees")({
  pendingComponent: PageSkeleton,
});
