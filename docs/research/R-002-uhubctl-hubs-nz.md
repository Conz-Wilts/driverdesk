# R-002. Which per-port-power USB hub can we buy in NZ for the lab?
- Date: 2026-10-10 · Stage/task: B.T2 · Plan question: R-B-3
- Confidence: medium (uhubctl table is authoritative; retailer stock/price is a snapshot)
- Answer: Buy 3× **StarTech HB30A4AIB** (4-port USB 3.0, VIA VL813 `2109:2817`, listed per-port in uhubctl) from PB Tech NZ (~NZD 71 incl. GST each). Backup: **UUGear MEGA4** (VL817 `2109:0817`, per-port via AP2511), ~€31 + shipping. Raspberry Pi 5 onboard ports are **ganged** — an external hub is required.

## Evidence
- uhubctl README compatibility table (master, v2.6.0): HB30A4AIB listed `2109:2817`, no caveats; MEGA4 `2109:0817`, no caveats. https://github.com/mvp/uhubctl
- Same README: Pi 5's four onboard hubs "advertise per-port power switching, but this is not true"; VBUS drops only when all ports off.
- README warns new hardware revisions of listed hubs may lose switching (D-Link DUB-H4/H7 only rev D/E).
- PB Tech NZ part HUBSTT4283767, NZD 61.74 ex GST (snapshot 2026-10-10). https://www.pbtech.com/pacific/product/HUBSTT4283767/
- UUGear MEGA4 €31 ex VAT, in stock. https://www.uugear.com/product/mega4-4-port-usb-3-ppps-hub-for-raspberry-pi-4b/

## Implications
- H-001 lists HB30A4AIB ×3. Each hub's physical port switches must stay ON (uhubctl switches downstream of them).
- Acceptance check on arrival (B.T7): `uhubctl` lists the hub with "ppps"; `uhubctl -l <loc> -p 1 -a cycle` drops VBUS (DUT disappears from `lsusb`, reappears). Fail → switch to MEGA4.
- Mac/Windows nodes need a USB-C→A adapter if they lack USB-A (mini PC has USB-A).

## Re-check when
Hub arrives (verify silicon revision) · uhubctl major release · before R1 lab expansion.
