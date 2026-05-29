import sys

def find_uncovered_lines(file_path, target_file):
    with open(file_path, 'r') as f:
        in_target = False
        uncovered = []
        for line in f:
            if line.startswith('SF:'):
                in_target = target_file in line
            elif in_target and line.startswith('DA:'):
                parts = line.strip()[3:].split(',')
                if len(parts) == 2 and parts[1] == '0':
                    uncovered.append(int(parts[0]))
        
        if uncovered:
            # group sequential lines
            groups = []
            start = uncovered[0]
            prev = start
            for i in range(1, len(uncovered)):
                if uncovered[i] == prev + 1:
                    prev = uncovered[i]
                else:
                    groups.append(f"{start}-{prev}" if start != prev else str(start))
                    start = uncovered[i]
                    prev = start
            groups.append(f"{start}-{prev}" if start != prev else str(start))
            print(f"Uncovered lines for {target_file}: {', '.join(groups)}")
        else:
            print(f"No uncovered lines for {target_file}")

if __name__ == '__main__':
    find_uncovered_lines('.test_logs/coverage-apiserver.lcov', 'leave_service.rs')
