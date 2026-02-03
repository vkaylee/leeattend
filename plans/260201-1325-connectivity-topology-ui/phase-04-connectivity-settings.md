---
phase: 4
title: "ConnectivitySettings Integration"
status: pending
effort: 0.5h
---

# Phase 4: ConnectivitySettings Integration

## Context Links

- Current: `/home/elt1541/leeattend/webdashboard/src/pages/settings/ConnectivitySettings.tsx`
- New component: `@/components/features/connectivity/ConnectivityTopology`

## Overview

**Priority:** High
**Status:** Pending

Replace the current tab-based layout with the new unified topology view.

## Current State (ConnectivitySettings.tsx)

```typescript
// Current: 74 lines, tab switcher
export default function ConnectivitySettings() {
  const [activeTab, setActiveTab] = useState<TabType>("agents");
  // ...
  return (
    <div>
      {/* Header */}
      {/* Tab switcher */}
      {activeTab === "agents" && <AgentsPage />}
      {activeTab === "devices" && <DevicesPage />}
    </div>
  );
}
```

## Target State

```typescript
// After: Simplified, single topology view
export default function ConnectivitySettings() {
  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      {/* Header - keep existing */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <Share2 size={24} />
          </div>
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-700 leading-tight">
              {t("common.settings.categories.connectivity")}
            </h1>
            <p className="text-slate-500 mt-1 font-medium">
              {t("common.settings.categories.connectivity_desc")}
            </p>
          </div>
        </div>
      </div>

      {/* New: Unified topology view */}
      <ConnectivityTopology />
    </div>
  );
}
```

## Related Code Files

### Modify
- `/home/elt1541/leeattend/webdashboard/src/pages/settings/ConnectivitySettings.tsx`

### Remove (deprecate)
These files become unused but keep for now (can clean up later):
- `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx`
- `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx`

### Import
- `@/components/features/connectivity/ConnectivityTopology`

## Implementation Steps

1. Update imports:
   ```typescript
   // Remove
   - import AgentsPage from "@/pages/AgentsPage";
   - import DevicesPage from "@/pages/DevicesPage";
   - import { useState } from "react";
   - import { HardDrive } from "lucide-react";

   // Add
   + import ConnectivityTopology from "@/components/features/connectivity/ConnectivityTopology";
   ```

2. Simplify component:
   ```typescript
   import { useTranslation } from "react-i18next";
   import { Share2 } from "lucide-react";
   import ConnectivityTopology from "@/components/features/connectivity/ConnectivityTopology";

   export default function ConnectivitySettings() {
     const { t } = useTranslation();

     return (
       <div className="space-y-8 animate-in fade-in duration-500">
         {/* Header */}
         <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
           <div className="flex items-center gap-4">
             <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
               <Share2 size={24} />
             </div>
             <div>
               <h1 className="text-3xl font-bold tracking-tight text-slate-700 leading-tight">
                 {t("common.settings.categories.connectivity")}
               </h1>
               <p className="text-slate-500 mt-1 font-medium">
                 {t("common.settings.categories.connectivity_desc")}
               </p>
             </div>
           </div>
         </div>

         {/* Topology view */}
         <div className="animate-in slide-in-from-bottom-4 duration-500">
           <ConnectivityTopology />
         </div>
       </div>
     );
   }
   ```

3. Result: ~30 lines (down from 74)

## Todo List

- [ ] Update ConnectivitySettings imports
- [ ] Remove tab state and switcher UI
- [ ] Add ConnectivityTopology component
- [ ] Verify header styling preserved
- [ ] Test page loads correctly

## Success Criteria

- [ ] Page renders without errors
- [ ] Header matches previous design
- [ ] ConnectivityTopology visible and functional
- [ ] No console errors

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Breaking change | Keep old pages as backup |
| Missing translations | Verify i18n keys exist |

## Security Considerations

None - UI refactor only.

## Next Steps

After Phase 4, proceed to Phase 5: Testing and polish.
