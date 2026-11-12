# Chuẩn hóa đầy đủ Employment, Contract và E-contract

Status: Approved — kế hoạch chuẩn hóa tổng thể
Next slice: Approved — payroll lock backend và employment lifecycle safety
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
- Expand schema/backfill đã có bằng chứng dev ghi tại Task 3/4; không đồng nghĩa production đã migrate hoặc concurrency/date-containment đã đạt đủ gate.
- Commit `8166c6fd9` thêm termination segment + relationship cùng transaction và date validation; unit/format/vet PASS. Lệnh `./leedevkit test api` kết thúc exit `0`, nhưng log integration chỉ có start/teardown, không có kết quả test cụ thể: chưa chứng minh PostgreSQL integration cho termination. Test rollback hiện kiểm no-commit, chưa kiểm persisted rollback.
- `use-payroll-lifecycle.ts` lưu lock trong `localStorage`; chưa tìm thấy `payroll_periods` trong runtime/migrations đã rà. ADR payroll lifecycle là `Proposed`, có ví dụ workspace/Rust cũ, không phải contract triển khai hiện tại. Payroll persist có nhánh xóa snapshot rồi ghi lại; cần transaction và kiểm lỗi trước khi khóa có hiệu lực.

## Thực hiện — 10 đầu việc theo thứ tự

- [ ] **1. Chốt ADR và inventory.** Cập nhật ADR hiện có; rà SQL, repositories, imports, jobs, API, UI và reporting dùng `employments`/`employment_id`. Chốt end-date inclusive, future termination, correction/effective change, phase/manager continuity và loại contract đồng tồn tại. **Gate:** consumer matrix có owner; case HR chưa quyết định là blocker cụ thể, không suy đoán.

- [x] **2. API và quyền trước UI — read-only slice.** Đã thêm `GET /api/v1/employees/{employee_id}/employment-relationships` và detail route; list dùng pagination server-side, stable ordering `start_date DESC, id DESC`, bounded `page/limit`, filter `employer_id`, filter `as_of`, UUID/date validation, `AppError`, canonical `employees:read`, và wrong-parent `404`. Legacy employment write endpoints giữ nguyên; relationship lifecycle writes, department scope, contract relationship context, retry/concurrency semantics còn ở Task 5/6. **Gate hiện tại:** unit/API tests kiểm 200, 400 malformed filters/pagination, 403, 404 wrong parent; full `./leedevkit test api` PASS. 401 được bảo vệ bởi handler auth pattern và route test hiện có; 409, cross-scope writes và lifecycle semantics chưa thuộc read-only slice.

- [x] **3. Hoàn thiện expand schema và integrity.** Kiểm tra migration đã áp dụng ở đâu: chỉ sửa file chưa áp dụng; nếu đã áp dụng, thêm migration mới, không sửa baseline/marker để che khác biệt. Dùng exclusion constraint theo employee–employer và date range cho relationship không soft-delete; không thêm GiST index trùng constraint. Bảo vệ segment employee/employer khớp relationship bằng composite FK phù hợp; contract legacy link phải thuộc cùng relationship. Kiểm date containment bằng transaction locks/trigger phù hợp cho cả sửa parent/child, không dùng CHECK xuyên bảng. **Gate:** schema contract kiểm bảng/marker/constraint/FK/trigger/orphan; PostgreSQL behavior fixture kiểm same-employer overlap reject, different-employer overlap allow, invalid dates, segment mismatch và contract mismatch; empty rollback pass. Concurrent overlap và restore gây overlap còn pending.

- [x] **4. Reconcile và backfill có kiểm soát.** Audit aggregate không PII: snapshot hiện tại có 5 active segments, 5 employee–employer pairs, không duplicate pair, không orphan/date lỗi, 0 contract. Migration `20261112020000_employment_relationship_backfill` map đúng nhóm có một segment; nhóm nhiều segment không suy đoán; mapping giữ UUID segment, tạo relationship UUID mới, link contract chỉ khi segment đã map. **Gate:** dev verification đạt 5 relationships/5 linked segments/0 unlinked; migration marker có 1; populated rollback bị từ chối và giữ nguyên marker/data. Production backfill cần backup, reconciliation report và HR sign-off trước khi chạy.

- [ ] **5. Employment lifecycle và đường ghi thống nhất.** **Slice 5a hoàn tất:** thêm `POST /api/v1/employees/{employee_id}/employment-relationships`; repository tạo relationship + segment đầu trong một transaction, khóa employee row, kiểm employee/employer tồn tại, giữ UUID mới cho relationship/segment, map temporal/identity constraint thành `409`. **Slice 5b hoàn tất một phần:** transfer department khi split segment giữ nguyên `relationship_id`, khóa employee row trước khi đọc chain, thêm ordering ổn định `start_date ASC, id ASC`, và có regression test; transfer chưa có payroll-lock guard/audit đầy đủ. **Slice 5c một phần:** commit `8166c6fd9` đồng bộ termination segment + relationship; chưa xử lý cả segment chain, khóa payroll, future-effective state hoặc PostgreSQL rollback/concurrency. Legacy PATCH/import/bulk chưa thống nhất. **Chưa đánh dấu Task 5 hoàn tất:** hoàn thiện termination, rehire, payroll-lock guard, audit, retry/idempotency và unified validation còn pending. **Gate:** locked-payroll guard và audit không PII thô; same-day/future/retro, concurrent transfer/termination, retry và partial failure không để dữ liệu nửa chừng.

- [ ] **6. Contract lifecycle độc lập.** Relationship là liên kết chính; kiểm nhất quán `employment_id` compatibility. Bỏ auto-close ngầm, dùng action thay thế/chấm dứt tường minh; renewal tạo văn bản mới, phân biệt phụ lục. Giữ compatibility cho client vốn tạo active; chỉ xóa mềm nháp chưa được tham chiếu, signed/effective dùng transition có lịch sử. **Gate:** draft/signed/future/indefinite/renewal và overlap theo loại văn bản có test; transfer không đổi hợp đồng; locked-period không bị lách.

- [ ] **7. Chuyển consumer, không remap lịch sử.** Resolver attendance/leave/schedule/salary/payroll/report chọn đúng relationship và segment theo ngày, không lấy employment mới nhất để đoán employer. Phase/manager/assignment dùng phạm vi đã chốt; FK vẫn trỏ segment khi cần lịch sử phân công. **Gate:** golden fixtures một employer giữ kết quả; multi-employer không double-count, phase/manager không mất liên kết, payroll khóa không đổi.

- [ ] **8. UI và tách E-contract.** `/employments` quản lý relationship và segment timeline; `/contracts` tái dùng registry/form thường từ econtracts, độc lập feature flag; `/econtracts` chỉ envelope, `/settings/econtracts` chỉ cấu hình ký. Cập nhật employee tabs, navigation, URL filter schema, route generator và i18n theo tooling/component có sẵn; loading/empty/error/retry, keyboard/mobile đầy đủ. **Gate:** đúng employee/relationship context, refresh/back giữ filter; không mock ký/download hoặc làm mất envelope/PDF cũ.

- [ ] **9. Đổi tên, bỏ compatibility và rollout.** Trước rename `employments` thành `employment_segments`, xác nhận toàn bộ SQL/runtime/job consumer hỗ trợ tên mới; dùng cutover có phối hợp hoặc adapter đã kiểm thử nếu cần mixed versions, không giả rename tự tương thích. Giữ UUID/FK, cập nhật source/types/schema/docs và chỉ đổi tên field API qua contract đã duyệt. Bỏ `employee_contracts.employment_id` cùng adapter/field trùng khi không còn consumer; nếu cần provenance thì định nghĩa rõ, không giữ làm authority thứ hai. **Gate:** backup/restore rehearsal, lock budget, owner HR/engineering được chỉ định và sunset cụ thể; zero legacy usage qua một chu kỳ payroll đại diện, reconciliation đạt, không rollback binary cũ khi không an toàn. Chưa đạt thì đánh dấu migration chưa hoàn tất, không bỏ gate để xong task.

- [ ] **10. Verification cuối và bằng chứng bàn giao.** PostgreSQL fresh + populated DB: migration/backfill chạy lại, constraint/concurrency, empty rollback và refusal rollback khi có dữ liệu; kiểm roll-forward/restore và golden attendance/payroll. Sau unit chạy đầy đủ `./leedevkit test api`, `./leedevkit test web`, `./leedevkit test all`, đọc từng phase/log và `git diff --check`. Trước browser đọc `docs/dev-accounts.md`, UI audit rules và lập route × role matrix cho employee/team_lead/shift_manager/hr_manager/admin trên route bị tác động, cả allow/deny, console/network. **Gate:** ghi rõ PASS/FAIL/SKIP/blocker và route chưa test; không credentials/tokens/PII trong artifacts, không dùng tài khoản dev ở production.

## Slice tiếp theo — khóa payroll và lifecycle an toàn

**Status: Approved.** Approval nhận ngày 2026-11-12. Chỉ triển khai theo từng slice; chưa sửa runtime, chưa commit/push hoặc chạy migration production. Owner kỹ thuật: backend; nghiệm thu nghiệp vụ: HR/payroll. Phạm vi đề xuất: kỳ tháng theo employer trong một organization ngầm định, không `workspace_id`; cần HR/payroll duyệt trước migration. Không tự nâng ADR `Proposed` thành quyết định đã duyệt.

- [ ] **A. Chốt contract tối thiểu.** Bổ sung ADR hiện có: kỳ có `open/locked`, unique `(employer_id, year, month)`, khoảng ngày theo lịch nghiệp vụ; `CALCULATED` chỉ là trạng thái hiển thị từ payslip, không giả có approval workflow. API đề xuất `GET /api/v1/payroll/periods/{year}/{month}?employer_id=…`, `POST …/lock`, `POST …/reopen`; read dùng `payroll:read`, action dùng quyền riêng được seed/test, không suy từ tên role. Reopen cần lý do, version và không dùng để lách correction payroll đã chốt/đã trả. **Gate:** duyệt scope employer, quyền, điều kiện lock/reopen và error `400/401/403/404/409`; chưa duyệt thì dừng schema. Full approval/disbursement state machine và adjustment engine không thuộc slice này.
- [ ] **B. Expand schema + cutover.** Thêm migration mới cho period, version, actor/time và audit; tái dùng audit storage nếu bảo đảm cùng transaction, không tạo hệ thống audit thứ hai. Phân loại metadata T3, lý do tự do T2; audit chỉ lưu ID/mã lý do an toàn. Đối soát payslip `finalized/paid`, yêu cầu xác nhận lịch sử khóa; không tự coi mọi kỳ cũ là open và không nhập `localStorage` thành chứng cứ. **Gate:** fresh/populated, restartable backfill, constraint, audit retention, rollback rỗng và fail-closed với dữ liệu; deployment cũ không được ghi vượt guard sau cutover.
- [ ] **C. Khóa và payroll persistence cùng protocol.** Sửa persistence trong `internal/repository/payroll.go`: snapshot + payslip commit một lần, không bỏ qua lỗi DELETE, phân biệt `pgx.ErrNoRows` với lỗi DB. Calculation và lock dùng chung period mutex có trong DB; kể cả period chưa tồn tại phải có khóa ổn định trước insert. Định nghĩa lock order cho period/employee/relationship/segment, sort batch keys; kiểm version/source thay đổi trong lúc tính, tránh lưu kết quả cũ. **Gate:** lock đối đầu calculate, hai calculate, child insert lỗi, timeout/cancel đều không mất snapshot hoặc ghi qua kỳ khóa; không tự reset `finalized/paid` thành `draft`.
- [ ] **D. Guard theo khoảng ảnh hưởng, không chỉ một ngày.** Thêm orchestration tối thiểu theo dependency hiện có, dùng cùng transaction/lock với mutation; batch-load trước validation, không thêm query-per-row. So sánh old/new để xác định employer và khoảng tác động: sửa start/end, kéo dài/rút ngắn, transfer cascade, termination; kiểm cả kỳ cũ/mới và mọi segment liên quan, kể cả NULL relationship compatibility. DB/lock lookup lỗi phải từ chối, không coi là open. **Gate:** biên ngày inclusive, nhiều kỳ/employer, kỳ chưa có row, retro/future; kỳ khóa không liên quan không bị chặn nhầm.
- [ ] **E. Thống nhất writers + audit/retry.** Rà create/update/delete/restore employment, relationship create, transfer, import/bulk, worker thật và synchronous fallback; salary/config/attendance/leave writers ảnh hưởng payroll cũng phải có coverage hoặc blocker rollout tường minh. Giữ employee restoration khác HR rehire: rehire mới tạo relationship + segment, không mở lại đợt cũ. Termination xử lý toàn chain và future segments theo rule được duyệt; không đổi contract đã ký ngầm, leave entitlement không bị tính lặp khi retry. Carry actor/correlation từ request/job, audit thành công cùng transaction; batch thất bại trả 0 committed. **Gate:** cùng request lặp không nhân đôi segment/audit/proration; mọi entry point dùng guard, không claim worker coverage khi chưa tìm thấy handler chạy thật.
- [ ] **F. Chuyển UI khỏi local lock.** `use-payroll-lifecycle.ts` và payroll API client đọc backend theo employer/kỳ, gửi version khi action; bỏ `localStorage` authority và operator name giả. Lỗi API không hiện “unlocked”; giữ loading/error/retry, i18n, a11y và cập nhật generated types bằng tooling hiện có. **Gate:** hai browser/refresh cùng trạng thái, request trực tiếp vẫn bị chặn, UI `403/409` đúng, multi-employer không lẫn scope.
- [ ] **G. Verification và đo sau cùng.** Thêm test PostgreSQL thật trong target/wrapper hiện có: rollback persisted data, chain sau transfer rồi terminate, đồng thời lock/calculate/transfer/terminate, batch partial failure, audit failure, retry và legacy compatibility. Chạy unit rồi full `./leedevkit test api`, `./leedevkit test web`, `./leedevkit test all`; mỗi integration case phải có tên/kết quả/log, phase rỗng hoặc skip không tính PASS. Browser theo dev-account/rule matrix trước tương tác; EXPLAIN cho guard/timeline với dữ liệu đại diện, `git diff --check`. **Done:** không writer nào vượt lock trong phạm vi công bố; blocker ngoài phạm vi được ghi rõ, không gọi toàn payroll immutable nếu attendance/leave/salary còn bypass.

Không rename/drop compatibility, không thêm cache hoặc service framework. Thay thế nhẹ hơn: chỉ chặn trên payslip `finalized/paid`; ít code hơn nhưng không thay được khóa kỳ chưa có payslip, nên không chọn làm đích.

## Đo hiệu năng và điều kiện hoàn tất

- Với dữ liệu đại diện, dùng `EXPLAIN (ANALYZE, BUFFERS)` cho relationship list/as-of, segment history và contract lookup; tránh N+1, đo latency và index usage. Chưa có số đo thì không tuyên bố tối ưu hiệu năng; không thêm cache/materialized view theo phỏng đoán.
- Hoàn tất khi schema đích dùng `employment_segments`, ID/FK lịch sử nguyên vẹn, relationship/contract không đổi khi transfer, quyền/khóa payroll enforce ở mọi writer, UI tách đúng và integration có bằng chứng.
- Full normalization chưa hoàn tất nếu còn consumer phụ thuộc compatibility chưa có ngày/điều kiện gỡ. Phạm vi này không bao gồm provider ký, event sourcing hoặc mô hình lịch sử tổng quát mới.
