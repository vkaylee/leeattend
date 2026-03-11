# Kế hoạch Refactor và Tối ưu hóa APIServer (Architecture Optimization Plan)

Tài liệu này định nghĩa chi tiết từng phase (giai đoạn) để tối ưu hóa kiến trúc hiện tại của `apiserver`, giúp giải quyết triệt để các vấn đề về "God-files" (File vạn năng), rò rỉ (leak) logic Database Transaction, và tối ưu hóa Dependency Injection trong Axum.

---

## Giai đoạn 1 (Phase 1): Tách các "God-files" (Ưu tiên Cao nhất)
*Mục tiêu: Áp dụng Single Responsibility Principle (SRP) để giảm độ phức tạp (Cognitive Load) và giảm khả năng merge conflict.*

### 1.1. Tối ưu `employee_repository.rs` (45 KB)
**Context:** Chứa tất cả logic từ CRUD cơ bản, đến việc tìm kiếm phức tạp (Search, Filter), Onboarding, chuyển đổi trạng thái nhân viên (Status changes).
**Kế hoạch thực hiện:**
- Tạo thư mục con: `src/repositories/employee/`.
- Tách `EmployeeRepository` (trait tổng) thành các trait/struct nhỏ hơn:
  1. `EmployeeCoreRepository`: Chỉ quản lý thao tác CRUD cơ bản cho entity `Employee`.
  2. `EmployeeOnboardingRepository`: Xử lý nghiệp vụ onboarding, tạo password, mã giới thiệu.
  3. `EmployeeQueryRepository`: Định nghĩa các queries phức tạp như tìm kiếm nâng cao (Full-text search, Pagination, Filters).
- Các Service hiện tại dùng `EmployeeRepository` sẽ chỉ inject đúng nhóm Trait cần thiết (hoặc tạo một Facade).

### 1.2. Tối ưu `roster_repository.rs` (41 KB)
**Context:** File chứa logic truy vấn lịch làm việc, kiểm tra xung đột ca làm, thao tác bản nháp (draft) và bản chính thức (published).
**Kế hoạch thực hiện:**
- Áp dụng một phần tư tưởng CQRS (Command Query Responsibility Segregation).
- Tách làm hai nhánh:
  1. `RosterQueryRepository`: Chuyên trách đọc dữ liệu, report, render lịch.
  2. `RosterCommandRepository`: Quản lý logic Insert, Update, Validate (xung đột ca, overwrite draft).

### 1.3. Tối ưu `leave_accrual_service.rs` (35 KB)
**Context:** Dịch vụ tích lũy/tính phép đang chứa vô số logic if/else và switch-case cho từng loại phép do quy định của nhiều bộ phận (Department) hay loại hợp đồng.
**Kế hoạch thực hiện:**
- Tách bằng **Strategy Pattern**.
- Tạo thư mục: `src/services/leave_accrual/strategies/`.
- Định nghĩa trait `AccrualStrategy`. Thay vì check điều kiện dài dòng, service sẽ inject đúng loại `Strategy` tính toán (ví dụ: `SeniorityAccrualStrategy`, `ProbationAccrualStrategy`) cho từng Employee.

### 1.4. Tối ưu `excel_writer.rs` (41 KB)
**Context:** Ghi export ra hàng chục loại file Excel khác nhau (Bảng công, Bảng lương, Báo cáo ngày).
**Kế hoạch thực hiện:**
- Phân rã thành bộ `ReportGenerator` (vd: `payroll_report.rs`, `attendance_report.rs`).

---

## Giai đoạn 2 (Phase 2): Quản lý Transaction qua Unit of Work
*Mục tiêu: Đóng gói toàn bộ Database logic khỏi Service layer.*

**Context:** Các file như `payroll_service.rs` và `attendance_import_service.rs` hiện đang nhận trực tiếp connection pool (`pool.begin()`) để tạo transaction. Việc này phá vỡ ranh giới của Layered Architecture khi Service phải "biết" mình đang chạy trên SQLx hay Postgres.

**Kế hoạch thực hiện:**
1. Tạo module mới `src/repositories/uow.rs`.
2. Định nghĩa trait `UnitOfWork` với các method trả về (return) các thể hiện (instance) của Repository hiện thân bên trong transaction đó.
   ```rust
   #[async_trait]
   pub trait UnitOfWork: Send + Sync {
       async fn begin(&mut self) -> Result<(), Error>;
       async fn commit(self) -> Result<(), Error>;
       async fn rollback(self) -> Result<(), Error>;

       // Accessors
       fn employees(&mut self) -> &mut dyn EmployeeRepository;
       fn leaves(&mut self) -> &mut dyn LeaveRequestRepository;
   }
   ```
3. (Tác động): Refactor lại các Service cần write ở nhiều bảng vào một lúc. Thay vì truyền `db_pool`, ta inject `Arc<dyn UnitOfWorkFactory>`. Service sẽ yêu cầu Factory tạo Transaction (UoW), thực hiện logic, và gọi `uow.commit()`.

---

## Giai đoạn 3 (Phase 3): Tối ưu State Management (Axum)
*Mục tiêu: Sạch hóa các file Router Handler, tăng tính an toàn kiểu (Type Safety) và phạm vi nhỏ gọn cho Dependencies.*

**Context:** `AppState` lưu trữ mọi thứ. Tham số handler hiện tại: `State(state): State<Arc<AppState>>`. Dù hàm đó chỉ cần `LeaveState`.
**Kế hoạch thực hiện:**
1. Trong `src/state.rs`, triển khai trait `FromRef` cho các State phụ:
   ```rust
   impl FromRef<Arc<AppState>> for Arc<EmployeeState> {
       fn from_ref(app_state: &Arc<AppState>) -> Self {
           Arc::new(app_state.employees.clone())
       }
   }
   ```
2. Cập nhật các Handlers trong thư mục `src/handlers/`. Chuyển signature từ:
   `pub async fn create_employee(State(state): State<Arc<AppState>>, Json(payload): Json<...>)`
   thành:
   `pub async fn create_employee(State(emp_state): State<Arc<EmployeeState>>, Json(payload): Json<...>)`
3. Điều này yêu cầu Service phải `Clone` cheap (bằng `Arc`), hiện tại `AppState` đã dùng `Arc`, nên refactor này sẽ rất mượt (Smooth).

---

## Quy trình Thực thi Đề xuất (Execution Flow)
Với tư cách là Orchestrator, tôi khuyến nghị thực thi theo đúng thứ tự (để hạn chế lỗi build kẹt cứng):
- **Bước 1:** Xử lý Giai đoạn 3 (Axum State) trước, vì nó dễ, ít rủi ro và cấu trúc lại mã router sạch sẽ ngay tức khắc.
- **Bước 2:** Xử lý Giai đoạn 1 (Tách `employee_repository.rs` và `roster_repository.rs`). Chạy lại Unit Test toàn bộ Repositories sau bước này.
- **Bước 3:** Tách `leave_accrual_service.rs` (Sẽ cần QA/cẩn thận vì liên quan đến logic phép năm tài chính).
- **Bước 4:** Xây dựng `UnitOfWork` pattern (Mất thời gian nhất vì phải đào qua nhiều service phức tạp).
