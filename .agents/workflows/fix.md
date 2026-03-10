---
description: Workflow hoàn chỉnh cho việc sửa code — format, lint, chạy test sau mỗi lần thay đổi
---

# /fix — Code Fix Workflow

Quy trình này áp dụng cho **mọi code change** trong dự án LeeAttend.

## Bước 1: Phân tích phạm vi thay đổi

Xác định component nào bị thay đổi:
- `apiserver/` và/hoặc `agent-main/` → scope là `--api`
- `webdashboard/` → scope là `--web`
- Cả hai / tái cấu trúc lớn → scope là `--all`

## Bước 2: Format code

Chạy formatter theo component:
- **Rust**: `./run-tests.sh --docker=dev exec apiserver cargo fmt`
- **Web**: `./run-tests.sh --docker=dev exec webdashboard npm run format` (nếu có script)

## Bước 2.5: Viết / cập nhật test

Trước khi chạy test, AI **PHẢI** viết test mới hoặc cập nhật test hiện có:

| Loại thay đổi | Hành động bắt buộc |
|---|---|
| Thêm function/method mới | Viết ≥1 unit test happy path + 1 edge case |
| Fix bug | Viết regression test tái hiện bug đó |
| Thêm/sửa API endpoint | Viết/cập nhật integration test |
| Thêm React component | Viết Vitest unit test |
| Sửa UI flow | Cập nhật hoặc thêm Playwright E2E test |
| Sửa legacy code không có test | Viết test cho đoạn code VỪA SỬA (không cần cover toàn bộ file) |

> Nếu việc viết test là không thể (ví dụ: pure config, infra glue), phải giải thích rõ lý do trong summary.

## Bước 3: Chạy test smoke để verify nhanh

// turbo
Chạy smoke test cho đúng scope đã xác định ở Bước 1:
```bash
# API scope
./run-tests.sh --api --smoke

# Web scope
./run-tests.sh --web --smoke

# Full scope
./run-tests.sh --all --smoke
```

## Bước 4: Xử lý kết quả

**Nếu smoke PASS:**
- Báo cáo kết quả cho user
- Task hoàn thành ✅

**Nếu smoke FAIL:**
// turbo
- Xác định test nào fail: `./run-tests.sh --logs`
- Chạy lại đúng test đó để debug: `./run-tests.sh --api --test-only --test=TÊN_TEST`
- Sửa lỗi → quay lại Bước 3
- **KHÔNG** báo cáo hoàn thành cho đến khi tests pass

## ⚠️ Quy tắc bất di bất dịch

1. **KHÔNG bao giờ kết thúc task khi tests vẫn fail**
2. **KHÔNG chờ user yêu cầu** — chạy test là bắt buộc sau mỗi lần sửa code
3. **LUÔN báo cáo** test nào đã chạy và kết quả ra sao trong summary cuối
