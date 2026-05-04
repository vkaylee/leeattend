import re
from pathlib import Path


def parse_logs() -> None:
    total_tests = 0
    passed_tests = 0

    nextest_pattern = re.compile(r"Summary \[.*?\] (\d+) tests? run: (\d+) passed")
    vitest_pattern = re.compile(r"Tests\s+(\d+)\s+passed\s+\((\d+)\)")
    pw_pattern = re.compile(r"^\s*(\d+)\s+passed\s+\(.*?\)", re.MULTILINE)

    log_dir = Path(".test_logs")
    for log_file in log_dir.glob("*.log"):
        # print(f"Checking {log_file}")
        content = log_file.read_text(errors="replace")
        ansi_escape = re.compile(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])")
        clean_content = ansi_escape.sub("", content)

        file_total = 0
        file_passed = 0

        # nextest
        matches = list(nextest_pattern.finditer(clean_content))
        if matches:
            last_match = matches[-1]
            file_total = int(last_match.group(1))
            file_passed = int(last_match.group(2))

        # vitest
        matches = list(vitest_pattern.finditer(clean_content))
        if matches:
            last_match = matches[-1]
            file_passed = int(last_match.group(1))
            file_total = int(last_match.group(2))

        # playwright
        if "playwright" in log_file.name.lower():
            matches = list(pw_pattern.finditer(clean_content))
            if matches:
                last_match = matches[-1]
                passed = int(last_match.group(1))
                file_passed = passed
                file_total = passed

        if file_total > 0:
            print(f"{log_file.name}: {file_passed}/{file_total}")
            total_tests += file_total
            passed_tests += file_passed

    print(f"Total: {passed_tests}/{total_tests}")


if __name__ == "__main__":
    parse_logs()
