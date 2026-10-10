# R-004. How are Linux device drivers built, and what of it becomes DriverDesk data?
- Date: 2026-10-10 · Stage/task: P0.T1 · Plan question: R-P0-1 (Linux part)
- Confidence: high (read from kernel source, torvalds/linux @ 3857c2fe5449, 2026-10-10)
- Answer: A Linux driver for a simple device is ~10% device facts wrapped in ~90% kernel plumbing. The facts fall into seven buckets — **match ids, variants, register map, init/probe sequence, data channels, power management, limits** — and Linux already expresses most of them as tables (`*_device_id`, `regmap_config`, `iio_chan_spec`, `chip_info`). DriverDesk's description should be exactly those tables as YAML, with the remaining imperative bits (calibration maths, conversion waits) as CEL `compute` steps or a Wasm logic module.

## What we read (local cache, gitignored: `cache/linux`, sparse checkout)
| File | Lines | Why |
|---|---|---|
| `drivers/iio/temperature/tmp117.c` | 243 | smallest complete I²C sensor driver (our lab sensor) |
| `drivers/iio/pressure/bmp280-{core,regmap,i2c,spi}.c`, `bmp280.h` | ~3,800 | 5 chip variants in one driver (BMP180/280, BME280, BMP380/580), calibration maths, PM |
| `drivers/hwmon/lm75.c` | 1,176 | same kind of device exposed via a *different* subsystem (hwmon) |
| `drivers/i2c/busses/i2c-tiny-usb.c` | 302 | USB→I²C bridge with vendor control requests (our `i2c-bridge` transport) |
| `drivers/usb/serial/cp210x.c` | 2,202 | 205 `USB_DEVICE()` ids + per-model quirks |
| `drivers/hid/hid-led.c`, `drivers/leds/leds-pca963x.c` | 535, 570 | actuators (LEDs) via HID and I²C |
| `include/linux/regmap.h`, `iio/types.h`, `uapi/linux/iio/types.h`, `device-id/usb.h` | — | the vocabulary |
| `Documentation/devicetree/bindings/iio/temperature/ti,tmp117.yaml`, `writing-schema.rst`, `vendor-prefixes.yaml` | — | how hardware *instances* are described (YAML + JSON Schema) |

## Anatomy of a Linux driver (TMP117 as the worked example)

1. **Match tables** — how the kernel decides this driver owns a device.
   - Devicetree: `{ .compatible = "ti,tmp117" }` — `vendor,part` strings; vendors registered in `vendor-prefixes.yaml`.
   - I²C: `{ .name = "tmp117" }`. USB: `struct usb_device_id` = `match_flags` + `idVendor`, `idProduct`, `bcdDevice_lo/hi`, device class/subclass/protocol, interface class/subclass/protocol/number. Flags say which fields count (`USB_DEVICE_ID_MATCH_VENDOR|PRODUCT|...`). Exported as modalias strings (`usb:v1209p0001d*dc*...`) so udev can autoload.
   - Each entry carries `driver_data` → a pointer to the variant's info table.
2. **Variants** — one driver, many chips. TMP117 reads `DEVICE_ID` reg 0x0F and picks TMP116/117/119 info (`0x1116`, `0x0117`, `0x2117`), falling back to the match-table entry. BMP280 has a `bmp280_chip_info` struct per chip: id register + accepted ids, regmap config, start-up time, channel list, available/default oversampling, IIR coefficients, sampling frequencies, scale coefficients, and function pointers (`chip_config`, `read_temp`, `read_calib`, `preinit`, `set_mode`, `wait_conv`). **This struct is a description format written in C.**
3. **Register map (regmap)** — `regmap_config`: `reg_bits`, `val_bits`, `reg_stride`, `max_register`, per-register predicates `writeable_reg`/`readable_reg`/`volatile_reg`/`precious_reg` (+ `*_noinc_reg` for FIFOs), `reg_defaults`, `cache_type`, `read_flag_mask`/`write_flag_mask` (SPI R/W bit), `use_single_read/write`. In practice the predicates are `switch` statements listing register addresses — i.e. data. Field access via `FIELD_PREP/FIELD_GET` masks and `regmap_write_bits(reg, mask, val)`.
4. **Probe / init sequence** — check bus capability (`I2C_FUNC_SMBUS_WORD_DATA`), enable supply (`vcc`), wait start-up (`fsleep(1500)`), read & verify chip id, soft reset (write `0xB6` to RESET), wait start-up again, read calibration block, write config registers, register with subsystem. Errors use `dev_err_probe` with a reason.
5. **Data model to userspace (IIO)** — each channel = `type` (`IIO_TEMP`, `IIO_PRESSURE`, `IIO_HUMIDITYRELATIVE`, … 43 types) + optional modifier (X/Y/Z…) + `info_mask` bits from 32 attributes (`RAW`, `PROCESSED`, `SCALE`, `OFFSET`, `CALIBBIAS`, `CALIBSCALE`, `SAMP_FREQ`, `OVERSAMPLING_RATIO`, `INT_TIME`, `HYSTERESIS`, …). Userspace computes `value = (raw + offset) * scale` in fixed units (temperature in milli-°C, pressure in kPa). TMP117: raw = sign-extended 16-bit reg 0x00, scale = 7.8125 m°C/LSB, calibbias writable (reg 0x07). The **same** device class (temp sensor) may instead be exposed via **hwmon** (lm75: `HWMON_CHANNEL_INFO(temp, ...)`), with different units/conventions — Linux has no single class model per device kind.
6. **Power management** — runtime PM with autosuspend: suspend = set sleep mode, wait, disable regulators; resume = enable regulators, wait start-up, re-run `chip_config` (registers are lost). System sleep maps to the same ops.
7. **Limits** — implicit, scattered: `clamp_t(int, val, S16_MIN, S16_MAX)` on calibbias, `clamp_val(var, 0, 419430400)` in humidity maths, "available" lists validated in `write_raw`. Nothing declares a safety envelope.

Lifetime/cleanup is handled by `devm_*` (resources released automatically on unbind) — runtime concern, not description.

## What is data vs. logic (bucket → DriverDesk section)
| Linux construct | Nature | DriverDesk 0.1 section |
|---|---|---|
| `of_device_id` / `i2c_device_id` / `usb_device_id` + `match_flags` | data | §5 Matching — copy the USB field set and flag semantics exactly; modalias-style keys; `compatible`-style `vendor,part` ids with a vendor-prefix registry |
| `driver_data` → `chip_info` per variant; chip-id register check | data | §4 Identity: `variants:` each with `id_check: {register, equals}` and overrides |
| `regmap_config` + access predicates + `reg_defaults` + field masks | data | §8 Registers: `reg_bits`, `val_bits`, `stride`, per-register `access: [read, write]`, `volatile`, `precious`, `reset`, `fields: {name: {bits: "7:5"}}`, `noinc` |
| probe/preinit/chip_config/resume sequences | data (ordered steps) | §12 States: `init`, `configure`, `suspend`, `resume` as step lists (write, read+expect, wait) |
| `start_up_time_us`, `fsleep`, conversion-time formula | data + small expressions | §13 Timing; formula as CEL |
| IIO channel `type` + `info_mask` + scale/offset | data | §15 Expose & class profiles — borrow IIO type names and RAW/SCALE/OFFSET/PROCESSED model; UCUM units instead of fixed milli-units |
| `*_avail` / `*_default` lists (oversampling, IIR, ODR) | data | parameters with `enum` + `default` |
| calibration readout + compensation formulas (BME280 ~10–20 lines of 32-bit integer maths with shifts) | logic | §10 `compute:` CEL steps — **needs bitwise/shift ops** (see below) — or Wasm logic module (P7) |
| `clamp_*`, avail checks | implicit safety | §14 Safety envelope — explicit, signed, runtime-enforced (Linux has no equivalent: our addition) |
| quirk flags in `driver_info` (cp210x) | data | `quirks:` per match entry |
| `devm_*`, locking, subsystem registration, sysfs | runtime | not in description |

## Findings that change the plan
1. **CEL has no bitwise or shift operators in the standard language.** BME280 compensation is shift-heavy (`>> 14`, `<< 20`). R-P0-2 default must add DriverDesk CEL functions (`shl`, `shr`, `band`, `bor`, `bxor`, `sign_extend(x, bit)`, `clamp`) or verify a maintained Rust CEL that ships a bitwise extension. Source: cel-spec `doc/langdef.md` grammar has only `* / % + -`, relations, `&& || ?:`; ints are 64-bit and **overflow is an error** (C code relying on 32-bit wrap must be checked when ported). *Action: P0.T3 / R-P0-5.*
2. **Separate "driver" from "instance".** DT bindings describe a board's *instance* (I²C address `reg = <0x48>`, `vcc-supply`), drivers describe the *chip*. Our description = chip; bus address/strap options are instance parameters (`instance: {address: {enum: [0x48,0x49,0x4a,0x4b], default: 0x48}}`), set by the bridge/board, not hard-coded.
3. **Variants are first-class.** Every real driver we read handles a family (TMP116/117/119; five Bosch chips). Spec 0.1 needs `variants` with id-register detection, or one file per chip duplicates ~80%.
4. **One class model.** Linux's IIO-vs-hwmon split for the same device is a known pain; our class profiles (`sensor.temperature` …) pick IIO's channel model and map to hwmon/IIO/macOS/Windows in surfaces (P5).
5. **dt-schema is our closest relative**: YAML authoring, JSON Schema, `$id` URIs, `additionalProperties: false`, required `examples:` that are validated. Adopt the same conventions (already ADR-0002) and the "examples must validate" rule (P0 exit criterion 1).
6. **Licence:** kernel driver code is GPL-2.0 — never copied into this repo. DT binding YAML files are dual `GPL-2.0-only OR BSD-2-Clause` → a future importer may read them under BSD. Register addresses and ids are facts; the P0.T10 benchmark records only driver path + commit.
7. **Size baseline for the yardstick:** TMP117 driver 243 lines of C → target description ≤ 80 lines YAML; BME280 portion of bmp280 → target ≤ 200 lines YAML + compute. Record actuals in the P0 report.

## Re-check when
Kernel 7.x regmap/IIO API changes (e.g. `device-id/` header split seen here is recent) · before writing the Linux surface (P5).
