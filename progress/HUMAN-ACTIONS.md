# Human actions

Things only a person can do (physical, money, legal, identity). Claude has already made every decision; each item says exactly what to do. Do the **open** items in any order; tell Claude "done H-00x" (or just run `/next` — it checks).

Rules for Claude: batch requests; give copy-pasteable commands; never ask for secrets in chat (give a command that stores them); say what each item unblocks; mark items `done` with the date when verified.

| ID | Status | What | Unblocks |
|---|---|---|---|
| H-001 | **open** | Buy lab hardware (list below, ~NZ$1,800) | B.T7, B.T8, P1 HIL |
| H-002 | done 2026-10-10 | GitHub home: public repo `Conz-Wilts/driverdesk` (owner's choice, no org) | B.T5 (CI) |
| H-003 | pending (B.T7) | Set up Raspberry Pi 5 lab node (flash card, plug in, run one command) | HIL Linux |
| H-004 | pending (B.T7) | Set up Windows mini PC lab node (first boot, run one command) | HIL Windows |
| H-005 | pending (B.T7) | Create macOS lab-runner user on your Mac, allow USB accessories | HIL macOS |
| H-006 | pending (P3) | Anthropic API key + spend cap | P3 AI drafter |
| H-007 | pending (R1) | Production key ceremony (hardware security keys) | R1 |
| H-008 | pending (P5) | Buy HID demo device | P5 |
| H-009 | pending (P5) | Press 3 buttons on the HID device during a watched test | P5 gate |
| H-010 | pending (P5) | Grant macOS Accessibility to `dd-agent` | P5 |
| H-011 | pending (P6) | Ask a lawyer to review the reverse-engineering checklist | P6 publishing |
| H-012 | pending (P6) | Buy trace target device + USB webcam | P6 |

---

## H-002 — GitHub home for the project (done 2026-10-10)

Owner chose a public repo on their personal account: https://github.com/Conz-Wilts/driverdesk. Existing `gh` scopes (`repo`, `workflow`) cover repo creation, branch protection and repo-level self-hosted runners; no org needed.

---

## H-001 — Buy the lab hardware (open)

Everything below is already chosen; just order it. Prices are NZD incl. GST, checked 2026-10-10 (they drift — anything within ~15% is fine). If an item is out of stock, use the listed alternative; if none is listed, tell Claude and it will pick a replacement.

**Ship to:** wherever the lab will live (needs mains power for the Pi 5, the mini PC, and 3 powered hubs; wired Ethernet or Wi-Fi for both).

### Order 1 — Digi-Key NZ (https://www.digikey.co.nz, free delivery over NZ$66)

| Qty | Item | Digi-Key part / link | Unit | Line | Why |
|---|---|---|---|---|---|
| 3 | Raspberry Pi Pico 2 **H** (headers soldered) | SC1632 — https://www.digikey.co.nz/en/products/detail/raspberry-pi/SC1632/26241102 | 13.03 | 39.09 | the device under test, one per lab node |
| 2 | Raspberry Pi Debug Probe | SC0889 — search "raspberry pi debug probe" | 26.06 | 52.12 | flash + read logs from the Pico automatically |
| 1 | Raspberry Pi 5 **8 GB** | SC1432 | 380.08 | 380.08 | the Linux lab node (`lab-linux`) |
| 1 | Raspberry Pi 5 Active Cooler | SC1148 | 10.86 | 10.86 | keeps the Pi stable under CI load |
| 2 | Adafruit BME280 sensor (STEMMA QT) | 1528-2652-ND — https://www.digikey.co.nz/en/products/detail/adafruit-industries-llc/2652/5604372 | 32.46 | 64.92 | first real I²C sensor driver (P1) |
| 2 | Adafruit TMP117 sensor (STEMMA QT) | 1528-4821-ND | 24.98 | 49.96 | second sensor, proves the driver format is generic |
| 4 | Adafruit STEMMA QT → male header cable | Adafruit 4209 | 2.06 | 8.24 | connects the sensors to the Pico |
| 2 | 400-point breadboard | Schmartboard 201-0015-01 | 11.95 | 23.90 | holds the Pico + LEDs |
| 1 | Jumper wires M/M 6" | SparkFun 12795 | 6.41 | 6.41 | wiring |
| 5 | 3 mm red LED | Kingbright WP710A10ID | 0.48 | 2.42 | simple actuator to test writes |
| 5 | 330 Ω resistor | Stackpole CF14JT330R | 0.22 | 1.09 | current limiting for the LEDs |
| 4 | USB-C ↔ USB-C cable 1 m | Adafruit 4199 | 21.61 | 86.44 | hubs/Pi/PC connections |
| | | | | **≈ 725.53** | |

### Order 2 — PB Tech (https://www.pbtech.co.nz)

| Qty | Item | PB Tech code | Unit | Line | Why |
|---|---|---|---|---|---|
| 3 | **StarTech HB30A4AIB** 4-port USB 3.0 hub (box must say HB30A4AIB) | HUBSTT4283767 | ~71.00 | 213.00 | per-port power switching, so CI can power-cycle the Pico (research R-002) |
| 1 | Raspberry Pi official 27 W USB-C PSU, AU plug | SEVRBP0511 | ~24.90 | 24.90 | Pi 5 power |
| 1 | Raspberry Pi official A2 64 GB microSD | SEVRBP0537 (alt: any SanDisk/Samsung A2 64 GB) | ~45.23 | 45.23 | Pi 5 OS disk |
| 4 | Raspberry Pi official micro-USB → USB-A cable | SEVRBP0365 | ~3.57 | 14.28 | Pico 2 and Debug Probe are micro-USB |
| 1 | USB-C (male) → USB-A (female) adapter, any brand | search "USB-C to USB-A adapter" | ~15.00 | 15.00 | the hub plugs into your Mac if it only has USB-C |
| 1 | **MINIX Z150-0dB** mini PC, Intel N150, 16 GB, 512 GB, Windows 11 Pro | IPCMNXZ15001 | ~744.00 | 744.00 | the Windows lab node (`lab-windows`); fanless, runs 24/7 |
| | delivery | | | ~10.00 | |
| | | | | **≈ 1,066.41** | |

Mini PC alternative if the MINIX is out of stock or over ~NZ$850: **Beelink EQ14** (N150 / 16 GB / 500 GB) from https://www.bee-link.com/products/beelink-eq14-n150 (~NZ$665, ships from China, slower).

### Total ≈ NZ$1,792

### When it arrives
Don't set anything up yet: just tell Claude "hardware arrived" (or run `/next`). Claude will then write H-003/H-004/H-005 with one command per machine. Keep the physical on/off switches on each hub **ON**.
