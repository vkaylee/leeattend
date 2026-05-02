import { createFileRoute } from "@tanstack/react-router";

function PageSkeleton() {
  return (
    <div className="p-6 space-y-6 animate-pulse">
      <div className="h-8 w-48 bg-slate-200 dark:bg-slate-700 rounded-full" />
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {[1, 2, 3, 4].map((index) => (
          <div className="h-32 bg-slate-100 dark:bg-slate-800 rounded-[3rem]" key={index} />
        ))}
      </div>
      <div className="h-64 bg-slate-100 dark:bg-slate-800 rounded-[3rem]" />
    </div>
  );
}

export const Route = createFileRoute("/_authenticated/_workspace/")({
  pendingComponent: PageSkeleton,
});
