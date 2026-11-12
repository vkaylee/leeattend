# LeeAttend API Server (Go)

Central high-performance API server for LeeAttend, built with Go 1.26+, `github.com/go-chi/chi/v5`, `oapi-codegen` strict server, `pgx/v5` connection pool, and `sqlc`.

> **Migration Note:** Migrated from Rust/Axum to Go in October 2026 (Phase 9 Production Cutover complete). Legacy Rust source in `src/` and `Cargo.toml` is decommissioned and retained for archival reference only. Database schema migrations in `migrations/` remain active.

### Prerequisites

- **Go 1.26+**: Install via [golang.org](https://go.dev/)
- **Docker / Podman**: For containerized execution and integration testing
### Installation

1. Navigate to the `apiserver` directory:
   ```bash
   cd apiserver
   ```

2. Copy the environment template:
   ```bash
   cp .env.example .env
   ```

3. Build the project:
   ```bash
   go build -o bin/apiserver .
   ```
### 🏃 Running as a Windows Service

1. Copy `nssm.exe` to the `apiserver/` directory.
2. Build the Go binary: `go build -o bin/apiserver .`.
3. Right-click `install_service.bat` and select **Run as Administrator**.
4. The service `LeeAttendAPIServer` will be installed and started.
5. Logs can be found in `apiserver/logs/`.

To remove the service, run `uninstall_service.bat` as Administrator.

### Running the Server

```bash
go run .
```

Via the repository orchestrator:
```bash
./leedevkit test api
```
The server listens on `http://127.0.0.1:3000` when started directly.

### API Endpoints

- `GET /health`: Check if the server is running.

### Route ownership

- `apiserver/routes.go` owns router construction and dependency wiring.
- `router` holds public routes: health, login/refresh/password recovery, and report WebSocket upgrade.
- `protectedRouter` holds authenticated routes. Do not add public paths to auth prefix lists; register them on `router` explicitly.
- `registerRoute` preserves `http.Request.PathValue` for existing handlers during migration.

## 🛠 Tech Stack

- **Web Engine**: `github.com/go-chi/chi/v5` with `net/http` handlers and explicit public/protected route ownership
- **API Contract**: `oapi-codegen` v2 (StrictServerInterface compile-time contract enforcement)
- **Database Layer**: `pgx/v5` connection pool + `sqlc` compile-time typed SQL (Zero ORM)
- **Precision Math**: `shopspring/decimal` (Zero floating-point in payroll/hours/money)
- **Cache & Distributed State**: `redis/go-redis/v9`
- **Authentication**: `golang-jwt/jwt/v5` + `argon2id`
## 🧪 Testing

Run API unit, integration, lint, and static-analysis gates through the repository orchestrator:

```bash
./leedevkit test api
```

The test environment runs the Go module under `apiserver/`, applies database fixtures as required, and isolates integration resources in containers.

Integration tests automatically create a fresh database, run migrations, and drop the database after completion.
