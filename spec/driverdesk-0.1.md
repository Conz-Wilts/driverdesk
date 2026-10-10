# DriverDesk Device Description Specification, version 0.1

- Identifier: `driverdesk/0.1`
- Status: **Draft** (2026-10-10). Sections marked **[provisional]** may change incompatibly in 0.2; all other sections follow the compatibility promise in §18.
- Licence: CC-BY-4.0 (see `spec/LICENSE`). A royalty-free patent pledge will accompany 1.0.
- Canonical schema: `spec/schema/0.1/` (JSON Schema 2020-12). Where this text and the schema disagree, file an issue; the schema wins for validation, this text wins for meaning.

---

## 1. Scope and goals

A **device description** is a single YAML document that states everything a host needs to drive one kind of hardware device: how to recognise it, how to talk to it, what its registers and commands are, how to bring it up and put it to sleep, what values it produces in which units, which actions are unsafe, and how to test it.

Goals:

1. **Data, not code.** A description is declarative. Imperative behaviour is limited to ordered step lists (§9) and side-effect-free expressions (§10). Anything that does not fit is delegated to a sandboxed logic module (§21) — never to native code shipped with the description.
2. **Target-neutral** (ADR-0013). A description MUST NOT assume a particular execution target. The same description can be interpreted by the DriverDesk runtime, compiled to a native driver (kernel module, DriverKit, UMDF, microkernel driver), or drive a kernel-bypass data path.
3. **Portable across operating systems.** Nothing in a description is OS-specific. OS integration happens through class profiles (§15) implemented by each OS surface.
4. **Checkable.** Every field has a type; every expression is type- and unit-checked; every safety rule is enforceable by the runtime before bytes reach the bus.
5. **Familiar.** Vocabulary follows Linux (regmap, IIO, devicetree, modalias), USB and HID specifications, and register-description formats (SVD, SystemRDL, reggen) so that kernel and firmware engineers recognise it (research R-004, R-005).

Non-goals for 0.1: describing the host side, firmware *updating* (integrate with LVFS/fwupd), and full behavioural verification.

## 2. Conformance language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in BCP 14 [RFC 2119] [RFC 8174] when, and only when, they appear in all capitals, as shown here.

Conformance classes:

- A **conforming description** satisfies the schema (`spec/schema/0.1/description.json`) and every rule in this text marked MUST.
- A **conforming validator** reports every schema violation and every lint rule of severity `error` in Appendix B.
- A **conforming runtime** executes descriptions with the semantics given here and enforces §14 (safety) without exception. A runtime that cannot honour a section a description uses MUST refuse to load the description and name the section.

## 3. Document structure and versioning

A description is one YAML 1.2 document (UTF-8, no BOM). Authors SHOULD use block style. Comments are allowed and are not significant.

Descriptions MUST mean the same thing under YAML 1.1 parsers, which many tools still use. Therefore the plain (unquoted) scalars `y`, `Y`, `yes`, `Yes`, `YES`, `n`, `N`, `no`, `No`, `NO`, `on`, `On`, `ON`, `off`, `Off`, `OFF` MUST NOT be used as keys or as string values; quote them or choose another word (`DD-L027`). Integers with a leading zero (`010`) MUST NOT be used (`DD-L028`); write `0o10`.

Top-level keys:

| Key | Required | Section | Stability |
|---|---|---|---|
| `spec` | yes | §3 | stable |
| `device` | yes | §4 | stable |
| `variants` | no | §4.3 | stable |
| `instance` | no | §4.4 | stable |
| `match` | yes | §5 | stable |
| `transport` | yes | §6 | stable (usb, serial, i2c, hid, bridge); provisional (spi, pci) |
| `types` | no | §7.4 | stable |
| `registers` | no | §8 | stable |
| `vars` | no | §9.6 | stable |
| `commands` | no | §9 | stable |
| `sequences` | no | §12.2 | stable |
| `settings` | no | §12.4 | stable |
| `states` | no | §12 | stable |
| `timing` | no | §13 | stable |
| `safety` | conditional (§14.1) | §14 | stable |
| `events` | no | §20.1 | stable |
| `expose` | yes | §15 | stable |
| `provides` | no | §6.7 | stable |
| `tests` | yes | §16 | stable |
| `visibility` | no | §17 | stable |
| `memory` | no | §20.3 | **provisional** |
| `queues` | no | §20.4 | **provisional** |
| `interrupts` | no | §20.2 | **provisional** |
| `streams` | no | §20.5 | **provisional** |
| `firmware` | no | §20.6 | **provisional** |
| `logic` | no | §21 | **provisional** |
| `hints` | no | §22 | **provisional** |
| `x-*` | no | §3.3 | — |

Unknown top-level keys other than `x-*` MUST be rejected.

### 3.1 `spec`

`spec` (string, REQUIRED) — the specification identifier, `driverdesk/<major>.<minor>`. For this version it MUST be `driverdesk/0.1`.

```yaml
spec: driverdesk/0.1
```

A validator that does not know the version MUST report `DD-L010` and, where it knows a newer version, SHOULD suggest an upgrade path.

### 3.2 Names

Unless stated otherwise, every user-chosen name (commands, registers, fields, variables, states, channels, settings, endpoints, events, tests) MUST match `^[a-z][a-z0-9_]*$` and be at most 64 characters. Register names MAY additionally use upper case (`^[A-Za-z][A-Za-z0-9_]*$`) because datasheets do. Names are case-sensitive and unique within their section.

References between sections use **paths**: `command`, `command.output_name`, `command.arg_name`, `REGISTER`, `REGISTER.field`.

### 3.3 Extensions

Keys beginning with `x-` (for example `x-acme-calibration-notes`) MAY appear at the top level and inside any object whose schema says `extensible`. Runtimes MUST ignore extensions they do not understand. Extensions MUST NOT change the meaning of standard fields. Safety-relevant behaviour MUST NOT be expressed in extensions.

### 3.4 Numbers

Integers MAY be written in decimal, hexadecimal (`0x1F`), octal (`0o17`) or binary (`0b1010`) as YAML 1.2 allows; validators MUST accept all four and SHOULD preserve the author's form in messages. Underscores are not allowed. Floating-point values use YAML floats.

### 3.5 Parametric values

Some integer fields (marked *parametric* in the schema) MAY instead be an expression object `{cel: "<expression>"}` evaluated once at bind time against `instance` and `variant` (§4.3, §4.4).

```yaml
transport:
  i2c:
    address: { cel: "instance.address" }
```

---

## 4. Identity

### 4.1 `device`

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `id` | string | yes | Globally unique description id: lower-case dot-separated, first label is the vendor prefix. `^[a-z0-9][a-z0-9-]*(\.[a-z0-9][a-z0-9-]*)+$` |
| `name` | string | yes | Human name of the device. |
| `vendor` | object | yes | `{ name: string (req), url: uri, prefix: string }`. `prefix` is the devicetree-style vendor prefix (`bosch`, `ti`). |
| `version` | string | yes | Semantic version of **this description** (not the device firmware). |
| `classes` | list of string | yes | Class profiles this description claims (§15). At least one. |
| `compatible` | list of string | no | Devicetree-style `vendor,part` strings this description serves, most specific first. Used for non-enumerating buses (§5.4). |
| `description` | string | no | Markdown. |
| `datasheets` | list of object | no | `{ url: uri (req), sha256: hex64 (req), title: string, revision: string }`. Documents are referenced, never embedded. |
| `license` | string | no | SPDX expression for the description text. RECOMMENDED: `CC0-1.0` or `Apache-2.0`. |
| `maintainers` | list of string | no | Names or `Name <email>`. |
| `sources` | list of object | no | Provenance of facts: `{ kind: datasheet|linux|capture|vendor|other, ref: string, note: string }`. |

```yaml
device:
  id: ti.tmp117
  name: TMP117 high-accuracy digital temperature sensor
  vendor: { name: Texas Instruments, prefix: ti, url: https://www.ti.com }
  version: 0.1.0
  classes: [sensor.temperature]
  compatible: ["ti,tmp117", "ti,tmp116"]
  datasheets:
    - url: https://www.ti.com/lit/gpn/tmp117
      sha256: "0000000000000000000000000000000000000000000000000000000000000000"
  license: CC0-1.0
```

### 4.2 Description identity and signing

The registry identifies a published description by the SHA-256 of its canonical JSON form (Appendix C). The `safety` block carries its own signature (§14.6).

### 4.3 `variants`

A family of chips that differ in small ways SHOULD be one description with variants (Linux `chip_info` pattern, R-004).

`variants` is a map from variant name to:

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `name` | string | yes | Human name (`TMP116`). |
| `id_check` | object | no | How to detect this variant at bind time (below). |
| `exclude` | list of path | no | Commands, outputs, channels, settings or registers that this variant does **not** have. |
| `params` | map name → scalar | no | Constants visible to expressions as `variant.<name>`. Every variant MUST define the same parameter names. |
| `default` | bool | no | Use when no `id_check` matches and the match entry did not name a variant. At most one variant. |

`id_check` is one of:

- `{ register: NAME, equals: int | [int...], mask: int }` — read register `NAME` (§8), apply optional `mask`, compare.
- `{ command: NAME, output: name, equals: scalar | [scalar...] }` — call a read-only command.
- `{ usb_descriptor: { bcd_device: {min, max} } }` — compare against the enumerated device.

Detection order: variants are checked in document order; the first match wins. If none match and no `default` exists, the runtime MUST fail binding with layer `description`.

```yaml
variants:
  tmp116: { name: TMP116, id_check: { register: DEVICE_ID, equals: 0x1116, mask: 0x0FFF }, exclude: [set_offset, temp.calibbias] }
  tmp117: { name: TMP117, id_check: { register: DEVICE_ID, equals: 0x0117, mask: 0x0FFF }, default: true }
```

### 4.4 `instance`

Facts about **one installation** of the device (which I²C address it was strapped to, which bus it hangs off) are instance parameters, not chip facts (devicetree lesson, R-004). `instance` is a map from parameter name to:

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `type` | type name (§7.1) | yes | Scalar type. |
| `default` | scalar | no | Used if the binding does not supply a value. |
| `enum` | list of scalar | no | Allowed values. |
| `min`, `max` | number | no | Allowed range. |
| `description` | string | no | |

Values are supplied by the host configuration, by a parent bridge, or by probing (§5.4), and are visible to expressions as `instance.<name>`.

```yaml
instance:
  address: { type: u8, enum: [0x48, 0x49, 0x4A, 0x4B], default: 0x48, description: "Set by the ADD0 pin" }
```

---

## 5. Matching

`match` (list, REQUIRED, at least one entry) says which hardware this description serves. The host evaluates entries against an enumerated device; any matching entry binds the description.

### 5.1 Common fields

Every entry has `bus` (REQUIRED) and MAY have:

- `variant` — the variant to use without running `id_check`.
- `quirks` — list of quirk names (strings, `^[a-z][a-z0-9_-]*$`) visible to expressions as `quirks` (a list) and documented in `x-` or `description`.
- `instance` — values for instance parameters implied by this entry.
- `priority` — integer, default 0, used only to break ties between descriptions of equal specificity (§5.6).

### 5.2 `bus: usb`

Fields mirror Linux `struct usb_device_id` (R-004). Every field present MUST match; absent fields are wildcards.

| Field | Type | Matches |
|---|---|---|
| `vid` | u16 | `idVendor` |
| `pid` | u16 | `idProduct` |
| `bcd_device` | `{min: u16, max: u16}` | `bcdDevice` inclusive range |
| `device_class`, `device_subclass`, `device_protocol` | u8 | device descriptor |
| `interface_class`, `interface_subclass`, `interface_protocol` | u8 | interface descriptor |
| `interface_number` | u8 | `bInterfaceNumber` |
| `manufacturer`, `product`, `serial` | string (glob `*`, `?`) | string descriptors — SHOULD only be used to disambiguate |

An entry MUST contain at least one of `vid`, `device_class`, `interface_class`. An entry with `vid` alone (no `pid`, class or interface fields) is reported by `DD-L011` (too broad).

Each USB entry has a canonical **modalias** form compatible with Linux, e.g. `usb:v1209p0001d*dc*dsc*dp*icFFisc*ip*in*`, used as the registry lookup key and for udev integration.

```yaml
match:
  - bus: usb
    vid: 0x1209
    pid: 0x0001
    interface_class: 0xFF
```

### 5.3 `bus: hid`

`vid`, `pid` (u16), `usage_page` (u16), `usage` (u16), `report_id` (u8). Matches HID devices on any transport (USB, Bluetooth, I²C-HID).

### 5.4 `bus: i2c` and other non-enumerating buses

I²C and SPI devices do not announce themselves. Entries have:

- `compatible` — a `vendor,part` string (devicetree style), and/or
- `probe` — `{ addresses: [u8...] }` permits the host to probe these addresses; the description's `variants[*].id_check` (or a `probe_check` with the same shape as `id_check`) MUST then be present so probing reads an identity register rather than writing.

A probe MUST NOT write to the device (rule `DD-L018`).

```yaml
match:
  - bus: i2c
    compatible: ti,tmp117
  - bus: i2c
    probe: { addresses: [0x48, 0x49, 0x4A, 0x4B] }
```

### 5.5 `bus: serial`, `bus: pci` [provisional]

- `serial`: `{ port_hint: string }` only; serial devices are normally matched through their USB bridge (§5.2) and bound explicitly.
- `pci` **[provisional]**: `vendor`, `device`, `subsystem_vendor`, `subsystem_device` (u16), `class` (u24), `class_mask` (u24), `revision` `{min, max}`. Modalias `pci:v0000XXXXd0000XXXXsv*sd*bc*sc*i*`.

### 5.6 Specificity and precedence

When several descriptions match one device, the host picks the most specific entry: one point per matched field, `pid` and `interface_number` count two, string fields count one half. Ties are broken by `priority`, then by registry trust level (vendor-signed over community), then by newest `device.version`. A user or OS policy MAY override the choice.

---

## 6. Transports

`transport` (REQUIRED) is an object with exactly one bus key (`usb`, `serial`, `i2c`, `hid`, `spi`, `pci`, `bridge`) and optional `framing` (§7.3).

Transports **stack**: an I²C chip (`transport.i2c`) is reached through a bridge description that `provides: i2c` (§6.7). The chip description never names the bridge.

### 6.1 `usb`

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `interface` | u8 | yes | Interface number to claim. |
| `alt_setting` | u8 | no | Default 0. |
| `configuration` | u8 | no | Required configuration value; default: the active one. |
| `endpoints` | map name → endpoint | no | Endpoint `{ address: u8, type: bulk|interrupt|isochronous, max_packet: u16 }`. Address bit 7 = IN. |
| `detach_kernel_driver` | bool | no | Default false. If true, the host MAY detach an OS driver bound to the interface. |
| `os_descriptors` | `{ winusb: bool }` | no | Device advertises MS OS 2.0 descriptors for driverless WinUSB binding. |

USB control transfers are issued with the `control` step (§9.3).

### 6.2 `serial`

`baud` (u32, req.), `data_bits` (5–8, default 8), `parity` (`none|even|odd|mark|space`, default none), `stop_bits` (`1|1.5|2`, default 1), `flow` (`none|rts_cts|xon_xoff`, default none), `line_ending` (string, for text protocols), `dtr`, `rts` (bool, initial line states).

### 6.3 `i2c`

`address` (u8 or u16 if `address_bits: 10`; *parametric*), `address_bits` (7 or 10, default 7), `max_speed_hz` (u32, default 100000), `smbus` (bool — device needs SMBus semantics such as PEC; default false), `pec` (bool).

Register access over I²C follows §8.6.

### 6.4 `hid`

`interface` (u8), `reports`: map name → `{ id: u8, kind: input|output|feature, size: u16 bytes }`. Reports are read/written with `hid_get`, `hid_set`, `hid_read`, `hid_write` steps.

### 6.5 `spi` [provisional]

`mode` (0–3), `max_speed_hz`, `bits_per_word` (default 8), `cs` (*parametric*), `lsb_first` (bool), `three_wire` (bool). Register access per §8.6 using `read_flag_mask`/`write_flag_mask`.

### 6.6 `pci` [provisional]

`bars`: map name → `{ index: 0..5, kind: mmio|io, size: u64, prefetchable: bool }`; `requires_iommu` (bool, default true — a conforming runtime MUST NOT give a userspace driver DMA without an IOMMU); `msi`/`msix` vector counts (see §20.2).

### 6.7 `bridge` and `provides`

A **bridge** is a device that gives access to another bus (USB-to-I²C, USB-to-SPI, USB-to-UART). Its own `transport` is how the host talks to the bridge (usually `usb`). It declares what it offers with `provides`:

```yaml
provides:
  i2c:
    write:      write_i2c       # command (addr, data: bytes) -> ()
    read:       read_i2c        # command (addr, length) -> (data: bytes)
    write_read: write_read_i2c  # optional combined transaction
    max_transfer: 64
    speeds_hz: [100000, 400000]
```

Each primitive names a command of the bridge with the signature shown. A runtime MUST use the primitive commands, and their safety envelope applies in addition to the child's. Bridges MAY provide `spi`, `uart`, `gpio` (`{ set: cmd, get: cmd, lines: u8 }`).

---

## 7. Data types, byte order, framing, checksums

### 7.1 Scalar type names

| Name | Meaning |
|---|---|
| `bool` | 1 byte, 0 = false, any other value = true on decode; encoded as 0/1 |
| `u8` `i8` | 8-bit unsigned/signed |
| `u16` `i16` `u24` `i24` `u32` `i32` `u64` `i64` | multi-byte integers; append `le` or `be` for byte order (`u16le`). Without suffix, the order is the enclosing `byte_order` (default `le`) |
| `f32` `f64` (+`le`/`be`) | IEEE 754 |
| `bytes(N)` | N bytes, N an integer literal or the name of an earlier integer field |
| `bytes` | the rest of the payload |
| `string(N)` / `string` | UTF-8 text, same length rules as `bytes` |
| `bits(N)` | an N-bit unsigned integer inside a bit-packed struct (§7.2) |

In expressions, all integer types are CEL `int` (64-bit signed) except `u64`, which is CEL `uint`. Floats are `double`. `bytes` are `bytes`. `string` is `string`.

### 7.2 Structs

A payload is an ordered list of fields: `[{ name: type }, ...]` or, as a shorthand when order is obvious from a single field, a map with one key. Bit-packed structs use `bits(N)` fields, LSB first, and MUST sum to a whole number of bytes.

```yaml
payload:
  - raw: i16le
  - status: u8
```

### 7.3 Framing

For message-oriented transports (USB bulk/interrupt, serial), `transport.framing` describes how commands are wrapped. It is an ordered list of frame fields:

| Field form | Meaning |
|---|---|
| `{ name: header, type: u8 }` | a fixed position field; commands set its value |
| `{ name: length, type: u8, counts: payload }` | length of a later field, filled automatically |
| `{ name: payload, type: bytes(length) }` | the command payload |
| `{ name: check, checksum: <algorithm>, over: [header, length, payload] }` | checksum (§7.5) |
| `{ name: sync, const: 0xAA55, type: u16be }` | a constant |

For text protocols (SCPI-like), use `framing: { text: { terminator: "\n", encoding: ascii } }`; commands then use `send: { text: "MEAS:TEMP?" }` and `expect: { text: { regex: "^(?P<value>[-0-9.]+)$" } }`.

```yaml
transport:
  usb: { interface: 0, endpoints: { out: { address: 0x01, type: bulk }, in: { address: 0x81, type: bulk } } }
  framing:
    - { name: header,  type: u8 }
    - { name: length,  type: u8, counts: payload }
    - { name: payload, type: bytes(length) }
    - { name: check,   checksum: crc8-smbus, over: [header, length, payload] }
```

### 7.4 `types`

Named reusable structs: `types: { sample: [ { raw: i16le }, { flags: u8 } ] }`, referenced by name wherever a type is expected.

### 7.5 Checksum catalogue

Algorithms are named from the CRC RevEng catalogue; validators MUST know at least: `none`, `sum8`, `xor8`, `crc8-smbus` (poly 0x07), `crc8-maxim` (Dallas/Maxim 1-Wire), `crc8-sensirion` (poly 0x31, init 0xFF), `crc16-ccitt-false`, `crc16-modbus`, `crc16-xmodem`, `crc16-usb`, `crc32-iso-hdlc`, `crc32c`. Unknown names are `DD-L013`.

---

## 8. Registers

For devices with a register file (I²C/SPI sensors, MMIO controllers), `registers` holds a regmap-style description (R-004).

### 8.1 `registers.config`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `reg_bits` | 8, 16, 32 | 8 | Width of a register address |
| `val_bits` | 8, 16, 32, 64 | 8 | Default width of a register value |
| `stride` | int | 1 | Address increment between consecutive registers |
| `byte_order` | `le`, `be` | `be` | Order of multi-byte values on the bus |
| `auto_increment` | bool | true | Multi-register reads advance the address automatically |
| `read_flag_mask`, `write_flag_mask` | int | 0 | Bits OR-ed into the address for reads/writes (SPI) |
| `space` | string | `bus` | Address space: `bus` (I²C/SPI), or a PCI BAR name (§6.6) for MMIO |

### 8.2 `registers.map`

A map from register name to:

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `address` | int | yes | Register address |
| `bits` | 8, 16, 24, 32, 64 | no | Width if different from `val_bits` |
| `access` | enum | yes | `ro` read-only · `rw` read/write · `wo` write-only · `rc` read clears · `rw1c` write 1 to clear · `rw1s` write 1 to set · `w1` write once |
| `volatile` | bool | no | Value changes without host writes; never cached (default: true for `ro`/`rc`, false otherwise) |
| `precious` | bool | no | Reading has side effects; never read speculatively, by debug tools, or by caches |
| `noinc` | bool | no | FIFO-style: multi-byte access does not advance the address |
| `reset` | int | no | Value after reset |
| `fields` | map name → field | no | Bit fields (§8.3) |
| `fieldset` | name | no | Use a shared fieldset (§8.4) instead of `fields` |
| `repeat` | `{ count: int, stride: int }` | no | This entry describes `count` registers `NAME0..NAMEn-1` |
| `description` | string | no | |

Ranges of two registers MUST NOT overlap (`DD-L005`) unless both set `overlap: true`.

### 8.3 Fields

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `bits` | `"H:L"` or `"N"` | yes | Bit range, inclusive, LSB = 0 |
| `access` | enum | no | Overrides the register's access for this field |
| `enum` | map name → int | no | Named values |
| `reset` | int | no | |
| `signed` | bool | no | Two's complement interpretation |
| `description` | string | no | |

Fields of one register MUST NOT overlap and MUST lie inside the register width.

### 8.4 `registers.fieldsets`

Reusable field maps: `fieldsets: { osr: { osrs_t: { bits: "7:5" }, osrs_p: { bits: "4:2" } } }`.

### 8.5 Example

```yaml
registers:
  config: { reg_bits: 8, val_bits: 16, byte_order: be }
  map:
    TEMP_RESULT: { address: 0x00, access: ro }
    CONFIG:
      address: 0x01
      access: rw
      reset: 0x0220
      fields:
        mode:     { bits: "11:10", enum: { continuous: 0, shutdown: 1, one_shot: 3 } }
        conv:     { bits: "9:7" }
        avg:      { bits: "6:5", enum: { none: 0, x8: 1, x32: 2, x64: 3 } }
        data_ready: { bits: "13", access: ro }
    TEMP_OFFSET: { address: 0x07, access: rw, reset: 0x0000 }
    DEVICE_ID:   { address: 0x0F, access: ro, volatile: false }
```

### 8.6 Register access on buses

- **I²C**: write = `[addr][value bytes]`; read = write `[addr]`, repeated start, read `n` bytes. With `reg_bits: 16`, the address is sent in `byte_order`.
- **SPI** [provisional]: first byte(s) = address OR `read_flag_mask`/`write_flag_mask`, then data.
- **MMIO** [provisional]: `space` names a BAR; access width = register `bits`; byte order is the device's.

---

## 9. Commands

`commands` is a map from command name to a command. A command is the unit of host→device interaction that safety rules, tests and class profiles refer to.

### 9.1 Command fields

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `description` | string | no | |
| `args` | map name → arg | no | Inputs (§9.2) |
| `send` | object | one of `send`/`steps` | Message to send through the framing (§9.4) |
| `expect` | object | no | Expected reply (§9.4) |
| `steps` | list of step | one of `send`/`steps` | Ordered operations (§9.3) |
| `output` | map name → output | no | Results (§9.5) |
| `timeout_ms` | int | conditional | Whole-command deadline. REQUIRED when the command talks to the device (`DD-L003`). |
| `retries` | int | no | Retries on timeout or checksum error, default 0. Only allowed when `idempotent` is true or the command only reads. |
| `idempotent` | bool | no | Running twice has the same effect as once. Default: true for commands with no writes. |
| `reads_only` | bool | no | Declares no device state changes; validators check this against the steps (`DD-L019`). |
| `requires_state` | list of state | no | States in which the command may run (§12). |

### 9.2 Arguments

`{ type: scalar type (req), unit: UCUM, min, max, enum: [...], default, description }`. Every numeric argument of a command that writes MUST be covered by a safety limit or by `min`/`max` (`DD-L004`).

### 9.3 Steps

Each step is an object with exactly one operation key:

| Step | Shape | Meaning |
|---|---|---|
| `read` | `{ read: REG, as: name }` or `{ read: REG.field, as: name }` | Read a register or field into a local |
| `read_block` | `{ read_block: REG, length: int\|cel, as: name }` | Read `length` bytes starting at `REG` (honours `noinc`) |
| `write` | `{ write: REG, value: int\|cel }` | Write a whole register |
| `update` | `{ update: REG, fields: { field: int\|cel\|enum-name } }` | Read-modify-write the named fields |
| `wait` | `{ wait_us: int\|cel }` or `{ wait_ms: int\|cel }` | Delay |
| `poll` | `{ poll: REG.field\|REG, until: cel, interval_us: int, timeout_ms: int }` | Read repeatedly until `until` is true (`value` is the variable) |
| `call` | `{ call: command, args: {...}, as: name }` | Run another command; its outputs become `name.<output>` |
| `let` | `{ let: name, cel: "..." }` | Compute a local |
| `store` | `{ store: var, cel: "..." }` | Assign a device variable (§9.6) |
| `check` | `{ check: cel, error: "message", layer: description\|transport }` | Fail the command if false |
| `send` / `receive` | `{ send: {...} }`, `{ receive: {...}, as: name }` | Framed message I/O inside a step list |
| `control` | `{ control: { direction: in\|out, type: standard\|class\|vendor, recipient: device\|interface\|endpoint, request: u8, value: u16\|cel, index: u16\|cel, length: int, data: cel }, as: name }` | USB control transfer |
| `transfer` | `{ transfer: { endpoint: name, data: cel \| length: int }, as: name }` | Raw USB bulk/interrupt transfer |
| `hid_get`, `hid_set`, `hid_read`, `hid_write` | `{ hid_get: report, as: name }` … | HID report I/O |
| `bus` | `{ bus: { i2c_write: { address, data } } }` and `i2c_read`, `i2c_write_read` | Raw bus primitive — only in bridge or recovery commands |
| `logic` | `{ logic: export_name, args: {...}, as: name }` [provisional] | Call a logic-module export (§21) |
| `dma_addr` | `{ dma_addr: region, as: name }` [provisional] | Device-visible address of a `memory` region (§20.3) |
| `read_memory`, `write_memory` | `{ read_memory: region, offset: int\|cel, length: int\|cel, as: name }`, `{ write_memory: region, offset, data: cel }` [provisional] | Access host memory shared with the device |
| `enqueue`, `await` | see §20.4 [provisional] | Queue operations |

Steps run in order. Any step failure fails the command. Locals are visible to later steps and to `output`.

### 9.4 `send` / `expect` (framed messages)

`send` assigns frame fields and payload: `send: { header: 0x20, payload: [ { enabled: bool }, { level: u8 } ] }`. Payload fields with a type and no value take the argument of the same name. `expect` lists frame-field values the reply MUST have, and a `payload` struct to decode; decoded fields become locals.

```yaml
commands:
  set_led:
    args: { enabled: { type: bool }, level: { type: u8, unit: "%", max: 100 } }
    send:   { header: 0x20, payload: [ { enabled: bool }, { level: u8 } ] }
    expect: { header: 0xA0 }
    timeout_ms: 50
```

### 9.5 Outputs

`output` maps names to `{ cel: expression (req), unit: UCUM, type: scalar type, description }`. Outputs are what tests (§16) and class profiles (§15) consume.

```yaml
  read_temp:
    reads_only: true
    steps:
      - { read: TEMP_RESULT, as: raw }
    output:
      celsius: { cel: "double(sign_extend(raw, 16)) * 0.0078125", unit: Cel }
    timeout_ms: 20
```

### 9.6 `vars`

Device-scoped variables that survive between commands (calibration coefficients, cached configuration): map name → `{ type: scalar type (req), initial: scalar, description }`. Written only by `store` steps; readable by every expression as `vars.<name>`. A runtime MUST reset `vars` to `initial` on unbind and MUST NOT persist them across binds unless a sequence re-populates them.

---

## 10. Expressions

Expressions use **CEL** (Common Expression Language, cel-spec) with the DriverDesk profile below. An expression appears as a string in a `cel` field.

### 10.1 Environment

| Name | Type | Available in |
|---|---|---|
| step locals (`as`, `let`) | any | later steps, `output`, `check` |
| `args.<name>` and bare argument names | per arg type | the command |
| `vars.<name>` | per var | everywhere |
| `instance.<name>` | per param | everywhere |
| `variant.<name>`, `variant.id` | scalar, string | everywhere |
| `quirks` | list(string) | everywhere |
| `value` | per register | `poll.until` |
| `settings.<name>` | per setting | everywhere |

### 10.2 Numeric semantics

CEL `int` is 64-bit signed, `uint` 64-bit unsigned, `double` IEEE 754. There are **no implicit conversions**: `raw / 100` with `raw` an `int` is integer division; write `double(raw) / 100.0` for a real number (`DD-L006` warns when an integer expression feeds an output with a non-dimensionless unit and contains `/`). Integer overflow is an **error** (cel-spec), not wrap-around; C code ported from drivers that relies on 32-bit wrap MUST mask explicitly with `band(x, 0xFFFFFFFF)`.

### 10.3 DriverDesk function library

Standard CEL has no bitwise operators (R-004). Conforming runtimes MUST provide:

| Function | Meaning |
|---|---|
| `shl(int, int) -> int`, `shr(int, int) -> int` | shift left; arithmetic shift right |
| `ushr(int, int) -> int` | logical shift right on the low 64 bits |
| `band`, `bor`, `bxor` `(int, int) -> int`; `bnot(int) -> int` | bitwise ops |
| `bit(int, int) -> bool` | test bit n |
| `field(int, hi, lo) -> int` | extract bits hi..lo |
| `sign_extend(int, bits) -> int` | interpret the low `bits` bits as two's complement |
| `clamp(x, lo, hi)` | int or double |
| `min(a, b)`, `max(a, b)`, `abs(x)` | int or double |
| `round(double) -> int`, `floor`, `ceil`, `trunc` | |
| `pow(double, double)`, `sqrt`, `log`, `log10`, `exp` | double |
| `be16(bytes, offset)`, `le16`, `be24`, `le24`, `be32`, `le32` `-> int`; `be16s`… signed variants | decode from bytes |
| `bytes_of(list(int)) -> bytes`, `size(bytes)` | |
| `crc(name, bytes) -> int` | checksum from §7.5 |

### 10.4 Restrictions

Expressions MUST be side-effect free and MUST compile (`DD-L012`). Runtimes MUST bound evaluation cost (a cost limit of 10 000 CEL cost units per expression is RECOMMENDED).

---

## 11. Units

Units are **UCUM** case-sensitive codes (`Cel`, `K`, `Pa`, `hPa`, `%`, `m/s2`, `mA`, `V`, `lx`, `Hz`, `s`, `ms`, `1` for dimensionless). Conforming validators MUST accept at least the subset in `spec/units/ucum-subset.yaml` and MUST check that an output's unit is convertible to the unit a class profile requires (`DD-L006`). Unknown codes are an error.

---

## 12. States, sequences and settings

### 12.1 `states`

```yaml
states:
  initial: unbound
  list: [unbound, idle, active, suspended]   # optional; inferred from transitions
  transitions:
    - { from: unbound, to: idle,   trigger: bind,    do: init }
    - { from: idle,    to: active, trigger: request }
    - { from: active,  to: idle,   trigger: { idle_ms: 5000 } }
    - { from: [idle, active], to: suspended, trigger: suspend, do: suspend }
    - { from: suspended, to: idle, trigger: resume, do: resume }
```

Triggers (`trigger`): `bind`, `unbind`, `request` (a command not allowed in the current state was requested), `suspend`, `resume`, `error`, `reset`, `{ idle_ms: int }`, `{ event: name }` (§20.1), `{ call: command }` (after the command succeeds). `do` names a sequence or is an inline step list. `from` may be a list or `"*"`.

Every state MUST be reachable from `initial` (`DD-L001`); every `to`/`from` MUST be a declared or inferred state (`DD-L002`).

If `states` is absent, the implied machine is `unbound → ready` on `bind` running sequence `init` if present.

### 12.2 `sequences`

Named step lists shared by states and commands. Reserved names with fixed meaning (Linux driver-model mapping, R-004):

| Name | Runs when | Linux analogue |
|---|---|---|
| `init` | device bound, before first command | `probe` |
| `configure` | after `init` and after every settings change | `chip_config` |
| `suspend` | host enters low power or idle policy fires | `runtime_suspend` |
| `resume` | leaving `suspend` | `runtime_resume` (MUST restore configuration) |
| `shutdown` | unbind or host shutdown | `remove`/`shutdown` |
| `recover` | after an `error` trigger | — |

### 12.3 Calibration example (BME280 pattern)

```yaml
vars:
  dig_t1: { type: u16 }
  dig_t2: { type: i16 }
sequences:
  init:
    - { write: RESET, value: 0xB6 }
    - { wait_ms: 2 }
    - { read_block: CALIB00, length: 26, as: c }
    - { store: dig_t1, cel: "le16(c, 0)" }
    - { store: dig_t2, cel: "le16s(c, 2)" }
```

### 12.4 `settings`

User- or OS-tunable configuration (IIO `*_available`/`*_default`, ESPHome options): map name → `{ type, enum | min/max, default, unit, apply: sequence-or-command }`. Changing a setting runs `apply` (default `configure`).

---

## 13. Timing

| Field | Type | Meaning |
|---|---|---|
| `timing.startup_us` | int\|cel | Time after power-on/reset before the device accepts commands |
| `timing.reset_us` | int\|cel | Time after a soft reset |
| `timing.min_interval_us` | int | Minimum gap between two bus transactions |
| `timing.conversion_us` | int\|cel | Typical measurement time; MAY depend on `settings` |
| `timing.watchdog_ms` | int | Device resets itself if idle this long |

A command's `timeout_ms` MUST be at least the sum of its literal waits (`DD-L014`).

---

## 14. Safety envelope

The safety envelope is the set of rules a runtime enforces **before** anything reaches the bus. It is the reason a description can be trusted to drive real hardware.

### 14.1 When required

A `safety` block is REQUIRED when any class in `device.classes` is an actuator class (`light.*`, `motor.*`, `power.*`, `heater.*`, `actuator.*`) or any command writes a register with a field that controls current, voltage, frequency, or power (`DD-L009`). It is RECOMMENDED for every description.

### 14.2 `limits`

Map path → `{ min, max, enum, unit, reason (req) }` where path is `command.arg`. Values outside the limit are rejected with layer `description` and test outcome `rejected_by_safety`.

### 14.3 `rates`

Map command → `"<n>/<s|min|h>"` (maximum invocations per period) or `{ max: "<n>/s", burst: int }`.

### 14.4 `registers`

Map `REG` or `REG.field` → `{ allowed: [int...] | { min, max }, forbidden: [int...], writable: false, reason }`. Applies to every write from any step, sequence, bridge or logic module.

### 14.5 `forbidden_sequences` and `interlocks`

- `forbidden_sequences`: list of `{ sequence: [command, ...], within_ms: int, reason }` — this order of commands within the window MUST be rejected.
- `interlocks`: list of `{ when: cel, deny: [command...], reason }` — while `when` is true, the commands are rejected.

### 14.6 Signing

`safety.signed_by` is `{ key: string (key id), signature: base64, signed_at: date-time }` over the canonical JSON of the `safety` block without `signed_by` (Appendix C). The registry MUST refuse to publish a vendor-trust description whose safety block is unsigned or whose signature does not verify. An unsigned block is a warning (`DD-L016`) in 0.1 for local development.

### 14.7 Enforcement semantics

A conforming runtime MUST:

1. check every argument against `limits` and `args` bounds before encoding;
2. check every register write (including read-modify-write results) against `safety.registers` and `access`;
3. apply `rates`, `forbidden_sequences`, and `interlocks`;
4. apply the envelope to commands issued by sequences, bridges, other commands, tests, and logic modules alike;
5. never offer a way to disable enforcement at run time.

A rejected operation fails with layer `description` and does not reach the transport.

```yaml
safety:
  limits:
    set_led.level: { max: 80, unit: "%", reason: "LED rated 20 mA; 80% keeps it under 16 mA" }
  rates:
    set_led: 100/s
  registers:
    CONFIG.mode: { forbidden: [2], reason: "reserved mode value" }
```

---

## 15. Exposure and class profiles

`expose` (REQUIRED, at least one entry) maps commands to the OS-visible device model. Each entry:

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `class` | profile name | yes | One of `device.classes` |
| `channel` | name | yes | Channel name, unique per class (`temp0`) |
| `read` | path `command.output` | conditional | Value source |
| `write` | command | conditional | Value sink |
| `event` | event name | no | Push source (§20.1) |
| `unit` | UCUM | no | Overrides the output's unit for display |
| `scale`, `offset` | number | no | IIO-style `(raw + offset) * scale` when `read` gives a raw value |
| `label` | string | no | |

References MUST resolve (`DD-L007`).

### 15.1 Class profiles

A **class profile** says what a description must expose to claim a class. Profiles live in `spec/schema/0.1/profiles/` and are referenced by name. 0.1 defines:

| Profile | Requires | Stability |
|---|---|---|
| `sensor.temperature` | ≥1 channel, `read`, unit convertible to `Cel` | stable |
| `sensor.humidity` | `read`, unit `%` (relative humidity) | stable |
| `sensor.pressure` | `read`, unit convertible to `Pa` | stable |
| `sensor.generic` | `read`, any unit | stable |
| `light.indicator` | `write`, args `enabled: bool` and optional `level` with unit `%` | stable |
| `bridge.i2c` | `provides.i2c` | stable |
| `bridge.spi`, `bridge.uart`, `bridge.gpio` | matching `provides` | stable |
| `input.keyboard`, `input.pointer`, `input.gamepad` | `event` channels carrying HID usages | provisional |
| `serial.uart` | `read` + `write` byte streams | provisional |
| `net.ethernet` | `streams` rx/tx (§20.5), `read` link state, MAC address output | provisional |
| `storage.block` | `queues` or commands `read_blocks`/`write_blocks`, block size output | provisional |
| `camera.video` | `streams` with a pixel format | provisional |
| `audio.pcm` | `streams` with sample rate/format | provisional |
| `display.basic` | framebuffer `memory` region + mode setting commands | provisional |
| `gpu.compute` | `queues` + `memory` + `firmware` | provisional (research) |

Channel naming SHOULD follow Linux IIO channel types and Zephyr sensor channels (R-005): `temp`, `pressure`, `humidityrelative`, `accel_x`, `illuminance`, …

---

## 16. Built-in tests

`tests` (REQUIRED, at least one) lists checks that run in simulation (against a software twin), on virtual USB, and on real hardware.

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `name` | name | yes | Unique |
| `call` | command | yes | Command to run (`DD-L008`) |
| `args` | map | no | Arguments |
| `expect` | matcher | yes | Below |
| `variant` | variant name | no | Only for this variant |
| `rung` | list of `sim`, `vusb`, `hil` | no | Where it may run; default all |
| `before` | list of step | no | Preparation (e.g. set a setting) |

Matchers: `ok` · `rejected_by_safety` · `{ error: { layer: name } }` · a map from output name to `{ equals: v }`, `{ between: [lo, hi] }`, `{ approx: { value: v, tolerance: t } }`, `{ one_of: [..] }`, `{ cel: "expr over outputs" }`.

Every safety limit SHOULD have a test that expects `rejected_by_safety` (`DD-L017`).

```yaml
tests:
  - { name: reads_room_temperature, call: read_temp, expect: { celsius: { between: [5, 45] } } }
  - { name: led_cap, call: set_led, args: { enabled: true, level: 100 }, expect: rejected_by_safety }
```

## 17. Visibility

`visibility: { default: public|private, private: [section-or-path...] }`. A private description (or private parts: typically `logic`, `firmware`, vendor test fixtures) is served only to authorised clients. Visibility is access control, **not** secrecy: anything a host executes can be observed by its owner (SCOPE "never" list).

## 18. Compatibility promise

From 1.0: a runtime implementing `driverdesk/1.y` MUST run every description with `spec: driverdesk/1.x` for x ≤ y. In 0.x, minor versions MAY break compatibility; validators SHOULD offer automatic migration. Sections marked provisional are excluded from the promise until promoted.

## 19. Security considerations

- Descriptions are untrusted input. Parsers MUST bound document size (RECOMMENDED 1 MiB), nesting depth (64), alias expansion (YAML anchors/aliases: RECOMMENDED limit 1 000 expansions), and expression cost (§10.4).
- A description cannot run native code. Logic modules (§21) run in a capability sandbox and can only call declared commands, subject to §14.
- Probing (§5.4) MUST be read-only. Raw `bus` steps are allowed only in bridge descriptions and `recover` sequences, and are still subject to §14.4.
- DMA (§20.3) MUST be confined by an IOMMU; a runtime without IOMMU protection MUST refuse descriptions with `memory` regions marked `dma: true`.
- Signatures (§14.6) bind the safety envelope to a vendor key; registry transparency logs make key misuse visible.

---

## 20. Complex devices

These sections extend the model to interrupt-driven, high-throughput and firmware-loaded devices (ADR-0013). Except §20.1, they are **[provisional]**.

### 20.1 `events`

Asynchronous device→host notifications (key presses, data-ready, link changes). Map name → `{ source, decode, output, rate_hint_hz }`:

- `source`: one of `{ usb_interrupt: endpoint }`, `{ hid_report: report }`, `{ interrupt: name }` (§20.2), `{ poll: { command: name, every_ms: int } }`, `{ frame: { header: int } }` (unsolicited framed messages).
- `decode`: payload struct (§7.2) or HID report layout reference.
- `output`: like command outputs.

```yaml
events:
  key:
    source: { hid_report: keys }
    decode: [ { modifiers: u8 }, { reserved: u8 }, { keycodes: bytes(6) } ]
    output:
      usages: { cel: "keycodes", description: "HID keyboard usages" }
```

### 20.2 `interrupts` [provisional]

Map name → `{ kind: msi|msix|intx|gpio, vector: int, status: REG, ack: { write: REG, value: cel } }`. A runtime routes an interrupt to the event that names it.

### 20.3 `memory` [provisional]

Host memory shared with the device. Map name → `{ size: int|cel, align: int, direction: to_device|from_device|bidirectional, dma: bool, coherent: bool, count: int }`. A step `{ dma_addr: region, as: name }` yields the device-visible address. Runtimes MUST map these through an IOMMU (§19).

### 20.4 `queues` [provisional]

Ring-buffer queues (NVMe submission/completion queues, NIC descriptor rings, GPU command rings). Map name → `{ memory: region, entry: type, depth: int|cel, producer: host|device, doorbell: { write: REG, value: cel }, head: REG|cel, tail: REG|cel, phase_bit: field-path, pair: queue }`. Commands enqueue with `{ enqueue: queue, entry: { field: cel } , as: slot }` and wait with `{ await: queue, match: cel, timeout_ms: int, as: name }`.

### 20.5 `streams` [provisional]

Continuous data (video frames, audio, network packets). Map name → `{ direction: in|out, source: { endpoint: name } | { queue: name }, format: string, rate: { cel } , unit: UCUM, frame: type, buffers: int }`. Class profiles `camera.video`, `audio.pcm`, `net.ethernet` consume streams. Streams are never routed through CEL per sample; expressions apply only to stream metadata.

### 20.6 `firmware` [provisional]

Firmware a device needs loaded at bind time: list of `{ name, sha256 (req), size, source: lvfs|registry|url, url, load: sequence }`. The firmware blob is fetched and verified by hash; `load` streams it with steps (`transfer`, `control`, `write`, or `enqueue`). Firmware *updating* of non-volatile memory is out of scope (LVFS/fwupd).

### 20.7 Example: a minimal NVMe-style controller (illustrative)

```yaml
transport:
  pci: { bars: { regs: { index: 0, kind: mmio, size: 0x4000 } }, requires_iommu: true }
registers:
  config: { reg_bits: 32, val_bits: 32, byte_order: le, space: regs }
  map:
    CAP:  { address: 0x00, bits: 64, access: ro, volatile: false }
    CC:   { address: 0x14, access: rw, fields: { en: { bits: "0" } } }
    CSTS: { address: 0x1C, access: ro, fields: { rdy: { bits: "0" } } }
memory:
  asq: { size: 4096, align: 4096, direction: to_device, dma: true }
  acq: { size: 4096, align: 4096, direction: from_device, dma: true }
queues:
  admin_sq: { memory: asq, entry: sqe, depth: 64, producer: host, doorbell: { write: SQ0TDBL, value: "tail" }, pair: admin_cq }
  admin_cq: { memory: acq, entry: cqe, depth: 64, producer: device, phase_bit: cqe.phase }
```

## 21. Logic modules [provisional]

`logic: { module: { oci: string, digest: sha256 }, world: string, exports: [name...], permissions: { commands: [name...] }, limits: { fuel: int, memory_bytes: int } }`. A logic module is a WebAssembly component (WIT world `driverdesk:logic@0.1.0`, ADR-0006). It can only call commands listed in `permissions.commands`, subject to §14. It never sees raw bus access or host memory.

## 22. Hints [provisional]

`hints` carries non-normative advice for execution targets (e.g. `{ native: { linux: { subsystem: iio } } }`, `{ polling: true }`). Hints MUST NOT change behaviour; a target MAY ignore them.

---

## Appendix A. Complete examples

Complete, validated examples live in `devices/examples/`: `minimal.yaml`, `pico-thermo.yaml`, `pico-bridge.yaml`, `tmp117.yaml`, `bme280.yaml`, and provisional complex-device examples `hid-keyboard.yaml`, `nvme-controller.yaml`. Every example is checked by `cargo xtask spec-check`.

## Appendix B. Diagnostic code registry

Schema errors use `DD-S` codes; lint rules use `DD-L` codes. Each code has a page at `https://driverdesk.dev/errors/<code>` (provisional domain) and in `docs/book/`.

| Code | Severity | Meaning |
|---|---|---|
| DD-S001 | error | YAML syntax error |
| DD-S002 | error | required field missing |
| DD-S003 | error | wrong type |
| DD-S004 | error | unknown field (with "did you mean") |
| DD-S005 | error | value not allowed (enum/const/pattern) |
| DD-S006 | error | value out of range |
| DD-S007 | error | duplicate key |
| DD-S008 | error | document too large / too deep / too many aliases |
| DD-S009 | error | other schema violation |
| DD-L001 | error | state unreachable from `initial` |
| DD-L002 | error | transition references an undefined state |
| DD-L003 | error | command talks to the device but has no `timeout_ms` |
| DD-L004 | error | numeric write argument has no safety limit or bounds |
| DD-L005 | error | overlapping register ranges |
| DD-L006 | error | unit or type mismatch in an expression or against a class profile |
| DD-L007 | error | `expose` references an unknown command, output or event |
| DD-L008 | error | test references an unknown command or output |
| DD-L009 | error | actuator class without a `safety` block |
| DD-L010 | error | unknown spec version |
| DD-L011 | warning | USB match too broad (vendor id only) |
| DD-L012 | error | CEL expression does not compile |
| DD-L013 | error | unknown checksum algorithm |
| DD-L014 | error | `timeout_ms` shorter than the sum of the command's waits |
| DD-L015 | error | write to a read-only register or field |
| DD-L016 | warning | safety block not signed |
| DD-L017 | warning | safety limit without a `rejected_by_safety` test |
| DD-L018 | error | probe would write to the device |
| DD-L019 | error | command declared `reads_only` contains a write |
| DD-L020 | error | reference to an unknown register or field |
| DD-L021 | error | field bits outside the register or overlapping |
| DD-L022 | error | unknown variable, argument or local in an expression |
| DD-L023 | warning | provisional section used |
| DD-L024 | error | variant `exclude` names something that does not exist |
| DD-L025 | error | `retries` on a non-idempotent command |
| DD-L026 | error | unknown UCUM unit |
| DD-L027 | error | YAML 1.1 boolean-like plain scalar (`on`, `off`, `yes`, `no`, `y`, `n`) used as a key or string |
| DD-L028 | error | integer with a leading zero (octal in YAML 1.1, decimal in 1.2) |

## Appendix C. Canonical form

The canonical form of a description (or of its `safety` block) is the JSON value produced by parsing the YAML (integers as JSON numbers, keys in mapping order discarded), serialised with RFC 8785 (JSON Canonicalization Scheme). Hashes and signatures are computed over these bytes.

## Appendix D. References

RFC 2119, RFC 8174, RFC 8785; YAML 1.2.2; JSON Schema 2020-12; cel-spec (github.com/google/cel-spec); UCUM (ucum.org); USB 2.0 and USB HID 1.11 specifications; Linux `include/linux/regmap.h`, `include/linux/iio/types.h`, `Documentation/devicetree/bindings/writing-schema.rst`; CRC RevEng catalogue; NVMe Base Specification 2.x. Research notes R-004, R-005.
