# Debugging

The goal is the root cause, found with the fewest wasted cycles, and a regression test so it never returns.

## The procedure

1. **Read the whole error.** Full output, not the last line. Re-run with `RUST_LOG=debug` (or `trace` for the failing module) and `RUST_BACKTRACE=1`.
2. **Reproduce deterministically.** Turn the failure into the smallest failing test or script. If it is intermittent, loop it until you have a reliable rate (see TESTING rule 3).
3. **Locate the layer** (the blame model). Walk down until the fault appears:
   - *Description*: does `dd validate` pass? Does the twin (sim) pass with this description?
   - *Logic*: does it fail without the Wasm module?
   - *Runtime*: does the same description fail on sim on every OS? → runtime or description.
   - *Transport*: does virtual USB (rung 5) fail where sim passes? Capture traffic: Linux `usbmon` (`tcpdump -i usbmon<N> -w cap.pcapng` or Wireshark), Windows USBPcap.
   - *Surface / OS*: fails on only one OS? → OS adapter or OS behaviour. Diff the traces across OSes.
   - *Firmware / hardware*: fails on HIL only? Flash known-good firmware (`firmware/*/known-good` tag), power-cycle the port with `uhubctl`, retry. Check `dmesg`/`log stream`/Event Viewer.
4. **Hypothesise, then test the cheapest first.** Write up to 3 ranked hypotheses in STATE.md ("H1: endpoint 0x81 stalls after reset because ...; test: ..."). Change **one** thing per experiment. Record the result next to each hypothesis.
5. **Regressions → bisect.** If it used to work: `git bisect run <test-command>`.
6. **Fix the cause, not the symptom.** No sleeps to "fix" timing, no retries to hide errors, no catch-and-ignore. If a timing tolerance truly is the issue, derive the value from a datasheet or measured trace and cite it in a comment.
7. **Add the regression test** at the lowest rung that catches it.
8. **Write it up** if it took more than 30 minutes or the cause was non-obvious: `docs/debug/D-NNN-slug.md` (symptom, root cause, how found, fix, test). Future debugging starts by grepping these.

## Strike limits (stop rabbit holes)

- **3 failed hypotheses** → research gate (`/research`): search the exact error text, the library's issue tracker, read the library source around the failing call.
- **5 failed hypotheses, or 2 hours on one bug** → step back. Write `docs/debug/D-NNN` with everything known so far. Then choose (via DECIDING):
  - an alternative approach that avoids the problem (e.g. a different library, a different transfer type), or
  - a minimal spike reproducing the issue outside our code (to report upstream), or
  - mark the task `[!] blocked: D-NNN` and move to another unblocked task. Add a new task "Unblock: D-NNN" to the plan so it is not forgotten.
- **Same bug class 3 times** (e.g. third timing bug in transports) → it is a design problem. Open a two-way/one-way decision about the design, not another patch.

## Suspect hardware last, but do suspect it

Before blaming hardware: known-good firmware, a different USB port/cable (lab nodes have two DUT ports), the twin on rung 5. If those point at hardware, add a human action ("reseat the BME280 breakout on lab-linux, photo of wiring") rather than guessing.

## Tools cheat sheet

| Need | Linux | macOS | Windows |
|---|---|---|---|
| USB tree | `lsusb -t -v` | `system_profiler SPUSBDataType`, `ioreg -p IOUSB -l` | `pnputil /enum-devices /connected`, USBView |
| Kernel log | `dmesg -w` | `log stream --predicate 'subsystem contains "usb"'` | Event Viewer, `Get-WinEvent` |
| USB capture | usbmon + Wireshark/tcpdump | (limited) — capture on Linux/Windows instead | USBPcap + Wireshark |
| Power-cycle port | `uhubctl -l <hub> -p <port> -a cycle` | `uhubctl` (works on macOS) | `uhubctl` or hub vendor tool |
| Flash/debug MCU | `probe-rs run --chip RP235x` | same | same |
| Runtime trace | `RUST_LOG=dd=trace` | same | same |
