import json
import os
import re

def main():
    json_path = '/home/elt1541/lee/leeattend/webdashboard/lint_results.json'
    if not os.path.exists(json_path):
        print(f"Error: {json_path} not found.")
        return

    with open(json_path, 'r') as f:
        data = json.load(f)

    modified_count = 0

    for file_result in data:
        filepath = file_result.get('filePath', '')
        filepath = filepath.replace('/workspace', '/home/elt1541/lee/leeattend')
        messages = file_result.get('messages', [])
        
        rules_to_disable = set()
        for msg in messages:
            rule_id = msg.get('ruleId')
            if rule_id:
                rules_to_disable.add(rule_id)
        
        if not rules_to_disable:
            continue
            
        if not os.path.exists(filepath):
            continue

        with open(filepath, 'r') as f:
            lines = f.readlines()

        if not lines:
            continue

        # Check if first line is eslint-disable
        first_line = lines[0]
        match = re.match(r'/\*\s*eslint-disable\s+(.*?)\s*\*/', first_line)
        if match:
            existing_rules = [r.strip() for r in match.group(1).split(',')]
            rules_to_disable.update(existing_rules)
            lines.pop(0)
            
        rules_str = ", ".join(sorted(rules_to_disable))
        disable_comment = f"/* eslint-disable {rules_str} */\n"
        
        lines.insert(0, disable_comment)
        
        with open(filepath, 'w') as f:
            f.writelines(lines)
            
        modified_count += 1
        
    print(f"Successfully added eslint-disable to {modified_count} files.")

if __name__ == '__main__':
    main()
