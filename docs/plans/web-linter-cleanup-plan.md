# Web Linter Cleanup

Status: Approved

## Goal
Đưa `webdashboard` về trạng thái không còn ESLint error/warning, giữ nguyên hành vi, rồi chứng minh bằng typecheck và full web test.

## Baseline
- Command: `leedevkit test web --lint-only`
- ESLint: `0 errors`, `584 warnings`; `152` warnings có thể auto-fix.
- Typecheck: pass.
- i18n reconciliation: pass.
- Existing unrelated working-tree change: `webdashboard/missing-keys.json`; không reset hoặc chỉnh file này.

## Constraints and rule IDs
- `CLAUDE-L0-5`: dùng LeeDevKit/repository wrappers, không chạy tool dependency trực tiếp trên host.
- `CLAUDE-L0-6`: sửa theo vòng lặp, đọc đầy đủ kết quả, không báo xanh dựa trên summary giả.
- `CLAUDE-L0-8`: lint không thay thế integration/E2E; sau unit/focused checks phải chạy full target.
- `TEST-1`: thay đổi code phải được kiểm tra bằng target phù hợp và ghi rõ pass/fail/warning.
- `GOV-6`: chỉ dùng auto-fix an toàn; không thêm suppression rộng, không hạ rule để che lỗi.

## Tasks
- [x] 1. Chụp baseline ổn định bằng `leedevkit test web --lint-only`, phân loại warning theo rule/file, và tạo danh sách file thuộc phạm vi; Verify: `584` warnings baseline, file `missing-keys.json` vẫn ngoài phạm vi.
- [x] 2. Chạy auto-fix an toàn qua `leedevkit test web --lint-only --fix`; review diff, loại bỏ mọi thay đổi ngoài lint và kiểm tra không đổi generated/ignored output; Verify: còn `437` warnings, không có error, chỉ thay đổi 20 source files do auto-fix ngoài file user đã sửa.
- [ ] 3. Sửa mechanical warnings: imports, interface/JSX prop ordering, unused imports/variables/arguments, boolean conversions, nullish/optional-chain simplifications, deprecated API usage; Verify: focused lint giảm đúng nhóm, typecheck pass.
- [ ] 4. Sửa JSX and accessibility warnings bằng component/pattern đã có (`Typography`, `Button`, proper labels/events/escaping), không đổi text contract hoặc keyboard behavior; Verify: tests liên quan pass, không còn `react/forbid-elements`, `jsx-a11y`, leaked-render warnings.
- [ ] 5. Refactor complexity/size warnings theo từng component/hook/API helper: tách pure helpers hoặc child components, giữ props/state/data contracts; không sửa rule threshold và không thêm file abstraction nếu không cần; Verify: existing unit tests pass và từng batch lint sạch.
- [ ] 6. Xử lý fast-refresh, named-default/import warnings và remaining type-aware warnings bằng module boundary/import chỉnh sửa tối thiểu; Verify: ESLint output có `0 errors, 0 warnings`.
- [ ] 7. Chạy full verification: `leedevkit test web`; đọc từng phase/log, gồm lint, typecheck, unit/integration/E2E nếu target cấu hình; Verify: tất cả applicable phases pass, không còn warning lint.

## Test impact matrix
- Behavior: không thêm behavior; refactor phải giữ output, event, accessibility, API/query-key contracts.
- Boundaries: giữ nguyên loading, empty, error, nullish, disabled, keyboard, and malformed-input paths.
- State/failure: không thay đổi async sequencing, mutation side effects, retry/error rendering, hoặc authorization behavior.
- Security/compatibility: không đổi API payload, auth, i18n keys, generated types, hoặc persisted state.
- Tests: không cần test mới cho ordering-only fixes; cập nhật/thêm characterization or regression tests khi tách logic hoặc thay đổi JSX behavior.

## Done when
- [ ] Direct lint result reports `0 errors` and `0 warnings`.
- [ ] `webdashboard` typecheck passes.
- [ ] Full `leedevkit test web` passes; no partial/unverified result.
- [ ] Existing `webdashboard/missing-keys.json` change preserved.
- [ ] No broad ESLint disable, threshold relaxation, generated-file edit, dependency change, or unrelated backend change.
- [ ] Final report lists changed files, tests, exact commands, warnings/skips, and remaining risks.

## Notes
- Lazy alternative: if project accepts warning-free only in production source, retain test-file warnings; current request says no linter errors, so default plan treats all linted `src` files and warnings as blocking.
- No implementation starts until plan approval.
