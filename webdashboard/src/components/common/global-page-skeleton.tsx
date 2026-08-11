import { PageContainer } from "@/components/ui/page-container";
import { Skeleton } from "@/components/ui/skeleton";

export function GlobalPageSkeleton() {
  return (
    <PageContainer className="duration-300">
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-48 rounded-full" />
        <Skeleton className="h-10 w-32 rounded-lg" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {[1, 2, 3, 4].map((index) => (
          <Skeleton className="h-32 rounded-[2rem]" key={`card-${index}`} />
        ))}
      </div>
      <Skeleton className="h-[400px] w-full rounded-[2rem]" />
    </PageContainer>
  );
}
