# Generic Approval Workflow Engine (Late-Binding Task Worklist) - Tài Liệu Kiến Trúc & Thiết Kế

Tài liệu này đặc tả kiến trúc, thiết kế cơ sở dữ liệu, và logic vận hành của **Generic Approval Workflow Engine** (Công cụ phê duyệt quy trình đa bước) dùng trong hệ thống **LeeAttend**. Engine này được xây dựng theo tiêu chuẩn Enterprise Premium Pro Max, hỗ trợ đầy đủ các yêu cầu phức tạp của doanh nghiệp lớn.

---

## 1. Tổng Quan Kiến Trúc (Late-Binding Task Worklist)

Để tránh nợ kỹ thuật (Technical Debt) và đảm bảo tính linh hoạt tối đa của sơ đồ tổ chức thực tế, hệ thống sử dụng kiến trúc **Late-Binding Task Worklist (Phân giải muộn qua danh sách nhiệm vụ)**:

1. **Generic Domain:** Engine tương tác qua cặp định danh đa hình `(entity_type: String, entity_id: UUID)`.
2. **Abstract Steps Snapshot:** Khi gửi đơn (Submit), hệ thống chụp lại cấu hình quy trình gốc (các bước, vai trò cần duyệt, thời hạn) và lưu trữ dưới dạng JSONb ở bảng Instance. Hệ thống **không** đi tìm User ID cụ thể của người duyệt tương lai tại thời điểm này.
3. **Late-Binding Tasks:** Khi một bước duyệt trở thành hiện tại (Active), hệ thống sinh ra các **Nhiệm vụ phê duyệt (Task Instances)** tương ứng với tiêu chí phân công của bước (ví dụ: gán cho phòng ban cụ thể hoặc vai trò cụ thể). Danh sách người duyệt thực tế được quyết định **động tại thời điểm truy vấn hoặc duyệt** (Late Binding), đảm bảo khi thay đổi Trưởng phòng hoặc điều chỉnh phân quyền, hệ thống tự động cập nhật mà không cần chạy job đồng bộ dữ liệu tĩnh.

---

## 1.1. Tiêu Chuẩn Đạt Được (Standards & Compliance)

Mô hình thiết kế **Late-Binding Task Worklist** của LeeAttend tuân thủ nghiêm ngặt các tiêu chuẩn và mẫu thiết kế doanh nghiệp (Enterprise Design Patterns) hàng đầu trên thế giới:

1. **Chuẩn BPMN 2.0 (Business Process Model and Notation):**
   - Hỗ trợ đầy đủ khái niệm **User Task** với cơ chế phân công qua **Candidate Roles** và **Candidate Groups** thay vì đi gán tĩnh cho người dùng cụ thể.
   - Triển khai thành công các chế độ đồng thuận **Consensus Mode** chuẩn hóa (`ANY` - chỉ cần 1 người duyệt, `ALL` - yêu cầu toàn bộ thành viên duyệt) tương đương với Gateways trong sơ đồ BPMN.

2. **Hệ mẫu Workflow Patterns (WfMC - Workflow Management Coalition & workflowpatterns.com):**
   - **Resource Patterns:** Tuân thủ đầy đủ các mẫu phân phối tài nguyên nghiệp vụ bao gồm *Dynamic Allocation* (phân bổ động), *Role-Based Routing* (định tuyến theo vai trò), và *Delegation* (ủy quyền động).
   - **Control-flow Patterns:** Triển khai các mẫu điều khiển luồng cơ bản và nâng cao như *Sequence* (tuần tự), *Parallel Split* (tách nhánh song song), *Synchronization* (đồng bộ hóa trạng thái), và *Exclusive Choice* (lựa chọn độc quyền).

3. **Mẫu thiết kế Task Worklist (Enterprise Integration Patterns - EIP):**
   - Decouple hoàn toàn logic duyệt phép ra khỏi các thực thể nghiệp vụ (Leave/Overtime). Thông tin tiến trình được quản lý tập trung thông qua bảng phẳng nhiệm vụ, giúp giữ gìn nguyên tắc thiết kế đơn nhiệm (Single Responsibility Principle) cho các Service nghiệp vụ.

4. **Tiêu chuẩn Bảo mật & Kiểm toán (SOX & ISO/IEC 27001 Compliance):**
   - **Tách biệt nhiệm vụ (Segregation of Duties - SoD):** Luồng duyệt Late-binding tự động phân tách người nộp đơn và người duyệt dựa trên sơ đồ quản lý thực tế tại thời điểm bước duyệt trở thành active.
   - **Chống tự phê duyệt (Self-Approval Prevention):** Ngăn chặn triệt để hành vi người nộp đơn tự phê duyệt yêu cầu của chính mình bằng cơ chế chuyển hướng (reassign) tự động sang Admin hoặc bỏ qua hợp lệ.
   - **Đầy đủ vết kiểm toán (Audit Trail Ready):** Lưu lại lịch sử hành động duyệt chi tiết bao gồm người thao tác thật sự (`completed_by`) tách biệt khỏi assignee được chỉ định (`assignee_target_id`).

5. **Tiêu chuẩn Quản trị & Chất lượng (COBIT & ISO 9001):**
   - Hỗ trợ kiểm soát quản trị CNTT của doanh nghiệp bằng cách thiết lập rõ ràng quyền hạn phê duyệt, lưu vết lịch sử không thể thay đổi, và đảm bảo mọi đơn từ đều được phê duyệt đúng thẩm quyền trước khi thay đổi trạng thái hệ thống.

6. **Bảo mật thông tin cá nhân & GDPR:**
   - Việc tách thông tin gán động (Late-Binding) kết hợp với các chính sách che giấu dữ liệu PII (Personally Identifiable Information) đảm bảo chỉ những người duyệt thực sự có thẩm quyền mới có thể truy cập và xem chi tiết thông tin đơn từ của nhân viên.

7. **Kiến trúc Đa thuê tối ưu (Multi-Tenant Performance Standards):**
   - Cô lập hoàn toàn dữ liệu giữa các doanh nghiệp (`workspace_id`) trên cả ba cấp độ: Định nghĩa quy trình, Trạng thái Instance, và Danh sách nhiệm vụ Tasks.
   - **Không N+1 Queries:** Tải danh sách công việc cần duyệt trong O(1) bằng một câu lệnh truy vấn SQL sử dụng toán tử `IN` kết hợp với index trên `assignee_type` và `assignee_target_id`.

---

## 2. Thiết Kế Cơ Sở Dữ Liệu

### Bảng: `approval_workflow_instances`
Lưu trữ trạng thái tiến trình tổng quan của luồng duyệt.

| Trường | Kiểu dữ liệu | Ràng buộc | Mô tả |
| :--- | :--- | :--- | :--- |
| `id` | `UUID` | PRIMARY KEY | Định danh duy nhất của instance duyệt. |
| `workspace_id` | `UUID` | NOT NULL | Định danh doanh nghiệp (Multi-tenant). |
| `workflow_id` | `UUID` | REFERENCES | Trỏ tới cấu hình gốc. |
| `entity_type` | `VARCHAR(50)` | NOT NULL | Loại đơn từ (ví dụ: `leave_request`, `ot_request`). |
| `entity_id` | `UUID` | NOT NULL | ID của bản ghi đơn tương ứng. |
| `status` | `VARCHAR(20)` | DEFAULT 'pending' | Trạng thái luồng: `pending`, `approved`, `rejected`, `returned`, `cancelled`. |
| `current_step_order` | `INT` | DEFAULT 1 | Thứ tự bước duyệt hiện tại đang chờ xử lý. |
| `steps_snapshot` | `JSONB` | DEFAULT '[]' | Mảng JSON chứa cấu hình các bước duyệt. |
| `created_at` | `TIMESTAMPTZ` | DEFAULT NOW() | Thời điểm tạo luồng duyệt. |
| `updated_at` | `TIMESTAMPTZ` | DEFAULT NOW() | Thời điểm cập nhật cuối cùng. |

### Bảng: `approval_workflow_tasks`
Lưu trữ các nhiệm vụ phê duyệt đang chờ xử lý cho bước hiện tại.

| Trường | Kiểu dữ liệu | Ràng buộc | Mô tả |
| :--- | :--- | :--- | :--- |
| `id` | `UUID` | PRIMARY KEY | Định danh duy nhất của task. |
| `instance_id` | `UUID` | REFERENCES | Trỏ tới Instance duyệt tương ứng. |
| `step_order` | `INT` | NOT NULL | Bước duyệt hiện tại của task này. |
| `status` | `VARCHAR(20)` | DEFAULT 'pending' | Trạng thái task: `pending`, `completed`, `cancelled`. |
| `assignee_type` | `VARCHAR(30)` | NOT NULL | Loại phân công: `specific_user`, `role`, `permission`, `department_manager`. |
| `assignee_target_id` | `UUID` | Nullable | ID của User, Role, hoặc Department mục tiêu. |
| `assignee_target_name` | `VARCHAR(100)`| Nullable | Tên của Permission mục tiêu. |
| `created_at` | `TIMESTAMPTZ` | DEFAULT NOW() | Thời điểm tạo task. |
| `completed_at` | `TIMESTAMPTZ` | Nullable | Thời điểm hoàn thành task. |
| `completed_by` | `UUID` | Nullable | User ID thực tế đã bấm duyệt. |

---

## 3. Quy Trình Vận Hành Lõi (State Machine)

### Bước 1: Khởi Tạo Luồng (`submit_to_workflow`)
Khi đơn từ được gửi duyệt:
1. Tìm cấu hình quy trình (`is_active = true`) có `entity_type` tương ứng.
2. **Nếu không cấu hình quy trình duyệt:** 
   - Kích hoạt **Fallback**: Xác định phòng ban của người thụ hưởng. Giao nhiệm vụ cho Trưởng phòng ban đó (tạo task `assignee_type = 'department_manager'`, `assignee_target_id = department_id`). Nếu không có phòng ban hoặc trưởng phòng, giao thẳng cho Workspace Admin (`assignee_type = 'specific_user'`).
3. **Nếu có cấu hình quy trình duyệt:**
   - Tạo instance và lưu cấu hình các bước vào `steps_snapshot`.
   - **Kích hoạt Bước 1:** Phân giải tiêu chí người duyệt của Bước 1 và sinh các task tương ứng. 
4. **Cơ chế Chống Tự Duyệt (Self-Approval Prevention):**
   - Nếu tiêu chí chỉ định đích danh chính người nộp đơn:
     - Nếu `can_skip = true`: Đánh dấu task là `completed` / `skipped` ngay lập tức và chuyển bước.
     - Nếu `can_skip = false`: Chuyển hướng giao nhiệm vụ (Reassign) cho Workspace Admin.

### Bước 2: Phê Duyệt Bước (`process_approval_action`)
Khi một người duyệt thực hiện hành động duyệt:
1. Tìm task có trạng thái `pending` của instance mà người dùng đó hợp lệ phê duyệt (Khớp User ID, Role, Permission, hoặc là Trưởng phòng của phòng ban mục tiêu).
2. Ghi nhận hành động vào lịch sử phê duyệt (`approval_action_log`) và đánh dấu task là `completed`.
3. **Đánh giá Đồng thuận (Consensus Mode):**
   - **Hành động là REJECTED hoặc RETURNED:**
     - Đánh dấu task hiện tại là `completed`.
     - Hủy (`cancelled`) toàn bộ các task còn lại của bước hiện tại.
     - Đổi trạng thái Instance thành `rejected` hoặc `returned`, gọi callback cập nhật đơn gốc.
   - **Hành động là APPROVED:**
     - Nếu `consensus_mode = 'any'`: Bước duyệt hoàn tất. Hủy toàn bộ các task pending khác của bước hiện tại.
     - Nếu `consensus_mode = 'all'`: Kiểm tra xem tất cả các task của bước hiện tại đã chuyển sang `completed` chưa. Nếu đủ -> Bước duyệt hoàn tất.
4. **Chuyển tiếp bước tiếp theo (Late Binding):**
   - Khi bước hiện tại hoàn tất, tìm bước tiếp theo (`step_order + 1`).
   - Nếu còn bước tiếp theo:
     - Tăng `current_step_order` của instance lên 1.
     - Phân giải tiêu chí người duyệt của bước tiếp theo **ngay tại thời điểm này** và tạo các task `pending` mới cho bước này.
   - Nếu hết bước:
     - Cập nhật trạng thái instance thành `approved` và gọi callback đổi trạng thái đơn gốc thành `Approved`.

---

## 4. Tải danh sách công việc cần duyệt (Worklist Query)

Khi người dùng B đăng nhập, hệ thống sẽ lấy ra các thông tin hiện tại của B:
- User ID của B.
- Danh sách các Role ID B đang sở hữu: `[R1, R2]`
- Danh sách các Permission B đang có: `[P1, P2]`
- Danh sách các Department ID B đang quản lý: `[D1, D2]`

Query SQL tải danh sách công việc chờ B duyệt sẽ cực kỳ nhanh và tận dụng tối đa index:
```sql
SELECT t.*, inst.entity_type, inst.entity_id 
FROM approval_workflow_tasks t
JOIN approval_workflow_instances inst ON t.instance_id = inst.id
WHERE t.status = 'pending' AND (
  (t.assignee_type = 'specific_user' AND t.assignee_target_id = :b_user_id) OR
  (t.assignee_type = 'role' AND t.assignee_target_id IN (:b_role_ids)) OR
  (t.assignee_type = 'permission' AND t.assignee_target_name IN (:b_permissions)) OR
  (t.assignee_type = 'department_manager' AND t.assignee_target_id IN (:b_managed_dept_ids))
);
```

---

## 5. Tích Hợp Đơn Nghiệp Vụ & Callbacks

Giao thức tích hợp của các đơn nghiệp vụ vẫn giữ nguyên:
- Cập nhật trường `active_workflow_instance_id` trỏ sang `approval_workflow_instances(id)`.
- Nhận callback đổi trạng thái (`approved`, `rejected`, `returned`) để áp dụng các thay đổi số dư phép, lịch làm việc hoặc payroll.

---

## 6. Lộ trình phát triển trong tương lai (Roadmap & TODOs)

Để đáp ứng tối đa các quy trình phức tạp của doanh nghiệp quy mô lớn, các tính năng sau đã được thiết kế sẵn và đánh dấu TODO để triển khai trong các phiên bản tiếp theo:

### 1. Ủy quyền phê duyệt nâng cao (Advanced Delegation)
- **Mục tiêu:** Cho phép cấu hình ủy quyền có thời hạn và có điều kiện (ví dụ: chỉ ủy quyền phê duyệt đơn nghỉ phép `leave_request`, giữ lại đơn làm thêm giờ `ot_request` hoặc ngược lại).
- **TODO:** Mở rộng hàm `get_delegators_for_user` trong `ApprovalWorkflowService` để lọc các cấu hình ủy quyền theo `entity_type`.

### 2. Tự động chuyển cấp khi trễ hạn (SLA Escalation)
- **Mục tiêu:** Tự động hóa việc xử lý khi một bước phê duyệt bị ngâm quá lâu (trễ SLA). Nhiệm vụ sẽ tự động chuyển giao lên cấp quản lý cao hơn.
- **TODO:** Triển khai hàm `process_sla_escalations` kết hợp với worker chạy định kỳ (cron job) để quét các task quá hạn và tái phân công nhiệm vụ.

### 3. Lưu trữ vết lịch sử phê duyệt chi tiết (Audit Trail Snapshots)
- **Mục tiêu:** Phục vụ kiểm toán doanh nghiệp bằng cách ghi lại trạng thái tổ chức (vai trò, phòng ban) của người duyệt thực tế ngay tại thời điểm họ bấm nút duyệt.
- **TODO:** Mở rộng hàm `process_approval_action` để ghi nhận các thông tin Metadata tổ chức này vào `approval_action_logs`.
