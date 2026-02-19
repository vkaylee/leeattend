# Debug Report: rustzk attendance-logs failure

## Bug Characterization
| Attribute   | Value          |
| ----------- | -------------- |
| Description | `get_attendance` fails with "Buffered response size exceeds maximum" |
| Severity    | Critical (Blocks attendance fetching) |
| Reproduction | `cargo run --example debug_zk` |

## Hypotheses

### Hypothesis 1: Protocol Parsing Misalignment (Transport Independent)
- **Statement**: The library misinterprets the payload of the `_CMD_PREPARE_BUFFER` response, reading bytes at the wrong offset as the buffer size.
- **Evidence FOR**: The same huge size `1784955226` (`0x6A6B111A`) occurs in both UDP and TCP.
- **Evidence AGAINST**: None.
- **Test**: Add logging to `rustzk/src/lib.rs` to print the raw payload of `_CMD_PREPARE_BUFFER` response.
- **Likelihood**: Critically High

### Hypothesis 2: UDP Junk Data / Concurrent Packets
- **Statement**: The device sends multiple packets or some "heartbeat" junk that the library doesn't skip, leading to the parser starting at the wrong byte.
- **Evidence FOR**: UDP is stateless and the device might be chatty.
- **Evidence AGAINST**: The connection is newly established.
- **Test**: Use `tcpdump` or `wireshark` (if possible) or add logging to raw packet reads in `rustzk`.
- **Likelihood**: Medium
