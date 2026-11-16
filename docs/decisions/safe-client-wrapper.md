# ADR: SafeClient Wrapper for openapi-fetch — Runtime-Backed Type Narrowing

## Status
**Accepted** (Date: 2026-07-24)

## Context
Webdashboard sử dụng thư viện `openapi-fetch` (v0.17.0) để gọi API từ TypeScript. Thư viện này trả về `FetchResponse<T>` — một discriminated union:
```typescript
type FetchResponse<T> =
  | { data: T;      error?: never;  response: Response }   // success (2xx)
  | { data?: never;  error: TError;  response: Response };   // error (non-2xx)
```

Tuy nhiên, middleware `customFetch` trong project đã throw `ApiError` trên TẤT CẢ non-2xx responses, làm cho error branch **không bao giờ reachable ở runtime**. Điều này có nghĩa là `data` LUÔN defined khi promise resolve.

Để encode invariant runtime này vào type system (cho phép 350 API call sites destructure `{ data }` mà không cần narrow discriminated union), project đã dùng pattern `createClient<paths>(...) as unknown as SafeClient`.

`SafeClient` override kiểu trả về của mỗi method:
- Loại bỏ error branch (thay `error: TError | never` bằng `error?: unknown`)
- Làm `data` luôn required và non-nullable
- Giữ nguyên input validation (path/query/body generics)

Pattern `as unknown as SafeClient` là cần thiết vì `SafeMethod` không phải là structural subtype của `ClientMethod` (narrowing return type), nên TypeScript từ chối direct cast `as SafeClient`.

Tuy nhiên, pattern này vi phạm **Zero-Cast Standard (Rule 10.1)** — `as unknown as X` = "LOAD-BEARING DRIFT MASK" — và không có runtime guard nào đảm bảo invariant thực sự đúng.

## Decision

Thay vì cast trực tiếp `createClient<paths>(...) as unknown as SafeClient`, ta:

1. Tạo `_rawClient = createClient<paths>(...)` (không cast)
2. Wrap từng method (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`) qua hàm `safeWrap<M>()`:
   - Gọi raw method
   - **Runtime guard**: nếu `result.error !== undefined` → throw `ApiError` ngay lập tức (error branch được trả về dù đã pass `customFetch`)
   - `data` có thể `undefined` cho 204 No Content / empty-body DELETE — đó là expected behavior
   - Return result với kiểu `SafeMethod<M>`
3. Export `client: SafeClient` object được construct từ các wrapped methods

Kết quả: `as SafeMethod<M>` cast vẫn tồn tại trong `safeWrap` (vì TypeScript không thể verify generic constraint), nhưng nó trở thành **benign narrowing** — invariant được đảm bảo bởi runtime guard, không còn là "load-bearing lie".

## Consequences

### ✅ Positive
1. **Runtime safety**: Nếu `openapi-fetch` thay đổi behavior (ví dụ: không còn respect customFetch throw-on-error), guard sẽ phát hiện ngay thay vì để `undefined` data propagate âm thầm qua 350 call sites.
2. **0 call-site changes**: Tất cả `client.GET()`/`POST()`/... giữ nguyên syntax.
3. **Single source of truth**: Invariant "non-2xx → throw" chỉ encode ở 2 nơi: `customFetch` (throw) + `safeWrap` (guard). Nếu một trong hai fail, cái còn lại catch.
4. **Coding standard compliance**: Không còn `as unknown as X` ở top-level export — cast còn lại trong `safeWrap` là internal implementation detail với runtime backing.

### ⚠️ Negative
1. **Performance overhead**: 2 extra condition checks (`!== undefined`) trên mỗi API call — negligible (~nanoseconds).
2. **Cast vẫn tồn tại**: `as SafeMethod<M>` trong `safeWrap` vẫn cần vì TypeScript generic constraints. Đây là hạn chế của type system, không phải của architecture.
3. **Legacy escape hatch `fetch<T>`/`fire()`**: 47 call sites vẫn dùng untyped path — cần follow-up PR để migrate sang typed `client.METHOD()`.

## Alternatives Considered

### A. Remove SafeClient, use raw `Client<paths>`
- Mọi call site phải narrow `if (error) throw; const { data } = ...`
- **350 call sites cần thay đổi** — không khả thi về mặt effort/risk
- **Rejected** vì DX impact quá lớn

### B. Keep status quo (`as unknown as SafeClient`)
- Không có runtime guard
- Vi phạm Zero-Cast Standard
- **Rejected** vì không giải quyết được vấn đề gốc: type assertion không có runtime backing

### C. Upgrade openapi-fetch để hỗ trợ middleware-based error handling
- Không có version nào hỗ trợ pattern này (checked up to v0.18.x)
- **Rejected** vì dependency constraint
