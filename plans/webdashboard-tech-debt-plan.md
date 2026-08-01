# 🌐 WebDashboard Technical Debt Remediation Plan

> **Generated:** 2026-08-01 · **Scope:** `webdashboard` (~125K LOC TS/TSX, 61 API modules)
> **Method:** Evidence-based audit (knowledge graph + grep + Read). Không giả định — mọi kết luận đều có file:line.
> **Status:** ⏳ Đề xuất — chờ approve trước khi thực thi.
> **Rules đã load:** `coding-standards.md`, `testing-standards.md`, `change-management.md`, `execution-wrappers.md`

---

## 0. Executive Summary

Bốn lớp nợ được xếp hạng theo mức ảnh hưởng người dùng và chi phí duy trì:

| Rank | Debt | Bằng chứng | Severity | Ước lượng |
|---|---|---|---|---|
| 🥇 #1 | **eContracts stub được ship ra production** (tính năng "giả" vỡ ngầm) | `econtracts.ts:60` `throw`, routed + app-registry, không feature flag | **CRITICAL** | 2-3 ngày (containment) + 1-2 tuần (wire backend) |
| 🥈 #2 | **API layer phân mảnh / di cư type-safety bỏ dở** | `cleanApiPath` vá `/api/api/v1`, 41 call site dùng escape hatch, 4 tầng fetch | **HIGH** | 5-7 ngày |
| 🥉 #3 | **Văn hóa "disable lint thay vì refactor"** — 281 eslint-disable, god components | `filter-panel.tsx` 632 dòng, disable `complexity`/`max-lines-per-function` ở đầu page | **HIGH** | 2-3 tuần (từng phần, theo PR) |
| #4 | **Vệ sinh repo** | `add-device-fail.png` (107KB), `test-results-new/.last-run.json`, `tsconfig.tsbuildinfo` bị track | LOW | 1 giờ |

**Nguyên tắc thứ tự:** TD-1 Phase 0 (ẩn feature) làm TRƯỚC TIÊN vì đang gây lỗi runtime cho user thật. TD-2 và TD-3 song song được — nhưng TD-3 nên refactor theo từng file nhỏ, PR riêng biệt để tránh review bị nghẽn.

---

## 1. 🥇 TD-1 — eContracts stub ship ra production (CRITICAL)

### Bằng chứng (đã verify)
- `webdashboard/src/lib/api/econtracts.ts` — toàn bộ là stub:
  - `saveEContractRule()` → `throw new Error("eContracts backend not wired — cannot save routing rule.")` (dòng 60)
  - `fetchEContractConfigs/Rules/Envelopes` → trả `[]` (dòng 31-53, 67-73)
- Backend `apiserver/src/services/econtract/` chỉ có **provider trait + dispatcher, KHÔNG có handler**.
- UI vẫn route + hiển thị: `/econtracts` (route-tree.gen.ts:261), `/workspace/settings/econtracts` (dòng 675), app launcher `app-registry.ts:120` (permission-gate nhưng KHÔNG feature flag).
- Stub được nối vào React Query mutation → user bấm Save là error:
  - `use-econtracts-settings.ts:85` → `saveEContractRule`, `:96` → `deleteEContractRule`
- Test hiện tại (`econtracts-settings-page.test.tsx`, `econtracts-page.test.tsx`) **mock các stub nên vẫn XANH** — tức là "green but broken" (test tạo cảm giác an toàn giả).

### Root cause
Tính năng được phát triển "contract-first" nhưng không có gate "backend sẵn sàng" trước khi mở UI; launcher/route không có cơ chế ẩn feature chưa hoàn thiện.

### Remediation — 4 Phase

**Phase 0 — Containment (khẩn cấp, 0.5-1 ngày, PR riêng, không cần chờ backend):**
1. Thêm cơ chế ẩn app trong registry: field `hidden?: boolean` hoặc `featureFlag?: string` vào `AppItem` (`app-registry.ts:42` khu vực type) + lọc trong `use-nav-menu-items.ts` và app launcher.
2. Set econtracts entry (`app-registry.ts:120`) → `featureFlag: "ff_econtracts"`, mặc định **OFF**.
3. Thêm feature flag registry tối giản (module `src/lib/feature-flags.ts`): đọc từ `localStorage`/env theo đúng convention `ff_<area>_<name>` (rule `change-management.md` §6).
4. Kiểm tra các route trực tiếp: nếu truy cập `/econtracts` khi flag OFF → redirect về launcher (guard trong route hoặc `AuthGuard`). Lazy routes giữ nguyên (bundle không bị xóa, tiết kiệm effort).
5. Xóa `document-management-page.tsx:20` comment lạc "Use correct type definition instead of any[]" nếu đã xử lý; nếu chưa, sửa type thật (dính TD-5).

**Phase 1 — Wire backend (1-2 tuần, có thể là backend task riêng):**
- Hiện thực handler econtract trong `apiserver` dựa trên trait/dispatcher có sẵn: CRUD config, routing rules, envelopes, signer status.
- Thêm migration nếu cần bảng lưu trữ (routing rules, envelopes).
- Hoàn thiện OpenAPI schema → `pnpm generate` (script hiện có trong repo) → `api-generated.ts` có paths.

**Phase 2 — Frontend thật (1-2 ngày):**
- Thay toàn bộ stub trong `econtracts.ts` bằng `client.GET/POST/PUT/DELETE` typed; xóa `UNWIRED()` + comment block.
- Bỏ `eslint-disable @typescript-eslint/require-await` (dòng 9).
- Cập nhật test cũ (bỏ mock stub, mock typed client theo pattern `use-*.test` hiện có).
- Thêm test cho happy path + error path của save/delete rule.

**Phase 3 — Rollout & cleanup:**
- Enable flag 100% (gradual nếu cần), verify staging trước production (gate `change-management.md` §3).
- Xóa flag + field `hidden` sau 30 ngày ổn định (rule §6: "stale flags are tech debt").

### Validation
- Phase 0: `/econtracts` trả redirect khi flag OFF; app launcher không hiện econtracts; `./leedevkit test web --unit-only` xanh.
- Phase 2: E2E test save routing rule đi tới backend thật; không còn `throw new Error("backend not wired")` trong `src/` (grep = 0).
- Phase 3: `./leedevkit test all` xanh; `grep -r "ff_econtracts"` chỉ còn entry registry (đã xóa sau cleanup).

---

## 2. 🥈 TD-2 — API layer phân mảnh, di cư type-safety bỏ dở (HIGH)

### Bằng chứng (đã verify)
- `client.ts:63-71` — `cleanApiPath()` vá chuỗi runtime: sửa `/api/api/v1` → `/api/v1`. Đây là **vá triệu chứng**: root cause nằm ở chỗ nhiều module gọi `apiFetch`/`client` với path đã chứa `/api/v1` trong khi `getFullUrl` (`api-fetch-helpers.ts:107-113`) lại prepend `API_BASE_URL` (`/api`).
- `client.ts:212-236` — escape hatch `fetch<T>()` / `fire()` với comment "Migrate callers... whenever the path becomes available": **41 call site / 34 file** vẫn đang dùng (đếm grep).
- 4 tầng fetch chồng lấn: raw `fetch` (logout/refresh/cookie) → `apiFetch` (helpers.ts:111) → `customFetch` (client.ts:77) → typed `client` (safeWrap).
- Schema drift còn sót: `approval-workflows.ts:59` `@ts-expect-error` + TODO "fix OpenAPI schema"; `shifts.ts:13` TODO `allow_warnings`.

### Root cause
Cuộc di cư từ "untyped apiFetch + manual types" sang "openapi-fetch typed client" được làm một nửa: escape hatch vẫn là con đường dễ nhất cho code mới, nên tỷ lệ áp dụng typed client không tăng.

### Remediation
1. **Fix gốc URL (0.5 ngày, ưu tiên #1 trong TD-2):**
   - Quy ước canonical: `API_BASE_URL` KHÔNG bao gồm `/api` (đổi comment + value), mọi path gọi client đều bắt đầu `/v1/...`; proxy Vite giữ nguyên.
   - Xóa `cleanApiPath()`; chuẩn hóa tại `getFullUrl` một nơi duy nhất (prepend `/api` cho path bắt đầu `/v1/`).
   - Rà 61 module trong `lib/api/`: sửa mọi call site tự prepend `/api/v1` (grep pattern `"/api/v1"` trong các file module, trừ `config.ts`).
   - Thêm test đơn vị cho `getFullUrl` (thêm case `/api/api/v1` regression — assert không bao giờ double prefix).
2. **Di cư 41 escape-hatch call site (3-4 ngày, chia theo domain):**
   - Mỗi call site trong 34 file → map sang `client.GET/POST/...`; nếu path thiếu trong `api-generated.ts` → thêm vào OpenAPI backend rồi regenerate (không dùng `as any`).
   - Nhóm theo domain: attendance (8-9 site), employees, leave, payroll, reports — mỗi nhóm 1 PR.
   - Sau mỗi PR: `grep -rn "fetch<\|fire("` giảm dần; cuối cùng = 0 trong `src/` (trừ `client.ts` định nghĩa — hoặc xóa luôn nếu hết người dùng).
3. **Fix schema drift (0.5 ngày):**
   - `approval-workflows.ts:59`: thêm `allow_partial_update` hoặc field `workflow_id` vào OpenAPI (tùy ý đồ backend), bỏ `@ts-expect-error`.
   - `shifts.ts:13`: thêm `allow_warnings` + `override_reason` vào schema backend, bỏ TODO.
4. **Hợp nhất tầng fetch (1 ngày, làm cuối):**
   - `apiFetch` và `customFetch` chỉ khác nhau ở header merge + timeout; gộp: `customFetch` gọi thẳng `apiFetch` với `headers` đã merge, xóa nhánh `Request`/`extractRequestInfo` nếu openapi-fetch cho phép truyền url string (kiểm tra version đang dùng — nếu bắt buộc `Request`, giữ nguyên `extractRequestInfo` nhưng bỏ body-cloning đã có `apiFetch` xử lý).
   - Giữ nguyên `raw fetch` cho logout/refresh (không cần token header — có lý do bảo mật, đừng gộp).

### Validation
- `grep -rn "cleanApiPath\|/api/api/v1" src/` = 0.
- `grep -rn "fetch<\|fire(" src/` (trừ `client.ts`) = 0.
- `./leedevkit test web --unit-only` + `--lint-only` xanh; coverage floor `src/lib/api/` (≥80/55/90/85) không tụt.

---

## 3. 🥉 TD-3 — eslint-disable sprawl & god components (HIGH)

### Bằng chứng (đã verify)
- **281** `eslint-disable` trong code production (không tính `__tests__`), riêng `route-tree.gen.ts` hợp lệ (file sinh).
- Top file vi phạm `max-lines-per-function`/`complexity`:
  - `components/ui/filter-panel.tsx` — 632 dòng
  - `components/shifts/shift-list.tsx` — 406 · `components/layout/icon-rail.tsx` — 391 · `components/leave/leave-type-manager.tsx` — 390 · `components/payroll/payroll-summary-table.tsx` — 374 · `components/shifts/weekly-roster-grid.tsx` — 361 · `pages/approvals/components/delegated-claims-tab.tsx` — 372 · `pages/monitor/jobs-table.tsx` — 356 · `pages/departments-page.tsx` — 348 · `pages/settings/custom-fields-page.tsx` — 347
- Page-level disable: `departments-page.tsx:1`, `working-patterns-editor-page.tsx:22`, `delegated-claims-tab.tsx:1`, `locations-settings.tsx:1`...

### Phân loại 281 disable (bước 0 — audit 0.5 ngày)
| Nhóm | Ví dụ | Hành động |
|---|---|---|
| A. Runtime-guard hợp lệ | `no-unnecessary-condition` cho API dynamic | Giữ + comment (đã có sẵn) |
| B. Cấu trúc (complexity / max-lines-per-function) | Page-level disable | **Refactor**, bỏ disable |
| C. Quy ước có chủ đích | `no-empty-function`, `prefer-nullish-coalescing` (cố tình giữ `""`) | Giữ, kèm comment |
| D. Vô tình/lạc đề | `unused-imports`, disable thừa sau khi refactor | Xóa ngay (Boy Scout) |

### Remediation (refactor theo từng PR nhỏ — mỗi file 1 PR, có test)
1. **`filter-panel.tsx` (632 dòng) — ưu tiên #1:** tách thành `filter-panel/` folder: `FilterFieldFactory` (render theo field type), `FilterValueInput` (per-type input), `use-filter-state.ts` (logic collapse/search), `filter-panel.tsx` (≤ 200 dòng). Có sẵn `filter-panel.test.tsx` (522 dòng) làm safety net.
2. **`delegated-claims-tab.tsx` (372)**: tách `DelegatedClaimsTable`, `ClaimStatusChip`, hooks query riêng.
3. **`jobs-table.tsx` (356) + `job-details-drawer.tsx`**: tách `JobStatusBadge`, `JobTableRow`, `use-jobs-query.ts`.
4. **`icon-rail.tsx` (391)**: tách `NavItemButton`, tooltip logic → `use-nav-item-tooltip.ts`.
5. **`shift-list.tsx` (406) / `weekly-roster-grid.tsx` (361) / `leave-type-manager.tsx` (390) / `payroll-summary-table.tsx` (374)**: tách header/toolbar/row component, giữ state hook riêng.
6. **Pages**: `departments-page.tsx`, `working-patterns-editor-page.tsx`, `custom-fields-page.tsx`, `locations-settings.tsx` — bỏ page-level disable sau khi tách `use-departments-page.ts` (đã tồn tại) và form components (đã có sẵn trong `pages/settings/components/`).
7. **Cảnh báo:** Không refactor lại `daily-report-table.tsx`/`employee-daily-drawer.tsx` ngay lúc này — đang có WIP refactor reports (git status có thay đổi `report_service`). Chờ WIP merge xong rồi mới chạm (tránh conflict).

### Validation
- Mỗi PR: bỏ disable tương ứng → `./leedevkit test web --lint-only` không còn cần nó; coverage file refactor không tụt dưới floor.
- Cuối chuỗi: đếm `grep -r "eslint-disable" src/ | grep -v __tests__` giảm từ 281 → ≤ 60 (chỉ còn nhóm A + C).
- Không có file production nào > 400 dòng (trừ file sinh/data).

---

## 4. #4 — Vệ sinh repo (LOW, 1 giờ, làm đầu tiên cùng Phase 0)

1. `git rm --cached` 3 mục rác: `webdashboard/add-device-fail.png`, `webdashboard/test-results-new/.last-run.json`, `webdashboard/tsconfig.tsbuildinfo`.
2. Bổ sung `.gitignore`: `add-device-fail.png`, `test-results-new/`, `*.tsbuildinfo`, `playwright-report/` (kiểm tra mục nào thiếu).
3. Xóa `test-results-new/` nếu không còn dùng (đã có `test-results/` mới).
4. Kiểm tra không làm vỡ CI (các file trên không phải dependency build).

### Validation
- `git ls-files | grep -E "\.png$|tsbuildinfo|test-results-new"` = 0 (trừ icon tauri hợp lệ).
- `./leedevkit test web --lint-only` vẫn xanh.

---

## 5. TD-5 — Quick wins (kèm theo các PR khác, mỗi cái < 30 phút)

| Item | Vị trí | Hành động |
|---|---|---|
| `any[]` chưa sửa | `pages/documents/document-management-page.tsx:20` | Dùng đúng type từ `api-generated.ts` |
| `Record<string, any>` | `hooks/common/use-filter-params.ts:36,63,76` | Generic hóa bằng `Partial<T>` / `URLSearchParams` typing |
| `any` còn sót production | `lib/api/token-store.ts`, `errors.ts`, `shift-utils.ts`, `roster-utils.ts`, `no-show-zone.ts`, `sheet.tsx`, `template-editor.tsx`, `customize-panel.tsx` | Thay bằng type cụ thể khi đang sửa file đó (Boy Scout) |
| Stub note sai | `lib/api/econtracts.ts` comment header | Xóa sau Phase 2 |

---

## 6. Trình tự & phụ thuộc

```
Tuần 1        Tuần 2        Tuần 3+ 
├─ TD-1 P0 ───┤ TD-1 P1 (backend, song song)
├─ TD-4 ──────┤ TD-1 P2 ──► TD-1 P3 (rollout flag)
├─ TD-2 step1 ┤ TD-2 step2 (per-domain PR) ──► TD-2 step3+4
└─────────────┴ TD-3 bắt đầu (1 file/PR, liên tục nhiều tuần, low-priority slot)
```

- **Blocker:** TD-1 P1 (backend) cần quyết định provider e-signature nào hỗ trợ (trait có sẵn — verify trong `apiserver/src/services/econtract/` trước khi code).
- **Không chạm** `daily-report-table.tsx` / `employee-daily-drawer.tsx` cho tới khi WIP reports merge (tránh conflict với work đang dở).
- TD-2 step 2 và TD-3 KHÔNG làm chung 1 PR (rủi ro diff quá lớn cho review).

## 7. Definition of Done

- [ ] `grep -r "backend not wired" src/` = 0 (hoặc econtracts bị ẩn hoàn toàn nếu chưa wire)
- [ ] `/api/api/v1` và `cleanApiPath` không tồn tại; `getFullUrl` có test double-prefix
- [ ] `fetch<`/`fire(` = 0 call site ngoài `client.ts`
- [ ] eslint-disable production ≤ 60, không file > 400 dòng
- [ ] `git ls-files` sạch rác test/build
- [ ] `./leedevkit test all` xanh trên `main` sau mỗi PR
- [ ] Coverage floor không tụt (vitest thresholds 65/58... giữ nguyên hoặc tăng)

## 8. Tuân thủ rules

- **R1 (Enterprise Premium Standard):** TD-1 P0 ẩn feature chưa production-ready; skeleton/empty-state/i18n giữ nguyên.
- **R2 (Change Management §6):** feature flag `ff_econtracts` đúng naming, cleanup trong 30 ngày; PR đủ 1 review, security 2 review.
- **R3 (Testing Standards):** mỗi PR có test, coverage floor không tụt; validation qua `./leedevkit test`.
- **R4 (Execution Wrappers):** mọi lệnh qua `scripts/_npm.sh`/`./leedevkit test` với redirect log.
- **R5 (Boy Scout):** TD-5 gom vào từng PR sẵn có, không tạo PR riêng cho quick win đơn lẻ.
