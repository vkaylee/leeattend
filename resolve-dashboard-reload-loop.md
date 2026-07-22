# Resolve dashboard reload loop

Request class: COMPLEX CODE / frontend deployment reliability.

Applicable rules:

- CORE-1 Enterprise Premium Standard: production-safe update UX and failure state.
- CORE-6 Hermetic Wrappers: validation runs through `./leedevkit test`.
- Change Management: deployment verification and cache invalidation are release gates.
- Configuration Management: required production deployment settings fail fast.

Scope:

- Generate a deployment-specific version manifest.
- Prevent repeated reloads while an edge deployment is converging.
- Apply explicit entry-point and manifest cache policies.
- Keep cache purge opt-in and fail deployment only when bounded post-deploy
  convergence verification cannot observe the released build.
- Verify the configured production domain when available, otherwise fall back
  to the Cloudflare Pages project domain.
