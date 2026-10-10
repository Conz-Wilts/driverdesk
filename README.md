# DriverDesk

Device drivers as **data**. A driver is a YAML description of a device (plus optional sandboxed Wasm logic), run by a portable Rust runtime on any operating system, tested automatically on every OS, and distributed through a signed, transparency-logged registry. Hardware makers upload a driver once and see it proven everywhere; OS writers implement one small host interface and get the whole catalogue; users plug a device in and it works, verified as genuine.

**Status:** pre-alpha — stage B (bootstrap). No usable code yet. Progress: [`progress/STATE.md`](progress/STATE.md). Stage plans: [`plans/`](plans/README.md).

- What we build and for whom: [`docs/vision/SCOPE.md`](docs/vision/SCOPE.md)
- North-star plan: [`docs/vision/driverdesk-plan.html`](docs/vision/driverdesk-plan.html)
- Decisions: [`docs/decisions/`](docs/decisions/)

## Licence

Code: [Apache-2.0](LICENSE). Specification text (`spec/`): [CC-BY-4.0](spec/LICENSE). Contributions under the [DCO](CONTRIBUTING.md).
