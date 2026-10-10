# R-001. Landscape: prior attempts, Linux precedents, tooling (Oct 2026)

- Date: 2026-10-10 · Stage: planning · Confidence: medium-high (secondary sources for some facts)
- Answer: The space is open (no company shipping cross-OS AI-drafted drivers). Prior art proves the pieces work individually (Devil, Termite, HID-BPF, WASI drivers) and the business model has a template (LVFS). The main historical failure mode is requiring OS adoption up front (UDI).

## Evidence
- UDI (1998–2004) died on adoption: Microsoft/Apple never joined; Linux community feared binary drivers; became SCO-specific. https://en.wikipedia.org/wiki/Uniform_Driver_Interface
- Devil IDL (OSDI 2000): declarative register descriptions, compiler checks; specs up to 5.9× less error-prone than C. https://static.usenix.org/event/osdi00/full_papers/merillon/merillon_html/index.html
- Termite (SOSP 2009) / Termite-2 (OSDI 2014): driver synthesis from specs; v1 output unmaintainable, v2 user-guided. https://www.eecg.utoronto.ca/~stumm/Papers/Ryzhyk-OSDI-14.pdf
- HID-BPF + udev-hid-bpf: Linux ships per-device sandboxed fixes as data; regular syncs in 2025 (6.15, Nov 2025 batch). https://libevdev.pages.freedesktop.org/udev-hid-bpf/
- LVFS: 179 vendors, 135M+ updates (Oct 2025); sponsor-funded (Dell, Lenovo, NVIDIA, HP). https://phoronix.com/news/LVFS-Fwupd-135-Million-Download
- Fuchsia DFv2: driver hosts as components; still no stable driver ABI. https://fuchsia.dev/docs/concepts/drivers/driver_framework
- WASI drivers (arXiv 2410.22919, NOMS 2025): USB/I²C drivers in Wasm, ≤ 8% USB overhead. https://arxiv.org/abs/2410.22919v2
- Rust in Linux permanent (Dec 2025 summit; formalised in 7.0 merge window). https://www.phoronix.com/news/Rust-To-Stay-Linux-Kernel
- Kernel AI policy (7.0) requires human accountability; staging rejects LLM-generated patches (Aug 2026); wireless maintainer strict. https://phoronix.com/news/Linux-Staging-Reject-LLMs
- Linux 7.3-rc1 ≈ 41M lines; AMD GPU driver ≈ 16%. https://phoronix.com/news/Linux-7.3-rc1-Code-Stats
- DevGen (ACL 2026 demo): LLMs generate QEMU device models from Linux drivers (reverse direction; relevant for our twins). https://aclanthology.org/2026.acl-demo.24/
- WinUSB auto-install via MS OS descriptors, no INF. https://learn.microsoft.com/windows-hardware/drivers/usbcon/automatic-installation-of-winusb
- macOS: libusb/IOKit can't claim kernel-claimed interfaces (exclusive access); DriverKit needs entitlements. https://developer.apple.com/forums/thread/124700
- raw-gadget + dummy_hcd not available on stock GitHub runner kernels; hidapi CI uses virtme-ng with a generic Ubuntu kernel. (lead: hidapi PR #815)
- nusb: pure-Rust USB for Linux/macOS/Windows/WebUSB; 0.2.7 latest seen; active in 2026. https://docs.rs/crate/nusb/latest

## Implications
ADRs 0001–0012; plan stages B–P7. Never require an OS change before delivering value. Use HID-BPF and LVFS as the reference story with Linux maintainers.

## Re-check when
Before R1 (market scan), before R3 (kernel policy).
