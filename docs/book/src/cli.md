# The `dd` command

## `dd validate`

```text
dd validate [--format human|json] [--deny-warnings] <FILES>...
```

Checks each description in three passes:

1. **Syntax** — YAML 1.2, with limits on size (1 MiB), nesting (64) and alias expansion (1000).
2. **Schema** — JSON Schema 2020-12 from `spec/schema/0.1/` (codes `DD-S001`–`DD-S009`).
3. **Lint** — semantic rules (`DD-L001`–`DD-L029`): references resolve, units match class profiles, expressions compile and use defined names, safety rules cover writes, timeouts are sane.

Exit status: `0` all files valid (warnings allowed), `1` at least one file invalid, `2` usage or file error. `--deny-warnings` makes warnings count as invalid.

`--format json` prints one object:

```json
{
  "spec": "driverdesk/0.1",
  "files": [
    {
      "file": "bme280.yaml",
      "valid": true,
      "diagnostics": [
        { "code": "DD-L016", "severity": "warning", "message": "safety block is not signed",
          "span": { "start": 4512, "end": 4519 }, "label": "unsigned",
          "help": "fine for local development; …", "related": [], "pointer": "/safety" }
      ]
    }
  ]
}
```

`span` offsets are bytes into the file; `pointer` is an RFC 6901 JSON pointer into the document.
