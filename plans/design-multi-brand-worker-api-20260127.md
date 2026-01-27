# Implementation Plan: Multi-Brand Worker API

Standardize the interface between the Edge Orchestrator (agent-main) and brand-specific sidecars (agent-zk-worker, etc.) using JSON-RPC 2.0.

## Approach
- **Standardized Contract**: Implement `device.*` and `common.*` namespaces for all workers.
- **Generic Client**: Refactor `agent-main` to use a generic `WorkerClient` instead of brand-specific logic.
- **Parallel Polling**: Use `tokio` to poll multiple devices concurrently.

## Steps
1. **Define API**: Standardize JSON-RPC 2.0 methods (`device.connect`, `device.get_logs`, etc.).
2. **Refactor Orchestrator**: Generic `WorkerClient` in Rust.
3. **Standardize Sidecars**: Update `agent-zk-worker` to match the new contract.
4. **Add Simulation**: Create `agent-mock-worker` for multi-brand testing.
