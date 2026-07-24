#!/usr/bin/env python3
"""Count .unwrap() and .expect() calls in Rust production code.

Properly skips code inside #[cfg(test)] mod tests { ... } blocks
and files in test directories.
"""
import os, re, sys, json
from pathlib import Path

def strip_rust_comments_and_strings(text: str) -> str:
    """Replace string literals and comments with spaces to avoid false matches."""
    # Remove block comments /* ... */
    text = re.sub(r'/\*.*?\*/', ' ', text, flags=re.DOTALL)
    # Remove line comments // ...
    text = re.sub(r'//[^\n]*', ' ', text)
    # Remove raw string literals r"..." and r#"... "#
    text = re.sub(r'r#*"[^"]*"#*', '""', text)
    # Remove regular string literals "..." (simple heuristic)
    text = re.sub(r'"([^"\\]|\\.)*"', '""', text)
    return text

def extract_production_code(filepath: str) -> str:
    """Read a Rust file and remove all #[cfg(test)] blocks + content inside mod tests."""
    with open(filepath, 'r') as f:
        lines = f.readlines()

    result_lines = []
    in_test_mod = 0  # nesting depth of #[cfg(test)] mod tests { ... }
    cfg_test_depth = 0  # tracks nested #[cfg(test)] annotations
    brace_depth_inside_test = 0
    test_mod_started = False

    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Track #[cfg(test)] attribute — might be followed by mod tests {
        if re.match(r'#\[cfg\(test\)\]', stripped):
            # Skip blank/comment lines and look for mod tests {
            lookahead = i + 1
            while lookahead < len(lines) and (
                lines[lookahead].strip() == '' or
                lines[lookahead].strip().startswith('//') or
                lines[lookahead].strip().startswith('#[')
            ):
                lookahead += 1
            if lookahead < len(lines) and re.match(r'mod\s+tests\s*\{', lines[lookahead].strip()):
                in_test_mod += 1
                test_mod_started = True
                i = lookahead + 1
                brace_depth_inside_test = 1  # we're inside the { from mod tests {
                continue

        if test_mod_started and in_test_mod > 0:
            # Count braces to track nesting depth
            brace_depth_inside_test += stripped.count('{') - stripped.count('}')
            if brace_depth_inside_test <= 0:
                in_test_mod -= 1
                test_mod_started = (in_test_mod > 0)
                brace_depth_inside_test = 0
                i += 1
                continue
            i += 1
            continue

        result_lines.append(line)
        i += 1

    return ''.join(result_lines)

def count_patterns(filepath: str) -> tuple[int, int]:
    """Return (unwrap_count, expect_count) for production code in file."""
    code = extract_production_code(filepath)
    # Strip comments and strings to avoid false matches
    code = strip_rust_comments_and_strings(code)
    unwrap_count = len(re.findall(r'\.unwrap\(\)', code))
    expect_count = len(re.findall(r'\.expect\(', code))
    return unwrap_count, expect_count

def is_test_file(filepath: str) -> bool:
    """Check if the file is a test file."""
    name = Path(filepath).name
    path = filepath
    # Test directories
    if '/tests/' in path:
        return True
    # Test file naming conventions
    if name.endswith('_test.rs') or name.endswith('_tests.rs') or name == 'tests.rs' or name == 'tests_mod.rs':
        return True
    return False

def scan_directory(root_dir: str) -> dict:
    """Scan all .rs files and return counts per file."""
    results = {'unwrap': {}, 'expect': {}}
    for dirpath, dirnames, filenames in os.walk(root_dir):
        for fname in filenames:
            if not fname.endswith('.rs'):
                continue
            fullpath = os.path.join(dirpath, fname)
            relpath = os.path.relpath(fullpath, root_dir)

            if is_test_file(relpath):
                continue

            uw, ex = count_patterns(fullpath)
            if uw > 0:
                results['unwrap'][relpath] = uw
            if ex > 0:
                results['expect'][relpath] = ex

    return results

def main():
    if len(sys.argv) < 2:
        print("Usage: count_unwrap.py <apiserver_src_dir> [--json]", file=sys.stderr)
        sys.exit(1)

    src_dir = sys.argv[1]
    output_json = '--json' in sys.argv

    results = scan_directory(src_dir)

    if output_json:
        print(json.dumps(results, indent=2))
    else:
        print("# .unwrap() counts per production file")
        for fpath, count in sorted(results['unwrap'].items(), key=lambda x: -x[1]):
            print(f"{count:4d}  {fpath}")
        print(f"\n# Total .unwrap(): {sum(results['unwrap'].values())}")
        print(f"\n# .expect() counts per production file")
        for fpath, count in sorted(results['expect'].items(), key=lambda x: -x[1]):
            print(f"{count:4d}  {fpath}")
        print(f"\n# Total .expect(): {sum(results['expect'].values())}")

if __name__ == '__main__':
    main()
