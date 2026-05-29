import sys

def parse():
    with open('.test_logs/coverage-apiserver.lcov', 'r') as f:
        file_name = None
        lf = 0
        lh = 0
        for line in f:
            if line.startswith('SF:'):
                file_name = line.strip().split(':')[1]
            elif line.startswith('LF:'):
                lf = int(line.strip().split(':')[1])
            elif line.startswith('LH:'):
                lh = int(line.strip().split(':')[1])
            elif line.startswith('end_of_record'):
                if 'employee_service.rs' in file_name or 'leave_service.rs' in file_name:
                    print(f'{file_name}: LF={lf}, LH={lh}, Cov={lh/lf if lf else 0:.2%}')

if __name__ == '__main__':
    parse()
