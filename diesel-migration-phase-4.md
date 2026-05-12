# Rule Compliance Check - Diesel Migration Phase 4

## Task: Phase 4: "The Long Tail" Migration & Integration
**Task Slug:** `diesel-migration-phase-4`

## Mandatory Rule IDs:
1. **[L0-CORE-1]** Enterprise Premium Standard: Đảm bảo tất cả các repo mới đều có xử lý lỗi qua `AppError` và tuân thủ pattern Hybrid.
2. **[PTY-SAFETY-1]** Hermetic Toolbox: Luôn sử dụng `./scripts/_cargo.sh`, `./scripts/_npm.sh`, và `./test.sh` thay vì lệnh trực tiếp trên host.
3. **[PTY-SAFETY-3]** Git Protocol: Luôn sử dụng `./scripts/git.sh` cho các thao tác git.
4. **[L0-CORE-4]** Boy Scout Rule: Dọn dẹp code cũ, sửa typo và cải thiện indentation (mẫu 3 dòng) khi đi qua các file liên quan.

## Strategic Questions Compliance:
- [x] Đã đặt 3 câu hỏi chiến lược trước khi thực hiện migration phức tạp.

## Implementation Notes:
- Ưu tiên tính nguyên tử (atomic commits).
- Không sử dụng `todo!()` hoặc `panic!()` trong code production.
- Đảm bảo strict indentation cho repository initialization để tránh Clippy lint.
