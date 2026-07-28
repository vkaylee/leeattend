# ADR-006: Client State Model — URL-First Layered State with Zod-Validated Search

**Status:** Accepted
**Date:** 2026-07-27
**Context:** Webdashboard SPA — canonical model for tab, filter, pagination, UI state
**Related:** — (first ADR in Client Infrastructure series)

---

## Context

Trong SPA hiện tại (`webdashboard/`), state phía client được xử lý theo 3+ pattern không đồng nhất:

- **useState local:** `use-attendance-logs-page.ts:149`, `use-attendance-adjustments-page.ts:29`, `leave-requests-page`, `shifts-page`, `profile-page`, `payroll-settings`, `leave-settings-tabs`, `lifecycle-templates-page`... — tab active, filter, employee selected sống trong component state, mất sạch khi user chuyển route hoặc back.
- **URL-then-strip anti-pattern:** `use-attendance-logs-page.ts:144-160` đọc query params `from`, `to`, `employee` để seed local state, rồi ngay lập tức `window.history.replaceState` xoá param — URL và state phân kỳ sau tương tác đầu tiên.
- **`validateSearch` một vài chỗ:** `shifts.rules.index.tsx`, `econtracts.tsx`, `workflows-settings.tsx` đã dùng `Route.useSearch()` với Zod schema — đúng chuẩn nhưng chưa nhân rộng.

Hệ quả:
- User chuyển tab/filter, sau đó chuyển page rồi quay lại → mất state (đây là "brainstorm" ban đầu của bug report).
- Support/QA không tái hiện được state từ link user gửi.
- Sentry breadcrumb chỉ có URL path, không có filter context.
- E2E test phải click chuỗi thao tác để đến state; không thể navigate trực tiếp.

Vòng phản biện đầu đề xuất **Zustand + persist** làm nồi lẩu state. → REJECT vì:
- Trùng nguồn sự thật với URL: back button, share link, deep-link email không đồng bộ được với store.
- Thêm dependency ~5KB gzip cho vấn đề đã có công cụ giải quyết (`@tanstack/react-router` + `zod` đã cài).
- Ẩn state khỏi observability (Sentry / Datadog RUM chỉ capture URL, không capture store).

Đề xuất thứ hai là **nuqs / useQueryState**. → REJECT vì trùng chức năng với TanStack Router `validateSearch` — kém type-safe hơn (không Zod-first), thêm dep vô ích.

---

## Decision

Áp dụng **URL-First Layered State Model** với 4 tầng phân biệt rõ ràng theo *nature* của state, không phải theo mức độ tiện.

### Cascade of state ownership

```
Server data              →  React Query (đã có)
  ↑
Shareable client state   →  URL search params (Zod-validated)
  ↑
Private UI state         →  sessionStorage (via useSessionState)
  ↑
Cross-session user prefs →  localStorage (via usePersistentState)
```

### Layer semantics

| Tầng | Nơi lưu | Ví dụ | TTL | Shareable | Hook |
|---|---|---|---|---|---|
| A. Server | React Query cache | list logs, employees | `staleTime` | qua URL | `useQuery` |
| B. Shareable | URL search params | tab, filter, sort, page, drawer open | Vô hạn | ✓ | `Route.useSearch()` + `usePatchSearch()` |
| C. Private UI | sessionStorage | scroll pos, form nháp, panel expand | Session tab | ✗ | `useSessionState()` |
| D. User prefs | localStorage | theme, column visibility, sidebar collapsed | Vô hạn | ✗ | `usePersistentState()` |

### Rules

1. **URL is the single source of truth** cho mọi state có thể share/deeplink/reproduce. Nếu 2 user paste cùng URL phải thấy **cùng UI state** (bỏ qua permission difference).
2. **Route file owns the contract.** File `routes/.../<page>.tsx` khai báo `validateSearch: schema.parse`. Không component nào đọc/parse URL bằng tay (cấm `new URLSearchParams(window.location.search)`).
3. **Zod schema với `.catch()` fallback.** Search schema phải xử lý an toàn khi:
   - Param cũ bị bỏ (backward compat khi rename)
   - User paste link nhập tay sai định dạng
   - Type mismatch (string → number coerce)

   Fallback KHÔNG BAO GIỜ throw. Toàn schema có `.catch(defaultValue)` ở top level.
4. **Search params replace, not push.** Update filter/tab dùng `navigate({ search, replace: true })` — tránh spam history stack. Chỉ `push` khi user thực sự "điều hướng" (đổi page).
5. **Không dùng `useState` cho state thoả 1 trong 4 điều kiện Layer B:**
   - Có thể share qua link cho đồng nghiệp / support
   - Cần giữ khi user back/forward hoặc refresh
   - Là input cho server query (filter → API call)
   - Có ảnh hưởng tới analytics/observability
6. **sessionStorage / localStorage key format:** `leeattend:<domain>:<field>` — namespace chặt để tránh đụng độ và dễ audit trong DevTools.
7. **SSR-safe hooks.** `useSessionState` / `usePersistentState` phải guard `typeof window === "undefined"` và return initial value trong hydration first render.

### Directory contract

```
webdashboard/src/
├── lib/state/
│   ├── use-search-patch.ts        ← merge helper cho Route.useSearch
│   ├── use-session-state.ts       ← sessionStorage-backed useState
│   ├── use-persistent-state.ts    ← localStorage-backed useState
│   └── index.ts                   ← barrel export
├── routes/.../<page>.tsx          ← validateSearch: schema.parse
└── pages/<domain>/
    ├── search-schema.ts           ← Zod schema, exported
    └── hooks/use-<page>-page.ts   ← Page State Hook (đã là pattern hiện có)
```

### Migration policy

- Tồn tại đồng thời cả 2 pattern trong quá trình rollout — không big-bang.
- Mỗi page migrate độc lập, mỗi PR self-contained (route file + schema + page hook).
- Reference migration: `attendance-logs-page` (page có state phức tạp nhất — tab + employee + dateRange + filterMode).
- Order áp dụng: attendance-logs → attendance-adjustments → leave-requests → shifts → settings.
- Trong quá trình rollout, code review reject PR mới nếu thêm `useState<TabType>` hoặc `URLSearchParams(window.location.search)` cho Layer B state.

---

## Consequences

**Tích cực:**
- Share link/deep-link/back-forward hoạt động free cho mọi filter — QA và Support không phải hỏi "làm sao bạn tới trang đó".
- Sentry breadcrumb URL capture đủ context bug reproduce (search params luôn phản ánh state).
- E2E test navigate trực tiếp URL → giảm test flake và tăng speed.
- Type-safe end-to-end nhờ Zod inference — page hook nhận `Route.useSearch()` đã typed.
- Không thêm dependency mới, không thêm bundle size, không thêm ADR churn.
- Tận dụng pattern `use-<page>-page.ts` đã tồn tại trong codebase — chỉ đổi ruột, không đổi shape.

**Tiêu cực:**
- URL có thể dài với filter nhiều field (ví dụ attendance có 6-7 param). Chấp nhận được với desktop dashboard. Nếu về sau cần rút gọn có thể serialize compact (`?f=base64url(cbor)`), NHƯNG chỉ khi có data cho thấy vấn đề thật sự.
- Migration effort trải dài — cần discipline PR review để không hồi quy vào pattern cũ.
- Form nháp phức tạp (workflow editor, employee bulk edit) không phù hợp URL → phải rõ ràng dùng `useSessionState` thay vì bị cám dỗ nhét vào URL.

**Rủi ro & mitigation:**
- Rủi ro: dev thêm filter mới nhưng quên update Zod schema → chạy runtime nhưng type lỏng lẻo. Mitigation: schema và setter đều generic từ Zod inference; TypeScript báo lỗi khi setter nhận key không có trong schema.
- Rủi ro: sessionStorage / localStorage schema đổi giữa các version → parse lỗi ở user cũ. Mitigation: hook wrap `JSON.parse` trong try/catch, fallback initial value; schema nội bộ có `version` field khi cần migration.
- Rủi ro: cross-tab sync (user mở 2 tab, đổi theme ở tab 1 mong tab 2 update). Không giải quyết ở ADR này — nếu cần, thêm hook `useBroadcastState` (BroadcastChannel API) sau, không ảnh hưởng contract hiện tại.
- Rủi ro: URL leak qua log/analytics chứa filter nhạy cảm (ví dụ `employee=<uuid>`). Mitigation: UUID không phải PII trực tiếp; sensitive filter (search text chứa tên/email) không đưa lên URL — dùng sessionStorage.

---

## Alternatives Considered

- **Zustand + persist middleware:** rejected — trùng nguồn sự thật với URL, ẩn state khỏi observability, thêm dep không cần thiết.
- **nuqs (useQueryState):** rejected — trùng chức năng với `Route.useSearch()`, kém type-safe hơn Zod-first.
- **Redux Toolkit + RTK Query:** rejected — over-engineering cho SPA scale hiện tại; React Query đã đủ cho server state.
- **React Router v6 useSearchParams:** N/A — dự án đã chọn TanStack Router (không đổi).
- **Local state + URL push on unmount:** rejected — race condition khi navigate nhanh, back button không phục hồi state chính xác.
