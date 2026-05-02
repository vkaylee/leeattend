#!/bin/bash
# system_blocker.sh
# ==============================================================================
# PHYSICAL BARRIER AGAINST LAZY AI
# Load this file into the AI's IDE environment via BASH_ENV.
# ==============================================================================

# Ghi đè lệnh python3
python3() {
    # Check if arguments contain "-c"
    for arg in "$@"; do
        if [[ "$arg" == "-c" ]]; then
            echo -e "\n\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\033[0m" >&2
            echo -e "\033[1;31m🚨 SYSTEM ERROR: ENVIRONMENT HAS LOCKED INLINE PYTHON COMMANDS!\033[0m" >&2
            echo -e "You (AI) just attempted to bypass GEMINI.md and use 'python3 -c'." >&2
            echo -e "The operating system has denied your execution request." >&2
            echo -e "\033[1;33m👉 PLEASE CREATE A PHYSICAL SCRIPT FILE AND RUN VIA _safe_run.py\033[0m" >&2
            echo -e "\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\033[0m" >&2
            return 127
        fi
    done
    # If valid, pass control to real python3
    command python3 "$@"
}

# Ghi đè lệnh python (đề phòng)
python() {
    for arg in "$@"; do
        if [[ "$arg" == "-c" ]]; then
            echo -e "\n\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\033[0m" >&2
            echo -e "\033[1;31m🚨 SYSTEM ERROR: ENVIRONMENT HAS LOCKED INLINE PYTHON COMMANDS!\033[0m" >&2
            echo -e "You (AI) just attempted to bypass GEMINI.md and use 'python -c'." >&2
            echo -e "The operating system has denied your execution request." >&2
            echo -e "\033[1;33m👉 PLEASE CREATE A PHYSICAL SCRIPT FILE AND RUN VIA _safe_run.py\033[0m" >&2
            echo -e "\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\033[0m" >&2
            return 127
        fi
    done
    command python "$@"
}

# Khóa luôn quyền bỏ qua alias/function
alias command='command '
