# Brainstorm: Rust Test Execution Model

**Topic:** Test Binary Structure & Parallelism
**Date:** 2026-02-05

## Question
After refactoring 111 test files into `tests/integration/main.rs`, does running tests with multiple workers/threads still imply a single binary?

## Conclusion
**YES.** usage of multiple workers (threads) does not create multiple binaries.

## Analysis

### 1. The Build Artifact (Single Binary)
- By consolidating all tests into `tests/integration/main.rs`, you have defined a **single integration test target** for Cargo.
- When you run `cargo test --test main`, `rustc` compiles this single entry point (and all its submodules) into **one executable** file.
- This file is located at `target/debug/deps/integration-<hash>`.
- **Contrast**: Previously, `cargo test` had to compile 111 separate binaries (one for each `.rs` file in `tests/`), resulting in 111 link steps.

### 2. Runtime Behavior (Multiple Workers)
- The compiled binary contains a built-in test harness (libtest).
- When executed, this single process spawns a **thread pool**.
- The "workers" you see or configure (e.g., via `RUST_TEST_THREADS` or Cargo's default) are **threads** within this *single process*.
- They are **not** separate processes or separate binaries.

### 3. Implications
| Feature | Old Model (Many Files) | New Model (Single Binary) |
| :--- | :--- | :--- |
| **Compilation** | 111 Binaries | 1 Binary |
| **Linking Time** | Very High (111x overhead) | Minimal (1x overhead) |
| **Execution** | 111 Processes (Process-level parallel) | 1 Process (Thread-level parallel) |
| **Resource Usage** | High (Process overhead) | Low (Shared memory) |
| **Shared State** | Impossible (Isolated processes) | Possible (Common `lazy_static` / `OnceLock`) |

### 4. Summary
Refactoring to a single binary fundamentally changes the architecture from "Multi-Process" to "Multi-Threaded Single-Process". This is why you see massive performance gains in compilation and linking. The parallelism remains, but it is now lighter and more efficient.
