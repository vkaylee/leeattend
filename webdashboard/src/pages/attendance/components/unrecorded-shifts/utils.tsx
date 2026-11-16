import {
  AlertTriangle,
  Check,
  Clock,
} from "lucide-react";

import type { ZonedNoShowRow } from "../../hooks/use-no-show-rows";


/** Decorative zone icon — aria-hidden, label lives on the parent cell. */
export function ZoneIcon({ zone }: { zone: ZonedNoShowRow["zone"] }) {
  if (zone === "b") return <AlertTriangle aria-hidden="true" className="size-3.5" />;
  if (zone === "a") return <Clock aria-hidden="true" className="size-3.5" />;
  return <Check aria-hidden="true" className="size-3.5" />;
}
