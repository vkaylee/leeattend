# LeeAttend API Server (Go)

Central high-performance API server for LeeAttend, built with Go 1.26+, standard library `net/http.ServeMux`, `oapi-codegen` strict server, `pgx/v5` connection pool, and `sqlc`.

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
### 🏃 Running as Windows Service

1. Copy `nssm.exe` to the `apiserver/` directory.
2. Build the project in release mode: `cargo build --release`.
3. Right-click `install_service.bat` and select **Run as Administrator**.
4. The service `LeeAttendAPIServer` will be installed and started.
5. Logs can be found in `apiserver/logs/`.

To remove the service, run `uninstall_service.bat` as Administrator.

### Running the Server

#### Development mode (with auto-reload)



```bash
go run .
```

Via devkit orchestrator:
```bash
./leedevkit test api-go
```
The server will start at `http://127.0.0.1:3000`.

### API Endpoints

- `GET /health`: Check if the server is running.

## 🛠 Tech Stack

- **Web Engine**: Standard Library `net/http.ServeMux` (Go 1.22+ method-and-path routing)
- **API Contract**: `oapi-codegen` v2 (StrictServerInterface compile-time contract enforcement)
- **Database Layer**: `pgx/v5` connection pool + `sqlc` compile-time typed SQL (Zero ORM)
- **Precision Math**: `shopspring/decimal` (Zero floating-point in payroll/hours/money)
- **Cache & Distributed State**: `redis/go-redis/v9`
- **Authentication**: `golang-jwt/jwt/v5` + `argon2id`
## 🧪 Testing

The project uses a **Dynamic Test Database** pattern for integration tests to ensure isolation and allow parallel execution.

### Prerequisites
- Postgres server running.
- `DATABASE_URL_FOR_TEST_GENERATION` environment variable set in `.env.test` pointing to a user with `CREATEDB` permissions.

### Running Tests
```bash
# Run all tests in parallel
cargo test

# Run a specific test suite
cargo test --test health_check
```

Integration tests automatically create a fresh database, run migrations, and drop the database after completion.
