import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute(
  "/_authenticated/_organization/settings/storage"
)({
  // Tương tự các trang settings khác, component chính sẽ nằm bên trong `.lazy.tsx`
});
