import json
import collections
import os

def analyze_lint(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Find the start of the JSON array
    try:
        json_start = content.find('[')
        if json_start == -1:
            print("No JSON array found")
            return
        
        json_content = content[json_start:]
        data = json.loads(json_content)
    except Exception as e:
        print(f"Error parsing JSON: {e}")
        return

    rule_counts = collections.Counter()
    module_counts = collections.Counter()
    file_counts = collections.Counter()
    
    for file_data in data:
        path = file_data['filePath']
        # Extract module name (e.g., src/pages/attendance)
        parts = path.split('/webdashboard/src/')
        if len(parts) > 1:
            module = parts[1].split('/')[0]
            if module == 'pages':
                module = 'pages/' + parts[1].split('/')[1]
        else:
            module = 'root'
            
        messages = file_data.get('messages', [])
        for msg in messages:
            rule = msg.get('ruleId', 'unknown')
            rule_counts[rule] += 1
            module_counts[module] += 1
            file_counts[path] += 1
            
    print(f"Total Issues: {sum(rule_counts.values())}")
    print("\n--- Top Modules ---")
    for mod, count in module_counts.most_common(20):
        print(f"  {mod}: {count}")
        
    print("\n--- Top Rules ---")
    for rule, count in rule_counts.most_common(10):
        print(f"  {rule}: {count}")

    print("\n--- Top Files ---")
    for path, count in file_counts.most_common(10):
        print(f"  {path.split('/webdashboard/')[-1]}: {count}")

if __name__ == "__main__":
    analyze_lint('lint-results.json')
