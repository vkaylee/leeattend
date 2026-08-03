# 🔧 Lint Gate Fix — Trả lại sức mạnh thực thi cho cấu hình deny

> **Generated:** 2026-08-01 · **Scope:** leedevkit upstream (tool ngoài) — fix qua PR, **KHÔNG sửa local `.leedevkit/`**
> **Method:** Evidence-based — mọi kết luận từ log thực tế `.test_logs/` và đọc source `.leedevkit/scripts/`.
> **Status:** ✅ **PR opened** — https://github.com/vkaylee/leedevkit/pull/4 (đang chờ review/merge)
> **Rules đã load:** `coding-standards.md`, `testing-standards.md`, `execution-wrappers.md`, `change-management.md`.

---

## ⚠️ Quan trọng: tại sao KHÔNG sửa `.leedevkit/` local

`.leedevkit/` nằm trong `.gitignore:88` của leeattend — là **tool ngoài** (orchestrator per-project install).
Mọi sửa đổi ở đó sẽ bị ghi đè khi leedevkit update, và CI dùng bản release mới chứ không phải bản local.
⇒ Fix phải đi qua **upstream** `vkaylee/leedevkit`, chờ merge + release, rồi `leedevkit update` tại leeattend.

---

## 0. Executive Summary

Cấu hình deny nghiêm ngặt nhất dự án (`Cargo.toml` workspace lints: `unwrap_used`, `expect_used`,
`panic`, `todo`, `disallowed_*` = `deny`) **không bao giờ được thực thi**. Ba cơ chế hỏng độc lập
tạo nên "false green": (1) `&&` short-circuit khiến clippy không chạy, (2) Linting phase không được
phép `sys.exit(1)`, (3) summary chỉ đọc test logs, không đọc clippy exit code.

Hệ quả: **2038 `.unwrap()`, 497 `.expect()`, 31 `panic!`, 366 `todo!`** tồn tại trong `apiserver/src`
trong khi hệ thống báo "All selected tests passed successfully!" — vi phạm trực tiếp Big Five #3.

**Ước lượng:** 1-2 giờ (3 file tooling, không đụng source apiserver). Sau đó build sẽ RED — đúng ý định.

---

## 1. Bằng chứng (đã verify)

| # | Cơ chế hỏng | Bằng chứng |
|---|---|---|
| 1a | **Clippy không chạy** — `cargo fmt --check && cargo clippy` (short-circuit) | `_test_modules.py:104`; `.test_logs/Linting_rust-backend-clippy.log` (22:58 hôm nay) chỉ có fmt diff, **0 dòng clippy output** (`grep -cE "Checking|Finished|error"` = 0) |
| 1b | **Linting fail không exit** — `if phase_name != "Linting": sys.exit(1)` | `_test_handler.py:246` — riêng Linting được miễn trừ |
| 1c | **Summary bỏ qua clippy exit code** — chỉ parse nextest/vitest/playwright patterns | `_test_handler.py:301-355` `print_test_summary()` |

Nợ thật đang bị che giấu (đã đếm bằng script brace-depth, loại test modules):
- `compliance_validation_service.rs` (service **hot nhất**, fan_in 927): **6 `expect()` production** (dòng 282-570)
- Tổng production: ≥ 2 unwrap, ≥ 9 expect, ≥ 1 panic (heuristic — con số thật cao hơn vì scanner không hoàn hảo)

---

## 2. Remediation — 3 thay đổi (đã làm trong PR #4 upstream)

### Fix 1: Tách fmt và clippy thành 2 task độc lập (`scripts/_test_modules.py`)
```python
# TRƯỚC: một task, `&&` làm clippy chết khi fmt fail
backend_cmd = build_compose_exec(rust_svc, f"cargo fmt --all {fmt_flag} && cargo clippy {pkg_flag} -- -D warnings", ...)

# SAU: 2 task riêng biệt, chạy song song, fail độc lập
tasks.append(("rust-backend-fmt", rust_svc,
    build_compose_exec(rust_svc, f"cargo fmt --all {fmt_flag}", workdir="/workspace", mode=mode)))
tasks.append(("rust-backend-clippy", rust_svc,
    build_compose_exec(rust_svc, f"cargo clippy {pkg_flag} -- -D warnings", workdir="/workspace", mode=mode)))
```
Giữ host pre-check (`rust-backend-fmt-check`) — phản hồi nhanh cho AI agents.

### Fix 2: Linting fail phải `sys.exit(1)` (`scripts/_test_handler.py`)
```python
# TRƯỚC
if phase_name != "Linting":
    sys.exit(1)
# SAU — bỏ ngoại lệ, lint là hard gate như mọi phase khác
sys.exit(1)
```

### Fix 3: Summary phản ánh failures thật (`scripts/_test_handler.py` `print_test_summary`)
Đầu hàm, đọc `self._results`; nếu có phase `status == "fail"`, in FAILED và không in "All selected tests passed successfully!".

### Test đi kèm
- `test_test_modules.py`: `test_lint_all`/`test_lint_apiserver` assert fmt + clippy là 2 lệnh riêng (không `&&`).
- `test_handlers.py`: 4 test mới (summary fail/pass, lint fail exit non-zero, lint pass không exit) + cập nhật `test_handle_test_coverage`.
- **Kết quả:** 79/79 test 2 file pass. 11 fail trong `test_skills_manager.py`/`test_usage.py` là pre-existing trên `main` (thiếu psutil/docker, không liên quan).

---

## 3. Validation

1. `python3 -c "import ast; ast.parse(...)"` cho `_test_modules.py`, `_test_handler.py`, và 2 file test → hết syntax error.
2. `python3 -m pytest scripts/tests/test_test_modules.py scripts/tests/test_handlers.py -q` → **79 passed**.
3. Toàn bộ `scripts/tests/` (trừ `test_safe_run.py` thiếu psutil) → **657 passed**, 11 failed pre-existing giống hệt trên `main`.
4. PR #4: https://github.com/vkaylee/leedevkit/pull/4 — `+156/−7`.

## 4. Hệ quả dự kiến & bước tiếp theo

- **Sau khi PR #4 merge + release:** tại leeattend chạy `leedevkit update` → `./leedevkit test api --lint-only` sẽ **RED** trên code hiện tại (2038 unwrap production + pedantic/nursery chưa từng bị kiểm) — đúng ý định của gate.
- **Đừng** sửa hết 2038 trong một PR. Trình tự:
  1. PR này (gate sống): chờ upstream merge + release + update local. RED là expected.
  2. PR tiếp: `#[allow(clippy::unwrap_used)]` scoped cho các module đã biết, kèm `// SAFETY:`/`// TODO: replace with AppError` — để build XANH trở lại mà vẫn chặn được code mới.
  3. Từng module: thay unwrap → `?` + `AppError` (bắt đầu từ `compliance_validation_service.rs`).
- **Blocker:** PR #4 chờ approve/merge của owner leedevkit. Trước khi merge, gate local vẫn mù — nhưng giờ đã có bằng chứng + PR mở, không còn "false sense of safety" thầm lặng.

---

## 5. Tuân thủ rules

- **R1 (Big Five #3):** gate deny được thực thi đúng như cam kết cấu hình.
- **R2 (Execution Wrappers):** không gọi cargo trực tiếp trong quá trình sửa; validation qua `./leedevkit test`.
- **R3 (Change Management):** thay đổi tooling nhỏ, 2 file, có review; không ảnh hưởng source.
- **R4 (Boy Scout):** không đụng gì khác ngoài 2 file tooling.
