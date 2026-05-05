# 💻 Coding Standards & Best Practices

## 1. Language Handling
- **Communication:** Respond in the language the user initiated the prompt with.
- **Code:** ALL variables, function names, and code comments MUST be in English.

## 2. Clean Code Standards
- **SOLID & DRY:** Enforce Single Responsibility. Extract reusable logic.
- **Self-Documenting:** Write clear names. Minimal comments. No over-engineering.

## 3. Error Handling & Observability
- **Structured Errors:** Use domain-specific error types (e.g., `AppError`). Never throw generic `Error`.
- **Error Boundaries:** Wrap all independent UI components in React Error Boundaries.
- **Telemetry Baseline:** All backend APIs MUST include request tracing/metrics (e.g., OpenTelemetry span/context).

## 4. Security & Compliance
- **Data Privacy:** NEVER log Personally Identifiable Information (PII) like emails, passwords, phone numbers in raw text. Mask them (e.g., `user_***@domain.com`).
- **Secrets:** NEVER hardcode secrets. Use environment variables.

## 5. Boy Scout Rule
If you see a bug, typo, or poor pattern nearby while working on a file, FIX IT IMMEDIATELY.
> 🔴 **NEVER say "this is out of scope for the current task" when a clear bug is visible.**

## 6. Enterprise-Grade Premium Standard (MANDATORY)
> 🔴 **MANDATORY:** Every single feature, component, and API developed in this workspace MUST default to an **Enterprise-Grade Premium Ready** standard. Do NOT deliver MVP-style or basic implementations.
- **Frontend:** Premium design patterns, i18n support, skeleton loaders, empty states, micro-interactions.
- **Backend:** Strict validation (Type-safety, UUIDs), explicit error handling, transaction boundaries, audit logging.

## 7. Performance & Resource Optimization
- **Performance Budget:** JS bundle `< 200KB` gzipped. Core Web Vitals: LCP < 2.5s. Zero memory leaks for SPAs.
- **Scalability:** ZERO N+1 Queries allowed. ALL public APIs MUST implement Rate Limiting.

## 8. Escalation Protocol
- **The Rule:** If an error (e.g., Infra, DevOps) persists after **2 failed self-correction attempts**, STOP IMMEDIATELY. Generate an Incident Report and ask for human intervention. Do not endlessly retry.
