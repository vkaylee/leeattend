# SOLID / DRY / KISS — Code Review Report

**Project:** LeeAttend API Server (Rust / Axum)  
**Date:** 2026-03-08  
**Scope:** `apiserver/src/` — repositories, services, handlers, models, domain

---

## Tổng quan

LeeAttend tuân thủ khá tốt SOLID/DRY/KISS. Kiến trúc layered (domain → repo trait → service → handler) rõ ràng. Dưới đây là đánh giá chi tiết.

---

## ✅ ĐIỂM TỐT

### 1. **Dependency Inversion (SOLID-D) — Tuyệt vời**

```
Trait: EmployeeRepository / PayrollRepository / ...
  └─ Impl: SqlxEmployeeRepository / SqlxPayrollRepository / ...
```

- Tất cả 14+ repository đều có **trait abstraction** + Sqlx implementation.
- `PoolBacked` supertrait bắt buộc compile-time rằng mọi repo phải nhận `PgPool`.
- Services inject `Arc<dyn XxxRepository>` — dễ mock, dễ test.

### 2. **Single Source of Truth (DRY) — `attendance_utils.rs`**

Hàm `determine_daily_status()` là SSoT cho status attendance. Được document rõ:
```
/// This is the **single source of truth** for status determination, used by:
/// - report_service.rs
/// - attendance.rs (worker batch)
```

### 3. **Error Handling (KISS) — `AppError` enum**

`errors.rs` thiết kế gọn: 9 variant, `From` impls tự động, `IntoResponse` hide errors nội bộ. Có unit test đầy đủ.

### 4. **Phased Decomposition — `PayrollService`**

```
fetch_context() → build_segments() → calculate_monthly_payslip() → persist_calculated_payslip()
```
Tách rõ 4 phases, dùng `PayrollContext` struct thay vì god function. `SegmentSnapshot` named struct thay tuple 10 phần tử.

### 5. **Domain Logic tách biệt — `domain/permissions.rs`**

60+ permissions, hierarchical role inheritance, pure functions (`base_permissions`, `role_hierarchy`, `role_permissions`). Unit test đầy đủ: roundtrip, hierarchy, cross-role.

### 6. **Extractors pattern — `handlers/common.rs`**

`WorkspaceId`, `MemberContext`, `EmployeeContext` — tái sử dụng qua tất cả handlers. `require_permission()` / `require_scoped_permission()` nhất quán.

### 7. **Unit tests coverage**

Nhiều service/domain có test inline: `roster_repository` (10+ tests), `payroll_service` (6 tests), `leave_accrual_service` (12+ tests), `attendance_utils` (15+ tests).

---

## ⚠️ VẤN ĐỀ CẦN CẢI THIỆN

### 1. **`AppState` God Object — Vi phạm SRP (SOLID-S)**

> [!WARNING]
> `AppState` struct có **40+ fields** (18 services + 14 repos + misc). Đây là Single Responsibility vi phạm rõ nhất.

[lib.rs:59-106](file:///home/elt1541/lee/leeattend/apiserver/src/lib.rs#L59-L106)

```rust
pub struct AppState {
    // 18 services, 14 repos, redis, cache, jwt_secret...
    // 40+ fields total
}
```

**Khuyến nghị:** Nhóm theo domain module:

```rust
pub struct AppState {
    pub employee: EmployeeModule,   // employee_repo + employee_service
    pub payroll: PayrollModule,     // payroll_repo + payroll_service
    pub attendance: AttendanceModule,
    pub auth: AuthModule,
    pub infra: InfraModule,         // db, redis, cache, jwt
}
```

**Mức độ:** 🔴 Cao — Ảnh hưởng maintainability khi thêm feature mới.

---

### 2. **Monolithic `create_app()` — Vi phạm SRP + KISS**

[lib.rs:182-822](file:///home/elt1541/lee/leeattend/apiserver/src/lib.rs#L182-L822)

Hàm `create_app()` dài **640+ dòng**, define tất cả routes. Khó navigate, dễ conflict khi merge.

**Khuyến nghị:** Tách thành module routers:

```rust
pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::dashboard(state.clone()))
        .merge(routes::auth(state.clone()))
        .merge(routes::employees(state.clone()))
        .merge(routes::payroll(state.clone()))
        // ...
        .layer(cors)
}
```

**Mức độ:** 🟡 Trung bình — Không ảnh hưởng runtime, nhưng ảnh hưởng DX.

---

### 3. **`main.rs` Manual DI Wiring — Vi phạm DRY**

[main.rs:126-364](file:///home/elt1541/lee/leeattend/apiserver/src/main.rs#L126-L364)

200+ dòng tạo repo → wrap Arc → inject vào service → wrap Arc. Pattern lặp lại:

```rust
let xxx_repo = Arc::new(SqlxXxxRepository::new(pool.clone()));
let xxx_service = Arc::new(XxxService::new(xxx_repo.clone(), pool.clone()));
```

**Khuyến nghị:** Tạo factory hoặc builder:

```rust
impl AppState {
    pub async fn from_config(config: &AppConfig) -> Arc<Self> {
        let repos = Repositories::new(&config.db);
        let services = Services::new(&repos);
        // ...
    }
}
```

**Mức độ:** 🟡 Trung bình — Boilerplate nhiều nhưng explicit.

---

### 4. **Mixed Error Types trong Services — Vi phạm consistency (KISS)**

Phần lớn service methods return `Result<T, StatusCode>`:

```rust
// employee_service.rs, payroll_service.rs, etc.
fn create_employee(...) -> Result<Employee, StatusCode>
```

Trong khi handlers và domain dùng `AppResult<T>` (`Result<T, AppError>`).

**Vấn đề:** Service layer coupling với HTTP concerns. Phải convert `StatusCode` → `AppError` ở handler boundary qua `From<(StatusCode, String)>`.

**Khuyến nghị:** Services nên return `AppResult<T>` hoặc domain-specific error:

```rust
fn create_employee(...) -> AppResult<Employee>  // hoặc Result<Employee, DomainError>
```

**Mức độ:** 🟡 Trung bình — Hoạt động nhưng vi phạm layer separation.

---

### 5. **Redundant `new()` + `from_pool()` trên Repositories**

Mỗi Sqlx repo có cả `new(pool)` và `from_pool(pool)` (từ `PoolBacked` trait). Hai phương thức làm cùng một việc.

```rust
impl SqlxEmployeeRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }  // ← manual
}
impl PoolBacked for SqlxEmployeeRepository {
    fn from_pool(pool: PgPool) -> Self { Self { pool } }  // ← trait
}
```

**Khuyến nghị:** Loại bỏ `new()`, dùng `from_pool()` thống nhất. Hoặc `new()` gọi `from_pool()`.

**Mức độ:** 🟢 Thấp — Nhưng vi phạm DRY thuần túy.

---

### 6. **Wildcard Re-exports `pub use xxx::*` — Vi phạm KISS**

[handlers/mod.rs](file:///home/elt1541/lee/leeattend/apiserver/src/handlers/mod.rs) và [models/mod.rs](file:///home/elt1541/lee/leeattend/apiserver/src/models/mod.rs) dùng `pub use xxx::*` globbing.

**Vấn đề:** Khi thêm public item vào sub-module, nó tự động export ra ngoài — có thể gây name collision hoặc leak API không mong muốn.

**Khuyến nghị:** Explicit re-exports cho handlers. Models có thể giữ `*` vì backward compat nhưng nên document rõ.

**Mức độ:** 🟢 Thấp — Chỉ xảy ra vấn đề khi scale.

---

### 7. **Thiếu Interface Segregation cho lớn repos (SOLID-I)**

`EmployeeRepository` trait có **35+ methods** (1413 dòng file). Bao gồm CRUD, working conditions, employment history, recruitment, department history, payroll config, turnover analytics.

**Khuyến nghị:** Tách thành nhỏ hơn:

```rust
trait EmployeeCrudRepository: PoolBacked { ... }
trait EmployeeWorkConditionRepository: PoolBacked { ... }
trait EmployeeRecruitmentRepository: PoolBacked { ... }
trait EmployeeAnalyticsRepository: PoolBacked { ... }
```

**Mức độ:** 🟡 Trung bình — Trait quá lớn gây khó mock partial behavior.

---

## 📊 Bảng tổng hợp

| Nguyên tắc | Đánh giá | Ghi chú |
|-----------|---------|---------|
| **S** — Single Responsibility | ⚠️ 7/10 | `AppState` god object, `create_app()` monolithic |
| **O** — Open/Closed | ✅ 9/10 | Trait-based repos, permission extension |
| **L** — Liskov Substitution | ✅ 9/10 | Proper trait impls, no violations detected |
| **I** — Interface Segregation | ⚠️ 6/10 | `EmployeeRepository` 35+ methods, `PayrollRepository` 20+ |
| **D** — Dependency Inversion | ✅ 9/10 | `Arc<dyn Trait>` everywhere, `PoolBacked` supertrait |
| **DRY** | ⚠️ 7/10 | `attendance_utils` SSoT tốt, nhưng `new()`/`from_pool()` trùng, main.rs boilerplate |
| **KISS** | ⚠️ 7/10 | Mixed error types, monolithic routing, wildcard exports |

**Overall: 7.7/10** — Kiến trúc solid, cần cải thiện tổ chức code (grouping, routing, error consistency).

---

## 🎯 Ưu tiên hành động

| Ưu tiên | Hành động | Effort |
|---------|----------|--------|
| **P1** | Tách `create_app()` thành module routers | 2-3h |
| **P2** | Unify error type: services dùng `AppResult` thay `StatusCode` | 4-6h |
| **P3** | Tách `AppState` thành domain modules | 3-4h |
| **P3** | Tách `EmployeeRepository` trait | 2-3h |
| **P4** | Remove `new()` redundancy ở repos | 1h |
| **P4** | Explicit re-exports thay `pub use *` | 1h |
