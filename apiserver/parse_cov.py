import sys
import re

with open(sys.argv[1], 'r') as f:
    content = f.read()

parts = content.split('<tr class=')
for part in parts:
    if 'employee_service.rs' in part or 'leave_service.rs' in part or '>Totals<' in part or '>TOTAL<' in part or '>Total<' in part:
        clean = re.sub(r'<[^>]+>', ' ', '<tr class=' + part).strip()
        print(clean)
