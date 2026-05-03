import json
import collections

with open('lint-results-internal.json', 'r') as f:
    data = json.load(f)
    
rule_counts = collections.Counter()
file_counts = collections.Counter()

total_errors = 0
total_warnings = 0

for file_data in data:
    errs = file_data.get('errorCount', 0)
    warns = file_data.get('warningCount', 0)
    if errs > 0 or warns > 0:
        file_counts[file_data['filePath']] += (errs + warns)
        total_errors += errs
        total_warnings += warns
        for msg in file_data.get('messages', []):
            rule = msg.get('ruleId', 'unknown')
            rule_counts[rule] += 1
            
print(f"Total Errors: {total_errors}, Total Warnings: {total_warnings}")
print("\nTop rules violated:")
for rule, count in rule_counts.most_common():
    print(f"  {rule}: {count}")

print("\nTop files by issues:")
for file, count in file_counts.most_common(15):
    print(f"  {file.split('/webdashboard/')[-1]}: {count}")

