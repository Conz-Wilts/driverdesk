# P6 — Works without the vendor: traffic → working driver

- Status: not started
- Depends on: P3 gate, P1 gate
- Branch: `stage/p6`
- Estimate: 3–4 weeks

Claim: from captured USB traffic between a vendor's Windows app and a device (plus a log of what was done in the app), the AI produces a description that makes the device work on Linux.

## Exit criteria (the gate)

1. Target device (R-P6-2) works on `lab-linux` from a description produced by the trace pipeline: every feature exercised in the capture session reproduces on Linux (verified by the device's responses and, where visual, by the lab webcam check in P6.T7).
2. The AI never saw community reverse-engineering notes for the device during the run (**blind**): pipeline inputs are logged and checked; afterwards the result is compared with the community protocol docs and agreement is scored per message type.
3. `dd trace import <pcapng>` handles usbmon (Linux) and USBPcap (Windows) captures; `dd trace annotate` aligns an action log (timestamped JSON from the automation script) with transfers.
4. Deterministic helpers exist and are used by the AI as tools: field-diff across messages, checksum finder (searches the CRC catalogue + sum/xor variants over candidate ranges), counter/sequence detector, length-field detector.
5. A Wireshark Lua dissector is generated from the final description and decodes the original capture (snapshot test of `tshark -X lua_script:... -V` output).
6. Legal checklist completed before capture (R-P6-1) and linked in the report.

## Pre-made decisions

- Capture where the vendor app runs: `lab-windows` with USBPcap; the vendor app is driven by a script (`pywinauto`, or Claude computer-use on the Windows node via RDP if the UI cannot be scripted) that writes an action log with timestamps.
- Only harmless, USB-powered, low-energy targets (no mains relays, no heaters, no motors).
- Pipeline = deterministic analysis tools + LLM hypothesis generation + bench-agent replay to confirm. LLM proposes, tools and hardware verify.

## Research questions

- **R-P6-1** Legal: interoperability reverse engineering in New Zealand (Copyright Act 1994 s 80A–80C decompilation/interoperability provisions?), US DMCA §1201(f), EU Software Directive Art. 6; vendor EULA terms for the chosen app. · needed by P6.T1 · Default: capture and analysis of our own device's traffic for interoperability is permitted; no decompiling the app; no circumvention of encryption; if the EULA forbids reverse engineering, choose a different device. Flag for counsel (H-011) before publishing anything about a commercial device. · Status: open
- **R-P6-2** Target device: cheap (< US$40), USB-powered, Windows-only vendor app, no Linux support in mainline, simple protocol (likely HID or vendor bulk), community RE exists (for blind scoring), visible output. · Default candidates: USB LED name badge (11×44, "LED Badge" family), USB RGB light controller. · Status: open
- **R-P6-3** Automating the vendor app on Windows: pywinauto feasibility; computer-use fallback. · Default: pywinauto. · Status: open
- **R-P6-4** pcapng parsing in Rust (`pcap-file`) and USB link types (220 usbmon mmapped, 249 USBPcap). · Default: `pcap-file` + own USB header decoders. · Status: open

## Tasks

- [ ] **P6.T0 Re-validate this plan.**
- [ ] **P6.T1 Legal checklist + device choice** (R-P6-1, R-P6-2) — `docs/legal/P6-checklist.md`; H-012 to buy the device; H-011 counsel flag (non-blocking for internal research, blocking for publishing).
- [ ] **P6.T2 Capture tooling** (R-P6-4) — import, normalise to `transfers.jsonl`, annotate with action log.
- [ ] **P6.T3 Vendor-app automation** (R-P6-3) — scripted sessions covering each feature at least 3 times with varied parameters (text, colours, brightness).
- [ ] **P6.T4 Deterministic analysis tools** — exit criterion 4; unit tests with synthetic protocols (known answers).
- [ ] **P6.T5 AI protocol inference** — agent loop (in `ai/trace/`) with the tools from P6.T4 as function tools; outputs a hypothesis description + evidence (which captures support each field).
- [ ] **P6.T6 Replay verification** — bench agent replays hypothesised commands on `lab-linux` within conservative limits, compares responses; iterates ≤ 5 rounds.
- [ ] **P6.T7 Visual check** (optional but planned) — a USB webcam on lab-linux pointed at the device (H-012 includes it); a vision model confirms "badge shows text HELLO" style assertions; results stored as images in the report.
- [ ] **P6.T8 Dissector generator** — exit criterion 5.
- [ ] **P6.T9 Blind scoring + report + gate** — exit criteria 1–2, `/gate P6`.

## Risks and debugging hints

- **Encrypted/obfuscated protocols**: abort and pick another device (do not attempt circumvention).
- **Firmware update traffic** in captures: never replay it. The analysis tools must classify and quarantine suspected bootloader/DFU traffic.
- **Small sample sizes**: vary one parameter at a time in automation scripts so field-diffing works.

## Human actions

H-011 (legal counsel flag), H-012 (buy target device + webcam), possibly H-013 (install vendor app on lab-windows if it needs an account/licence acceptance).

## Gate procedure

`/gate P6`, report, merge, tag `gate/P6`.
