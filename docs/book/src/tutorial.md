# Describe your first device in 30 minutes

You will write a description for **pico-thermo**: a Raspberry Pi Pico running small firmware that appears on USB as a vendor-specific device, reports its chip temperature, and drives an LED. By the end, `dd validate` accepts your file and you know every section a description has.

You need: the `dd` tool (`cargo build -p dd-cli`, binary at `target/debug/dd`) and a text editor. No hardware is needed for this tutorial.

The finished file is `devices/examples/pico-thermo.yaml` if you get stuck.

## 1. Start the file (2 min)

Create `pico-thermo.yaml`:

```yaml
spec: driverdesk/0.1
device:
  id: example.pico-thermo
  name: Pico Thermo LED
  vendor: { name: Example Labs, prefix: example }
  version: 0.1.0
  classes: [sensor.temperature, light.indicator]
```

- `spec` says which version of the standard the file follows.
- `device.id` is globally unique: vendor prefix first, lower case, dots.
- `classes` says what the device *is* to the operating system. Each class has a profile that says what you must expose (a temperature sensor must expose a channel in a temperature unit).

Run `dd validate pico-thermo.yaml`. You get `DD-S002` errors for the sections still missing. That is expected; each message names the section to add.

## 2. Say which hardware this is for (3 min)

```yaml
match:
  - bus: usb
    vid: 0x1209
    pid: 0x0001
    interface_class: 0xFF
```

The fields mirror Linux's `usb_device_id`. Every field you write must match; fields you leave out match anything. `0x1209:0x0001` is a pid.codes test id. `interface_class: 0xFF` (vendor-specific) means no operating system driver claims the device first.

Try deleting `pid` and `interface_class` and validating: you get warning `DD-L011`, because a vendor id alone would claim every product of that vendor.

## 3. Say how to talk to it (5 min)

```yaml
transport:
  usb:
    interface: 0
    endpoints:
      out: { address: 0x01, type: bulk, max_packet: 64 }
      in:  { address: 0x81, type: bulk, max_packet: 64 }
    os_descriptors: { winusb: true }
  framing:
    - { name: header,  type: u8 }
    - { name: length,  type: u8, counts: payload }
    - { name: payload, type: bytes(length) }
    - { name: check,   checksum: crc8-smbus, over: [header, length, payload] }
```

The firmware speaks a tiny framed protocol over two bulk endpoints: one header byte, a length byte, the payload and a CRC-8. `framing` describes that once; every command reuses it. The runtime fills in `length` and `check` for you.

## 4. Describe the commands (8 min)

```yaml
commands:
  hello:
    send:   { header: 0x01 }
    expect: { header: 0x81, payload: [ { proto: u8 }, { fw_major: u8 }, { fw_minor: u8 } ] }
    output:
      protocol: { cel: "proto", type: u8 }
    timeout_ms: 50
    retries: 2
  read_temp:
    send:   { header: 0x10 }
    expect: { header: 0x90, payload: [ { raw: i16le } ] }
    output:
      celsius: { cel: "double(raw) / 100.0", unit: Cel }
    timeout_ms: 50
    retries: 2
  set_led:
    args:
      enabled: { type: bool }
      level:   { type: u8, unit: "%", min: 0, max: 100 }
    send:   { header: 0x20, payload: [ { enabled: bool }, { level: u8 } ] }
    expect: { header: 0xA0 }
    timeout_ms: 50
```

Three ideas carry most descriptions:

- **send / expect**: the bytes going out and the reply that must come back. Decoded reply fields (`raw`) become names you can use.
- **output** turns raw values into real quantities with a [CEL](https://cel.dev) expression and a [UCUM](https://ucum.org) unit (`Cel` is degrees Celsius). CEL never converts types silently: `raw / 100` would be integer division and the validator warns (`DD-L006`); `double(raw) / 100.0` is what you want.
- **timeout_ms** is required for anything that talks to the device (`DD-L003`). `retries` is only allowed for commands that are safe to repeat (`DD-L025`): reading is, toggling an LED by "flip" would not be.

## 5. Make it safe (4 min)

`set_led` drives real hardware, and `light.indicator` is an actuator class, so a `safety` block is required (`DD-L009`):

```yaml
safety:
  limits:
    set_led.level: { max: 80, unit: "%", reason: "LED rated 20 mA at 3.3 V; 80% duty keeps average current under 16 mA" }
  rates:
    set_led: 100/s
```

The runtime checks these rules **before** any byte reaches the bus, for every caller: apps, tests, other commands, logic modules. There is no switch to turn them off. Every limit needs a `reason` a reviewer can check against the datasheet. When a vendor publishes, they sign this block (until then you see warning `DD-L016`).

## 6. Lifecycle (3 min)

```yaml
states:
  initial: unbound
  transitions:
    - { from: unbound, to: idle,   trigger: bind,  do: [ { call: hello, as: h }, { check: "h.protocol == 1", error: "unsupported firmware protocol" }, { wait_ms: 10 } ] }
    - { from: idle,    to: active, trigger: request }
    - { from: active,  to: idle,   trigger: { idle_ms: 5000 } }
    - { from: "*",     to: unbound, trigger: unbind }
```

When the device is plugged in (`bind`), the runtime calls `hello` and refuses firmware that speaks another protocol version. The states map onto Linux's probe/suspend/resume model, which keeps generated kernel drivers simple later.

Note the words: `trigger`, `unbound`. Unquoted `on` and `off` mean true/false to older YAML parsers, so the validator rejects them as names (`DD-L027`).

## 7. Expose it and test it (4 min)

```yaml
expose:
  - { class: sensor.temperature, channel: temp0, read: read_temp.celsius }
  - { class: light.indicator,    channel: led0,  write: set_led }
tests:
  - { name: hello_protocol_1, call: hello, expect: { protocol: { equals: 1 } } }
  - { name: room_temperature, call: read_temp, expect: { celsius: { between: [5, 45] } } }
  - { name: led_on,  call: set_led, args: { enabled: true, level: 50 }, expect: ok }
  - { name: led_cap, call: set_led, args: { enabled: true, level: 90 }, expect: rejected_by_safety }
visibility: { default: public }
```

`expose` is what the operating system sees: a temperature channel and an LED. `tests` run in simulation, on a virtual USB device, and on real hardware in the lab — the same tests everywhere. The last test proves the safety limit works; the validator warns (`DD-L017`) when a limit has no such test.

## 8. Validate (1 min)

```text
$ dd validate pico-thermo.yaml
pico-thermo.yaml: ok (0 errors, 1 warnings)
```

The one warning is the unsigned safety block. You have written a complete driver without a line of C.

## Where next

- A register-based I²C sensor with variants and calibration: `devices/examples/tmp117.yaml`, `bme280.yaml`.
- A USB-to-I²C bridge that every I²C chip description can run through: `pico-bridge.yaml`.
- Complex devices (provisional): `hid-keyboard.yaml`, `nvme-controller.yaml`.
- The full rules: [specification](spec.md).
