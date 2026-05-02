# ruff: noqa: E501
from pathlib import Path

GUARD_TEMPLATE = """# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\\033[0m"
    echo -e "👉 This script is an internal module. Use: \\033[1m./test.sh\\033[0m"
    exit 1
fi
"""


def apply_guards() -> None:
    scripts = list(Path("scripts").glob("_*.sh"))
    # Exclude public tool wrappers and the already-updated bootstrap
    exclude = [
        Path("scripts/_cargo.sh"),
        Path("scripts/_npm.sh"),
        Path("scripts/_sqlx.sh"),
        Path("scripts/_bootstrap.sh"),
    ]

    count = 0
    for script_path in scripts:
        if script_path in exclude:
            continue

        content = script_path.read_text(encoding="utf-8")

        if "BASH_SOURCE" in content:
            print(f"Skipping {script_path} (already has guard)")
            continue

        lines = content.splitlines()
        if lines and lines[0].startswith("#!"):
            lines.insert(1, "\n" + GUARD_TEMPLATE)
        else:
            lines.insert(0, GUARD_TEMPLATE)

        script_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"✅ Applied guard to {script_path}")
        count += 1

    print(f"\n✨ Successfully protected {count} internal modules.")


if __name__ == "__main__":
    apply_guards()
