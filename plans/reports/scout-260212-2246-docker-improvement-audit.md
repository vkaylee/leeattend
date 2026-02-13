# Scout Report: Docker Improvement Audit

## Files Scanned

| File | Location |
|------|----------|
| docker-compose.yml | root (prod) |
| docker-compose.dev.yml | root (dev) |
| docker-compose.test.yml | root (test) |
| Dockerfile, Dockerfile.dev, Dockerfile.test | apiserver/ |
| Dockerfile, Dockerfile.dev, Dockerfile.test | agent-main/ |
| Dockerfile, Dockerfile.test | webdashboard/ |
| nginx-dev.conf | root |
| nginx.conf | webdashboard/ |
| .dockerignore | apiserver/, agent-main/, webdashboard/ |

## Critical Issues

### 1. Hardcoded Secrets
| File | Line | Value |
|------|------|-------|
| docker-compose.dev.yml | 53 | `JWT_SECRET=supersecret` |
| docker-compose.yml | 8 | `POSTGRES_PASSWORD: password_required` |
| docker-compose.dev.yml | 9 | Same weak default password |

**Fix:** Externalize to `.env` file, remove defaults for secrets.

### 2. Unpinned Base Images
All Rust Dockerfiles use `FROM rust:latest`. Node uses `node:20-slim` (major only).

**Fix:** Pin exact versions (e.g., `rust:1.83`, `node:20.11.0-slim`).

### 3. Running as Root
No Dockerfile has a `USER` directive — all services run as root.

**Fix:** Add non-root user in runtime stages.

## High Priority

| Issue | Files Affected | Fix |
|-------|---------------|-----|
| No resource limits | All compose files | Add `deploy.resources.limits` |
| Missing nginx security headers | nginx-dev.conf, webdashboard/nginx.conf | Add X-Frame-Options, CSP, etc. |
| Incomplete .dockerignore | All 3 services | Add `.env*`, `*.log`, `.DS_Store` |
| No `--locked` on cargo install | Dockerfile.test (both) | Add `--locked` flag |

## Medium Priority

| Issue | Files Affected | Fix |
|-------|---------------|-----|
| npm install on every start | docker-compose.dev.yml:97 | Bake into image or skip if node_modules present |
| No gzip in nginx | Both nginx configs | Add gzip directives |
| Unpinned tool downloads | Dockerfile.test (sccache, nextest) | Pin versions + SHA verification |
| No HEALTHCHECK in prod Dockerfiles | apiserver/Dockerfile, agent-main/Dockerfile | Add HEALTHCHECK directive |
| Playwright downloads uncached | webdashboard/Dockerfile.test | Use BuildKit cache mount |

## Low Priority

| Issue | Files Affected |
|-------|---------------|
| No client_body_size_limit in nginx | Both nginx configs |
| DB fsync=off in dev compose | docker-compose.dev.yml (OK for dev) |
| SCCACHE_DIR not mounted as volume | agent-main/Dockerfile.test |

## What's Already Good

- Multi-stage builds in production Dockerfiles (apiserver, agent-main, webdashboard)
- BuildKit cache mounts for cargo registry in prod builds
- cargo-chef for dependency layer caching
- Host cargo registry mounted in dev (avoids re-downloading)
- Proper apt cache cleanup in all images
- SQLX_OFFLINE mode for build-time DB independence
- Health checks on db/redis in compose files
