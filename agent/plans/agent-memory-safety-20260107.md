# Implementation Plan: Agent Memory Safety & Stability

**Task:** Prevent memory leaks and high resource consumption during 24/7 operation.
**Mode:** HARD (Stability/Performance)

## Root Cause Analysis
1.  **Unbounded Queue:** `asyncio.Queue()` has no limit. If the DB/Network is slower than the device polling (e.g., during network outage), the queue grows indefinitely until OOM.
2.  **Full Fetch Spikes:** `ZKService.get_attendance_logs()` fetches *all* logs (potentially 100k+) every 10 seconds. This creates massive temporary objects and CPU churn.
3.  **Inefficient Deduplication:** The `processed_logs` set grows until 20k, then arbitrarily cuts 10k. This "bulk chop" might remove recent logs, causing re-processing of old logs on the next fetch.

## Implementation Steps

### Step 1: Bounded Queue & Backpressure (Core Fix)
**Goal:** Prevent the queue from eating all RAM.
- **File:** `agent/app_container.py`
- **Action:**
    - Initialize `self.log_queue = asyncio.Queue(maxsize=1000)`.
- **File:** `agent/device_manager.py`
- **Action:**
    - Change `self.log_queue.put_nowait(...)` to `await self.log_queue.put(...)` in `_monitor_device`.
    - This creates natural backpressure: if the consumer (DB saver) is slow, the producer (poller) will pause.

### Step 2: Smart Polling (Adaptive Interval)
**Goal:** Reduce CPU/RAM stress when the device has too much data.
- **File:** `agent/device_manager.py`
- **Logic:**
    - Default Interval: `10s`.
    - If `fetched_count > 5000`: Increase interval to `60s`.
    - If `fetched_count > 20000`: Increase interval to `300s`.
    - Log a warning suggesting the user to clear data on the device.

### Step 3: Optimized Deduplication (Rolling Window)
**Goal:** Stable memory usage for the deduplication cache without "forgetting" recent logs.
- **File:** `agent/device_manager.py`
- **Refactor:**
    - Replace `processed_logs: Set` with a combination of `Set` (fast lookup) and `Deque` (ordering).
    - **Logic:**
      ```python
      # On new log
      if log_id not in self.cache_set:
          self.cache_set.add(log_id)
          self.cache_deque.append(log_id)
          
          # Maintain size
          while len(self.cache_deque) > 20000:
              old_id = self.cache_deque.popleft()
              self.cache_set.discard(old_id)
      ```
    - This ensures we always keep the *most recent* 20,000 logs in memory, preventing re-processing loops.

### Step 4: Add Resource Monitoring (Optional)
- **File:** `agent/app_container.py`
- **Action:** Add a simple log in `get_agent_status` showing current queue size.

## Testing Plan (New Test File: `tests/test_memory_safety.py`)
1.  **Test Queue Backpressure:**
    - Mock a slow consumer.
    - Flood producer.
    - Assert producer blocks or waits.
2.  **Test Adaptive Interval:**
    - Mock `ZKService` returning 10,000 logs.
    - Assert next sleep time is > 10s.
3.  **Test Deduplication Logic:**
    - Feed 25,000 unique logs.
    - Assert cache size stays at 20,000.
    - Assert oldest 5,000 are removed.

## Timeline
| Phase | Duration |
|-------|----------|
| Queue & Backpressure | 10 min |
| Smart Polling | 15 min |
| Deduplication Refactor | 15 min |
| Testing | 20 min |
| **Total** | **1 hour** |

## Verification
- Run `pytest tests/test_memory_safety.py`.