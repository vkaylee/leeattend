# Chuẩn hóa đầy đủ Employment, Contract và E-contract

Status: Approved
Updated: 2026-11-12

Bản sửa thay kế hoạch đã duyệt trước: bổ sung đích cuối `employment_segments`, constraint và gate bỏ compatibility. Hướng chuẩn hóa đầy đủ đã được chọn và triển khai theo từng slice có kiểm chứng.

## Mục tiêu và mô hình đích

```text
employees
  └── employment_relationships
        ├── employment_segments   # đổi tên employments hiện có, giữ UUID/FK
        └── employee_contracts
              └── econtract_envelopes
```

Relationship là một đợt làm việc với một pháp nhân; segment là lịch sử phân công. Contract thuộc relationship, envelope là lịch sử yêu cầu ký. Phase giữ phạm vi relationship hoặc segment tùy loại, không thay thế lifecycle của chúng.

## Quyết định và giới hạn

- Khác employer được làm đồng thời; cùng employee–employer không overlap relationship. Rehire tạo relationship mới; transfer và renewal không tạo relationship mới.
- Giữ nguyên UUID segment và FK lịch sử. Không tạo bảng segment thứ hai để copy dữ liệu; không đổi nghĩa `employment_id` cũ thành relationship ID.
- Relationship sở hữu employee/employer và đợt làm việc; segment sở hữu department/title/location/manager theo ngày. Field legacy trùng phải nhất quán, không có hai nguồn sửa độc lập.
- Chặn hồi tố ảnh hưởng payroll đã khóa, kể cả qua import/bulk/legacy API. Điều chỉnh dùng quy trình riêng; không xây thêm hệ thống adjustment nếu đã có.
- Contract đã ký/có hiệu lực không sửa đè; thay thế, phụ lục, chấm dứt có lịch sử. Không áp một luật overlap cho mọi loại văn bản.
- Tách `/contracts` khỏi `/econtracts`; không tích hợp provider, không giả ký thành công. Giữ `ff_econtracts` tắt khi provider chưa sẵn sàng.
- Không sửa `.leedevkit/`, không ghi đè thay đổi UI/RBAC khác, không commit/push hoặc chạy migration production. Rename/drop và cleanup cần gate tương thích, cùng phê duyệt riêng cho thao tác production không thể đảo ngược an toàn.

## Rule IDs

- `architecture-governance.md §1–2, §5`: mô hình tối thiểu, ADR, owner và điều kiện gỡ compatibility.
- `database-rules.md §1–4`: single organization, parameterized SQL, transaction, soft-delete, FK/index và rollback.
- `migration-and-rollback.md §1–6`: expand/migrate/verify/contract, backfill restartable, đối soát và recovery.
- `api-governance.md §1–3, §6`: contract-first, không tráo ID/response legacy, pagination và generated clients.
- `testing-standards.md §1–4`: negative assertions, PostgreSQL integration, kiểm từng phase; unit-only không chứng minh hoàn tất.

## Hiện trạng — không coi là hoàn tất

- Đã có ADR `docs/adr/2026-11-12-employment-relationship-and-contract-boundaries.md`; cần cập nhật đích rename và cleanup.
- Đã thêm route danh sách contract theo employee, permission checks và parent ownership tests trong working tree; chưa hoàn tất department/employer scope.
- Migration `20261112010000_employment_relationships` mới tạo bảng, nullable links, FK/index; GiST index chưa chặn overlap. Chưa có bằng chứng fresh/populated migration, reconciliation hoặc backfill hoàn tất.
- Lint-only sau auto-format đã PASS; full API sau sửa format chưa được xác minh lại. Chưa xác minh web/all hoặc browser trong slice này.

## Thực hiện — 10 đầu việc theo thứ tự

- [ ] **1. Chốt ADR và inventory.** Cập nhật ADR hiện có; rà SQL, repositories, imports, jobs, API, UI và reporting dùng `employments`/`employment_id`. Chốt end-date inclusive, future termination, correction/effective change, phase/manager continuity và loại contract đồng tồn tại. **Gate:** consumer matrix có owner; case HR chưa quyết định là blocker cụ thể, không suy đoán.

- [x] **2. API và quyền trước UI — read-only slice.** Đã thêm `GET /api/v1/employees/{employee_id}/employment-relationships` và detail route; list dùng pagination server-side, stable ordering `start_date DESC, id DESC`, bounded `page/limit`, filter `employer_id`, filter `as_of`, UUID/date validation, `AppError`, canonical `employees:read`, và wrong-parent `404`. Legacy employment write endpoints giữ nguyên; relationship lifecycle writes, department scope, contract relationship context, retry/concurrency semantics còn ở Task 5/6. **Gate hiện tại:** unit/API tests kiểm 200, 400 malformed filters/pagination, 403, 404 wrong parent; full `./leedevkit test api` PASS. 401 được bảo vệ bởi handler auth pattern và route test hiện có; 409, cross-scope writes và lifecycle semantics chưa thuộc read-only slice.

- [x] **3. Hoàn thiện expand schema và integrity.** Kiểm tra migration đã áp dụng ở đâu: chỉ sửa file chưa áp dụng; nếu đã áp dụng, thêm migration mới, không sửa baseline/marker để che khác biệt. Dùng exclusion constraint theo employee–employer và date range cho relationship không soft-delete; không thêm GiST index trùng constraint. Bảo vệ segment employee/employer khớp relationship bằng composite FK phù hợp; contract legacy link phải thuộc cùng relationship. Kiểm date containment bằng transaction locks/trigger phù hợp cho cả sửa parent/child, không dùng CHECK xuyên bảng. **Gate:** schema contract kiểm bảng/marker/constraint/FK/trigger/orphan; PostgreSQL behavior fixture kiểm same-employer overlap reject, different-employer overlap allow, invalid dates, segment mismatch và contract mismatch; empty rollback pass. Concurrent overlap và restore gây overlap còn pending.

- [x] **4. Reconcile và backfill có kiểm soát.** Audit aggregate không PII: snapshot hiện tại có 5 active segments, 5 employee–employer pairs, không duplicate pair, không orphan/date lỗi, 0 contract. Migration `20261112020000_employment_relationship_backfill` map đúng nhóm có một segment; nhóm nhiều segment không suy đoán; mapping giữ UUID segment, tạo relationship UUID mới, link contract chỉ khi segment đã map. **Gate:** dev verification đạt 5 relationships/5 linked segments/0 unlinked; migration marker có 1; populated rollback bị từ chối và giữ nguyên marker/data. Production backfill cần backup, reconciliation report và HR sign-off trước khi chạy.

- [ ] **5. Employment lifecycle và đường ghi thống nhất.** **Slice 5a hoàn tất:** thêm `POST /api/v1/employees/{employee_id}/employment-relationships`; repository tạo relationship + segment đầu trong một transaction, khóa employee row, kiểm employee/employer tồn tại, giữ UUID mới cho relationship/segment, map temporal/identity constraint thành `409`. **Slice 5b hoàn tất một phần:** transfer department khi split segment giữ nguyên `relationship_id`, khóa employee row trước khi đọc chain, thêm ordering ổn định `start_date ASC, id ASC`, và có regression test; transfer chưa có payroll-lock guard/audit đầy đủ. Legacy PATCH/import/bulk chưa đổi. **Chưa đánh dấu Task 5 hoàn tất:** termination, rehire, payroll-lock guard, audit, retry/idempotency và unified validation còn pending. **Gate:** locked-payroll guard và audit không PII thô; same-day/future/retro, concurrent transfer/termination, retry và partial failure không để dữ liệu nửa chừng.

- [ ] **6. Contract lifecycle độc lập.** Relationship là liên kết chính; kiểm nhất quán `employment_id` compatibility. Bỏ auto-close ngầm, dùng action thay thế/chấm dứt tường minh; renewal tạo văn bản mới, phân biệt phụ lục. Giữ compatibility cho client vốn tạo active; chỉ xóa mềm nháp chưa được tham chiếu, signed/effective dùng transition có lịch sử. **Gate:** draft/signed/future/indefinite/renewal và overlap theo loại văn bản có test; transfer không đổi hợp đồng; locked-period không bị lách.

- [ ] **7. Chuyển consumer, không remap lịch sử.** Resolver attendance/leave/schedule/salary/payroll/report chọn đúng relationship và segment theo ngày, không lấy employment mới nhất để đoán employer. Phase/manager/assignment dùng phạm vi đã chốt; FK vẫn trỏ segment khi cần lịch sử phân công. **Gate:** golden fixtures một employer giữ kết quả; multi-employer không double-count, phase/manager không mất liên kết, payroll khóa không đổi.

- [ ] **8. UI và tách E-contract.** `/employments` quản lý relationship và segment timeline; `/contracts` tái dùng registry/form thường từ econtracts, độc lập feature flag; `/econtracts` chỉ envelope, `/settings/econtracts` chỉ cấu hình ký. Cập nhật employee tabs, navigation, URL filter schema, route generator và i18n theo tooling/component có sẵn; loading/empty/error/retry, keyboard/mobile đầy đủ. **Gate:** đúng employee/relationship context, refresh/back giữ filter; không mock ký/download hoặc làm mất envelope/PDF cũ.

- [ ] **9. Đổi tên, bỏ compatibility và rollout.** Trước rename `employments` thành `employment_segments`, xác nhận toàn bộ SQL/runtime/job consumer hỗ trợ tên mới; dùng cutover có phối hợp hoặc adapter đã kiểm thử nếu cần mixed versions, không giả rename tự tương thích. Giữ UUID/FK, cập nhật source/types/schema/docs và chỉ đổi tên field API qua contract đã duyệt. Bỏ `employee_contracts.employment_id` cùng adapter/field trùng khi không còn consumer; nếu cần provenance thì định nghĩa rõ, không giữ làm authority thứ hai. **Gate:** backup/restore rehearsal, lock budget, owner HR/engineering được chỉ định và sunset cụ thể; zero legacy usage qua một chu kỳ payroll đại diện, reconciliation đạt, không rollback binary cũ khi không an toàn. Chưa đạt thì đánh dấu migration chưa hoàn tất, không bỏ gate để xong task.

- [ ] **10. Verification cuối và bằng chứng bàn giao.** PostgreSQL fresh + populated DB: migration/backfill chạy lại, constraint/concurrency, empty rollback và refusal rollback khi có dữ liệu; kiểm roll-forward/restore và golden attendance/payroll. Sau unit chạy đầy đủ `./leedevkit test api`, `./leedevkit test web`, `./leedevkit test all`, đọc từng phase/log và `git diff --check`. Trước browser đọc `docs/dev-accounts.md`, UI audit rules và lập route × role matrix cho employee/team_lead/shift_manager/hr_manager/admin trên route bị tác động, cả allow/deny, console/network. **Gate:** ghi rõ PASS/FAIL/SKIP/blocker và route chưa test; không credentials/tokens/PII trong artifacts, không dùng tài khoản dev ở production.

## Đo hiệu năng và điều kiện hoàn tất

- Với dữ liệu đại diện, dùng `EXPLAIN (ANALYZE, BUFFERS)` cho relationship list/as-of, segment history và contract lookup; tránh N+1, đo latency và index usage. Chưa có số đo thì không tuyên bố tối ưu hiệu năng; không thêm cache/materialized view theo phỏng đoán.
- Hoàn tất khi schema đích dùng `employment_segments`, ID/FK lịch sử nguyên vẹn, relationship/contract không đổi khi transfer, quyền/khóa payroll enforce ở mọi writer, UI tách đúng và integration có bằng chứng.
- Full normalization chưa hoàn tất nếu còn consumer phụ thuộc compatibility chưa có ngày/điều kiện gỡ. Phạm vi này không bao gồm provider ký, event sourcing hoặc mô hình lịch sử tổng quát mới.
