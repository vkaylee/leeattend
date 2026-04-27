---
title: "Redis RBAC Caching Implementation Plan"
status: "Planned"
created: "2026-04-28"
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

### Phase 1: Chuẩn bị Models
- Tạo struct `CachedMemberContext` trong `apiserver/src/handlers/common.rs` có implement `Serialize`, `Deserialize`. Struct này sẽ chứa các field đã được rút gọn từ các truy vấn DB.
- Đảm bảo Enum `Permission` có thể chuyển đổi qua lại giữa `String` dễ dàng.

### Phase 2: Cập nhật `MemberContext::from_request_parts`
- Chỉnh sửa `apiserver/src/handlers/common.rs`.
- Khởi tạo RedisCache client từ `app_state.infra.redis.clone()`.
- Bọc toàn bộ 6 DB queries hiện tại vào một hàm `fetch_fn`.
- Sử dụng phương thức `redis_cache.get_or_fetch(key, ttl, fetch_fn).await` (đã có sẵn ở `apiserver/src/cache/redis_cache.rs`).
- Sau khi lấy được `CachedMemberContext` từ Redis/DB, map nó về struct `MemberContext` chuẩn.

### Phase 3: Triển khai Cache Invalidation (Thu hồi quyền Real-time)
- Tìm tất cả các endpoints cập nhật quyền (ví dụ trong `apiserver/src/handlers/workspaces/roles.rs`, `members.rs`, `permissions.rs`).
- Khi có thay đổi: Gọi lệnh `redis_cache.delete(&format!("cache:auth:permissions:{workspace_id}:{user_id}")).await`.
- Nếu thay đổi cấp Role (ảnh hưởng nhiều user): Có thể cần quét và xóa nhiều keys (dùng Redis `SCAN` + `DEL`).

### Phase 4: Testing & Verification
- Đảm bảo các Integration Tests (đặc biệt là tests về RBAC, permission matrix) vẫn chạy pass (Dùng lệnh `./run-tests.sh --api --smoke </dev/null`).

## 4. Các câu hỏi mở (Open Questions)
- Cần user xác nhận về việc xử lý Invalidation khi cập nhật quyền của Role (ảnh hưởng tất cả member của Role đó). Xóa từng key hay áp dụng global cache invalidation flag?

---
**Chờ xác nhận từ User để chuyển sang trạng thái IMPLEMENTATION.**
