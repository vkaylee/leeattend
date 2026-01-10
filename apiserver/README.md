# LeeAttend API Server (Rust)

Central high-performance API server for LeeAttend, built with Rust and Axum.

## 🚀 Getting Started

### Prerequisites

- **Rust 1.75+**: Install via [rustup.rs](https://rustup.rs/)
- **Cargo**: Rust package manager (installed with Rust)

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
   cargo build
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
We recommend using `cargo-watch` for development:
```bash
# Install cargo-watch if you haven't
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
```

#### Standard run
```bash
cargo run
```

The server will start at `http://127.0.0.1:3000`.

### API Endpoints

- `GET /health`: Check if the server is running.

## 🛠 Tech Stack

- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Runtime**: [Tokio](https://tokio.rs/)
- **Serialization**: [Serde](https://serde.rs/)
- **Logging**: [Tracing](https://github.com/tokio-rs/tracing)

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
