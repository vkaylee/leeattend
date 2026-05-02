import os
import glob
import sys

GUARD_TEMPLATE = '''# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\\033[0m"
    echo -e "👉 This script is an internal module. Use: \\033[1m./test.sh\\033[0m"
    exit 1
fi
'''

def apply_guards():
    scripts = glob.glob('scripts/_*.sh')
    # Exclude public tool wrappers and the already-updated bootstrap
    exclude = ['scripts/_cargo.sh', 'scripts/_npm.sh', 'scripts/_sqlx.sh', 'scripts/_bootstrap.sh']
    
    count = 0
    for script in scripts:
        if script in exclude:
            continue
            
        with open(script, 'r') as f:
            content = f.read()
        
        if 'BASH_SOURCE' in content:
            print(f"Skipping {script} (already has guard)")
            continue
            
        lines = content.splitlines()
        if lines and lines[0].startswith('#!'):
            lines.insert(1, '\n' + GUARD_TEMPLATE)
        else:
            lines.insert(0, GUARD_TEMPLATE)
            
        with open(script, 'w') as f:
            f.write('\n'.join(lines) + '\n')
        print(f"✅ Applied guard to {script}")
        count += 1
    
    print(f"\n✨ Successfully protected {count} internal modules.")

if __name__ == "__main__":
    apply_guards()
