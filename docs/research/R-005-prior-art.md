# R-005. Prior art for device descriptions: what DriverDesk borrows and imports
- Date: 2026-10-10 · Stage/task: P0.T1 · Plan question: R-P0-1
- Confidence: high for Linux (source read, R-004); medium-high for others (primary docs via research subagent)
- Answer: Nobody describes a *whole* device (identity + match + transport + registers + sequences + channels + safety) as data. Each format covers one slice. DriverDesk 0.1 composes the best slice of each: **match** from Linux `usb_device_id`/`compatible`, **variants** from Linux `chip_info`, **registers** from regmap + SVD/reggen access vocabulary + `device-driver` fieldsets, **channels** from IIO + Zephyr sensor channels + UCUM, **instance config** from devicetree/Zephyr bindings, **schema conventions** from dt-schema. Safety envelope is new (no prior format has one).

## Summary table
| Format | Describes well | Borrow | Importer priority |
|---|---|---|---|
| **Linux drivers** (regmap, IIO, `*_device_id`, chip_info) — R-004 | match ids, variants, register access flags, init/PM sequences, channel model | USB match field set + flags; `vendor,part` ids; `variants` with id-register check; `readable/writable/volatile/precious/noinc`; IIO RAW/SCALE/OFFSET/PROCESSED; init/suspend/resume as step lists | **High** — facts only (GPL code never copied); used to build `benchmark/` ground truth |
| **Linux DT bindings (dt-schema)** | board-level *instance* of a device; YAML + JSON Schema | `$id` URIs, `additionalProperties: false`, validated `examples:`, vendor-prefix registry, supplies | Medium (bindings are GPL-2.0 OR BSD-2-Clause) |
| CMSIS-SVD | MCU memory-mapped register maps | `access`, `readAction`, `modifiedWriteValues`, `resetValue/resetMask`, `enumeratedValues`, `dim/dimIncrement`, `derivedFrom` | **High** (Apache-2.0, thousands of files) |
| SystemRDL 2.0 | register hierarchy, sw/hw access semantics | `sw`/`hw` split, `onread`/`onwrite` (`rclr`, `woclr`), `singlepulse`, user-defined properties as extension | Medium (compiler MIT; PeakRDL LGPL — not in runtime) |
| IP-XACT 1685-2022 | SoC IP packaging | `vendorExtensions` idea | Low |
| Zephyr bindings + sensor API | instance binding; normalized sensor channels with fixed units | `compatible`, `on-bus`, `include`, typed `properties` with `enum/default`; channel taxonomy (AMBIENT_TEMP, PRESS, HUMIDITY…), triggers, attributes | **High** (channels), Medium (bindings); Apache-2.0 |
| ESPHome YAML | user-facing sensor config | `update_interval`, `filters`, `device_class`, `state_class`, `unit_of_measurement`, `accuracy_decimals`; component list = popularity signal | Medium (YAML side MIT; C++ GPLv3 — never copied) |
| SCPI / Modbus / SunSpec | instrument command trees; scaled register points | SCPI query form for `commands`; SunSpec point: `type`, `units`, `sf`, `access`, `mandatory` | Medium (SunSpec JSON, Apache-2.0), Low (SCPI) |
| USB descriptors / HID report descriptors | self-identification; HID field layout + units | identity triple; HID usage pages as semantic tags; logical/physical min/max + unit exponent | **High** — live import at plug-in (skeleton description) |
| Devil / Termite | typed device variables; driver synthesis | typed variables over registers; ordering constraints; consistency checking | None — lesson: hand-written formal specs died on authoring cost → AI authoring + checker, no full behavioural spec |
| **`device-driver` 2.1.1** (Rust, MIT/Apache-2.0) | I²C/SPI register/command/buffer models | `fieldset` as reusable object; `repeat {count, stride}`; field → enum conversion; `byte-order`, `default-access`, `address-overlap` | **High** — v2.0 (2026-08) replaced YAML/JSON with its own DDSL; import DDSL + v1 manifests. Closest project; roadmap (init routines, state machines) overlaps ours → collaborator *and* overlap risk |
| svd2rust / chiptool | SVD → Rust; YAML registers + transforms | declarative **transforms/patch layer** to fix imported vendor data | Medium (chiptool YAML) |
| OpenTitan reggen (hjson) | silicon register blocks | `swaccess` enum (`ro`, `rw`, `wo`, `rw1c`, `rw1s`, `rc`, `r0w1c`…), `resval`, `multireg`, `window` | Low |

## Decisions for spec 0.1 (feed P0.T2/T3)
1. **Register access**: `access:` uses a reggen-style enum (`ro`, `rw`, `wo`, `rw1c`, `rw1s`, `rc`, `w1`) plus regmap flags `volatile`, `precious`, `noinc`. One vocabulary, mapped to SVD on import.
2. **Fields**: `fields: {name: {bits: "7:5", enum: {...}, reset: ...}}`; reusable `fieldsets:` referenced by registers; `repeat: {count, stride}` for arrays.
3. **Variants**: top-level `variants:` each with `id_check` and overrides (Linux TMP116/117/119 and Bosch BMx80 patterns).
4. **Instance parameters** separate from chip facts: bus address options, straps, supplies (devicetree lesson).
5. **Channels**: class profiles use IIO's raw/scale/offset model; channel names map to Zephyr channels and HID usages; units in UCUM (not fixed milli-units).
6. **Sequences**: `init`, `configure`, `suspend`, `resume` as ordered step lists (write, read+expect, wait) inside the state machine.
7. **Expressions**: CEL + DriverDesk function library (`shl`, `shr`, `band`, `bor`, `bxor`, `sign_extend`, `clamp`, `be16/le16/be24`) because standard CEL has no bitwise ops and errors on overflow (R-004).
8. **Schema conventions**: dt-schema style — `$id` URIs, closed objects, every example validated.
9. **Importers** (later stages, in order): Linux facts (benchmark), SVD, live USB/HID descriptors, `device-driver` DDSL, chiptool YAML, SunSpec.
10. **Never copy** GPL code (Linux, ESPHome C++). Facts (addresses, ids, bit positions) with a source citation only.

## Evidence
- Linux: R-004 (torvalds/linux @ 3857c2fe5449).
- CMSIS-SVD https://open-cmsis-pack.github.io/svd-spec/ · SystemRDL https://www.accellera.org/downloads/standards/systemrdl, https://github.com/SystemRDL/systemrdl-compiler · IP-XACT https://standards.ieee.org/ieee/1685/10583/
- Zephyr sensor channels https://github.com/zephyrproject-rtos/zephyr/blob/main/include/zephyr/drivers/sensor.h · ESPHome https://github.com/esphome/esphome · SunSpec https://github.com/sunspec/models · HID https://www.usb.org/hid
- Devil https://www.usenix.org/legacy/events/osdi2000/merillon.html · Termite https://dl.acm.org/doi/10.1145/1629575.1629583
- device-driver https://github.com/diondokter/device-driver (CHANGELOG 2.0.0 2026-08-21, 2.1.1), https://device-driver.com/book/v2/intro.html
- chiptool https://github.com/embassy-rs/chiptool · svd2rust https://github.com/rust-embedded/svd2rust · reggen https://opentitan.org/book/util/reggen/

## Implications
- P0 plan updated (P0.T0): §4 adds variants, new instance-parameters section, R-P0-2 default gains the bitwise function library.
- Outreach idea (R1): contact `device-driver` maintainer about interop — draft only, human sends.

## Re-check when
device-driver 3.x · Zephyr sensor API changes · before writing importers.
