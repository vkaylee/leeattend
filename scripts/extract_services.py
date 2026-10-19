import os, re

root = 'apiserver/src'
svc_types = []       # (file, line, name)
repo_fields = []     # (file, line, owner, field, trait)
tx_calls = []        # (file, line, text)

field_re = re.compile(r'^\s*(?:pub(?:\(crate\))?\s+)?([a-z_0-9]+)\s*:\s*(?:Arc<)?\s*dyn\s+([A-Za-z0-9_:]+)')
tx_re = re.compile(r'(\.transaction\(|TransactionConn|\.begin\(\)|\.commit\(\)|\.rollback\(\)|pools\.get|pool\.get|\bbegin\b|\bcommit\b|\brollback\b)')

for root_dir, dirs, files in os.walk(os.path.join(root,'services')):
    for f in sorted(files):
        if not f.endswith('.rs'): continue
        if f.endswith('_test.rs') or f.endswith('_tests.rs') or 'tests' in f: continue
        p = os.path.join(root_dir, f)
        rel = os.path.relpath(p, root)
        try: lines = open(p, errors='ignore').read().split('\n')
        except: continue
        cur_struct = None
        for i, ln in enumerate(lines, 1):
            m = re.match(r'^pub struct ([A-Za-z0-9_]+)', ln)
            if m:
                name = m.group(1)
                if re.search(r'(Service|Helper|Pipeline|Provisioner|Buffer|Writer|Calculator|Matcher|Coordinator|Evaluator|Resolver|Dispatcher|Storage)', name):
                    svc_types.append((rel, i, name))
                cur_struct = name
            m2 = re.match(r'^pub trait ([A-Za-z0-9_]+)', ln)
            if m2:
                svc_types.append((rel, i, 'trait '+m2.group(1)))
            fm = field_re.match(ln)
            if fm and cur_struct:
                repo_fields.append((rel, i, cur_struct, fm.group(1), fm.group(2)))
            if tx_re.search(ln):
                tx_calls.append((rel, i, ln.strip()[:150]))

print("=== SERVICE-LIKE PUB TYPES ===")
for x in svc_types: print(f"{x[0]}:{x[1]}\t{x[2]}")
print()
print("=== REPO FIELDS IN SERVICES ===")
for x in repo_fields: print(f"{x[0]}:{x[1]}\t{x[2]}.{x[3]}: dyn {x[4]}")
print()
print("=== TX CALLS IN SERVICES ===")
for x in tx_calls: print(f"{x[0]}:{x[1]}\t{x[2]}")
