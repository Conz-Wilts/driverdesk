# DriverDesk

DriverDesk describes device drivers as **data**. A driver is one YAML file — a *device description* — that says how to recognise a device, how to talk to it, what its registers and commands mean, what values it produces in which units, what must never be done to it, and how to test it. A portable runtime runs the description on any operating system.

This book is for people who write descriptions (hardware engineers, firmware developers, AI drafting tools) and for people who build runtimes and OS integrations.

- New here? Start with [Describe your first device in 30 minutes](tutorial.md).
- Looking up a message from `dd validate`? Every code has a page under [Diagnostic codes](errors/README.md).
- Need the exact rules? Read the [specification](spec.md).

Status: spec **0.1, draft**. Sections marked *provisional* (PCI, DMA, queues, streams, firmware loading, logic modules) may change in 0.2.
