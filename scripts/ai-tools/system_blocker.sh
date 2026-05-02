#!/bin/bash
# system_blocker.sh
# ==============================================================================
# HÀNG RÀO VẬT LÝ CHỐNG AI LƯỜI BIẾNG
# Nạp file này vào môi trường IDE của AI qua biến BASH_ENV.
# ==============================================================================

# Ghi đè lệnh python3
python3() {
    # Kiểm tra xem tham số có chứa "-c" không
    for arg in "$@"; do
        if [[ "$arg" == "-c" ]]; then
            echo -e "\n\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\033[0m" >&2
            echo -e "\033[1;31m🚨 LỖI HỆ THỐNG: MÔI TRƯỜNG ĐÃ KHÓA LỆNH PYTHON INLINE!\033[0m" >&2
            echo -e "Bạn (AI) vừa cố tình bỏ qua GEMINI.md và sử dụng 'python3 -c'." >&2
            echo -e "Hệ điều hành đã từ chối quyền thực thi của bạn." >&2
            echo -e "\033[1;33m👉 HÃY TẠO FILE SCRIPT VẬT LÝ VÀ CHẠY QUA _safe_run.py\033[0m" >&2
            echo -e "\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\033[0m" >&2
            return 127
        fi
    done
    # Nếu hợp lệ, chuyển quyền cho python3 thật
    command python3 "$@"
}

# Ghi đè lệnh python (đề phòng)
python() {
    for arg in "$@"; do
        if [[ "$arg" == "-c" ]]; then
            echo -e "\n\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\033[0m" >&2
            echo -e "\033[1;31m🚨 LỖI HỆ THỐNG: MÔI TRƯỜNG ĐÃ KHÓA LỆNH PYTHON INLINE!\033[0m" >&2
            echo -e "Bạn (AI) vừa cố tình bỏ qua GEMINI.md và sử dụng 'python -c'." >&2
            echo -e "Hệ điều hành đã từ chối quyền thực thi của bạn." >&2
            echo -e "\033[1;33m👉 HÃY TẠO FILE SCRIPT VẬT LÝ VÀ CHẠY QUA _safe_run.py\033[0m" >&2
            echo -e "\033[0;31m!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\033[0m" >&2
            return 127
        fi
    done
    command python "$@"
}

# Khóa luôn quyền bỏ qua alias/function
alias command='command '
