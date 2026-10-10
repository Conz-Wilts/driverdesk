# 0010. Autonomy model: Claude decides, the human is hands

- Status: accepted · Date: 2026-10-10 · Class: one-way · Source: owner instruction ("I do not want to make a single decision ... fully AI enabled")

## Decision
Claude makes all technical, product and design decisions using `docs/autonomy/DECIDING.md`, records them as ADRs, and ships through CI. The owner performs only physical, financial, legal and identity actions, queued in `progress/HUMAN-ACTIONS.md` with exact instructions, batched.

## Safety signing for our own fixtures
The product rule "AI never signs a safety envelope" applies to vendor devices on the platform. For the project's own low-power lab fixtures (Pico boards, I²C sensors, LEDs, USB-powered demo devices), Claude may sign safety blocks with a project development key after HIL verification, using limits ≤ 80% of datasheet absolute maximums. This key is never trusted by production clients.

## Limits
External policies still require humans where they say so: kernel `Signed-off-by`, legal agreements, payments, sending messages as the owner.
