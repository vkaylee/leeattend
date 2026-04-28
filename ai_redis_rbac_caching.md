---
title: "Redis RBAC Caching Implementation Plan"
status: "COMPLETED"
created: "2026-04-28"
completed: "2026-04-28"
---

# Architecture & Implementation Plan: Redis RBAC Caching

## 1. Mục tiêu (Objective)
Tối ưu hóa việc kiểm tra phân quyền (Dynamic RBAC) trên mỗi request. Hiện tại, `MemberContext::from_request_parts` (nằm tại `apiserver/src/handlers/common.rs`) đang gọi từ 4-6 queries tới DB (Postgres) cho **mỗi request**, gây ra bottleneck (thắt cổ chai) nghiêm trọng.

Quyết định kiến trúc: Sử dụng **Distributed Cache (Redis)** để lưu trữ danh sách permissions của user.

## 2. Thiết kế Kiến trúc (Architecture Design)
- **Data Structure**: Lưu cache ở dạng JSON serializable struct.
  - Key format: `cache:auth:permissions:{workspace_id}:{user_id}`
  - Value format (JSON): Chứa danh sách `roles`, `permissions` (dạng mảng string), `managed_department_ids`, và `is_global_manager`.
- **TTL (Time to Live)**: 1 giờ (3600 giây).
- **Invalidation Strategy**: 
  - Khi Admin cập nhật Role của User, thay đổi Permission của Role, hoặc gán Extra Permissions, hệ thống sẽ xóa (DEL) key này khỏi Redis.
  - Request tiếp theo sẽ gặp "Cache Miss" và tự động query lại DB để nạp quyền mới vào Redis.

## 3. Các bước triển khai (Implementation Steps)

### ✅ Phase 1: Chuẩn bị Models
- Tạo struct `CachedMemberContext` trong `apiserver/src/handlers/common.rs` có implement `Serialize`, `Deserialize`.
- Đảm bảo Enum `Permission` có thể chuyển đổi qua lại giữa `String` dễ dàng.

### ✅ Phase 2: Cập nhật `MemberContext::from_request_parts`
- Chỉnh sửa `apiserver/src/handlers/common.rs`.
- Triển khai inline cache-aside pattern: `redis_cache.get` → fallback `fetch_member_context_from_db` → `redis_cache.set`.
- Xử lý an toàn `Option<RedisPool>` (graceful fallback khi Redis không khả dụng).

### ✅ Phase 3: Triển khai Cache Invalidation (Thu hồi quyền Real-time)
Đã thêm logic `redis_cache.delete` / `redis_cache.delete_pattern` tại:
- `apiserver/src/handlers/workspaces/members.rs`: `add_member`, `update_member_role`, `remove_member`
- `apiserver/src/handlers/workspaces/roles.rs`: `assign_extra_role`, `revoke_extra_role`, `delete_workspace_role`, `update_role_permissions`
- `apiserver/src/handlers/workspaces/permissions.rs`: `assign_extra_permission`, `revoke_extra_permission`
- Thêm method `delete_pattern` vào `RedisCache` để hỗ trợ xóa hàng loạt khi thay đổi cấp Role (ảnh hưởng nhiều user).

### ✅ Phase 4: Testing & Verification
**Unit Tests** (7 tests trong `apiserver/src/cache/redis_cache.rs`):
- `test_rbac_permission_cache_key_format`
- `test_rbac_permission_cache_key_uniqueness`
- `test_rbac_invalidation_pattern_format`
- `test_rbac_ttl_is_one_hour`
- `test_cached_member_context_serialization_roundtrip`
- `test_cached_member_context_global_manager_none_departments`
- `test_cached_member_context_empty_permissions`

**Integration Tests** (6 tests trong `apiserver/tests/integration/rbac/cache.rs`):
- `test_cache_aside_repeated_requests_succeed`
- `test_cache_invalidation_on_member_role_update`
- `test_cache_invalidation_on_extra_permission_grant`
- `test_cache_invalidation_on_extra_permission_revoke`
- `test_cache_invalidation_on_role_permissions_update`
- `test_cache_invalidation_on_member_removal`

**Bonus Fix**: Sửa lỗi `setup-test-env.sh` không source được function `leeattend_run_integration` — nguyên nhân do `test-runner.sh` truyền argument dạng single-string thay vì separate args.

---
**Status: ✅ COMPLETED — All phases implemented and verified.**
