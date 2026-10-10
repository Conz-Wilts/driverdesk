# 0012. USB identifiers for project fixtures

- Status: accepted · Date: 2026-10-10 · Class: two-way · Review trigger: R1 (apply for our own PIDs)

## Decision
Lab fixtures use pid.codes test identifiers `1209:0001`–`1209:000F` (reserved for testing). Never reuse IDs that an existing kernel driver matches (e.g. i2c-tiny-usb's IDs), so no OS driver binds to our fixtures. In R1, request dedicated PIDs (pid.codes for open hardware, or Raspberry Pi's PID allocation for RP-based projects).
