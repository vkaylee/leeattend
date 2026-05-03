import sys

with open('webdashboard/src/pages/leave-requests-page.tsx', 'r') as f:
    lines = f.readlines()

imports = [
    'import { LeaveImportBanner } from "@/components/features/leave/leave-import-banner";\n',
    'import { CreateLeaveRequestModal } from "@/components/features/leave/create-leave-request-modal";\n',
    'import { LeaveRequestRow } from "@/components/features/leave/leave-request-row";\n'
]

# Find the end of imports (first empty line or import statement block end)
lines = lines[:35] + imports + lines[35:38] + lines[366:]

with open('webdashboard/src/pages/leave-requests-page.tsx', 'w') as f:
    f.writelines(lines)
print("Done")
