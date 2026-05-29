import sys
lf_total = 0
lh_total = 0
with open('.test_logs/coverage-apiserver.lcov', 'r') as f:
    for line in f:
        if line.startswith('LF:'):
            lf_total += int(line.strip().split(':')[1])
        elif line.startswith('LH:'):
            lh_total += int(line.strip().split(':')[1])
print(f"TOTAL: LF={lf_total}, LH={lh_total}, Coverage={lh_total/lf_total if lf_total else 0:.2%}")
