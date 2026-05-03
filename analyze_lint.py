import re
from collections import defaultdict

file_path = 'lint_current.txt'
rule_counts = defaultdict(int)
file_counts = defaultdict(int)
current_file = None

with open(file_path, 'r') as f:
    for line in f:
        line = line.strip()
        if not line: continue
        if line.startswith('/workspace/') or line.startswith('/home/'):
            current_file = line.split('/')[-1]
            continue
            
        # Match ESLint error/warning lines like:
        # 19:73  error  'variables...'  @typescript-eslint/no-base-to-string
        match = re.search(r'\s+(error|warning)\s+.*?\s+([a-zA-Z0-9_/@-]+)$', line)
        if match:
            rule = match.group(2)
            rule_counts[rule] += 1
            if current_file:
                file_counts[current_file] += 1

print("Top 10 Rules:")
for rule, count in sorted(rule_counts.items(), key=lambda x: x[1], reverse=True)[:10]:
    print(f"{rule}: {count}")

print("\nTop 10 Files with most errors:")
for file, count in sorted(file_counts.items(), key=lambda x: x[1], reverse=True)[:10]:
    print(f"{file}: {count}")
