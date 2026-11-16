import { useSearch } from "@tanstack/react-router";
import React from "react";

import AssignmentRulesPage from "./assignment-rules-page";
import WorkingPatternsPage from "./working-patterns-page";

export default function ShiftsRulesPage() {
  // Safe search param parsing
  const search = useSearch({ strict: false });
  const activeTab = search.tab ?? "patterns";

  

  return (
    <div className="space-y-6">
      <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        {activeTab === "patterns" ? (
          <WorkingPatternsPage hideHeaderTitle />
        ) : (
          <AssignmentRulesPage hideHeaderTitle />
        )}
      </div>
    </div>
  );
}
