# Diagnostic codes

Every message from `dd validate` has a stable code. `DD-S` codes come from the schema check, `DD-L` codes from semantic lint rules.

| Code | Severity | Meaning |
|---|---|---|
| [DD-S001](DD-S001.md) | error | YAML syntax error |
| [DD-S002](DD-S002.md) | error | required field missing |
| [DD-S003](DD-S003.md) | error | wrong type |
| [DD-S004](DD-S004.md) | error | unknown field (with "did you mean") |
| [DD-S005](DD-S005.md) | error | value not allowed (enum/const/pattern) |
| [DD-S006](DD-S006.md) | error | value out of range |
| [DD-S007](DD-S007.md) | error | duplicate key |
| [DD-S008](DD-S008.md) | error | document too large / too deep / too many aliases |
| [DD-S009](DD-S009.md) | error | other schema violation |
| [DD-L001](DD-L001.md) | error | state unreachable from `initial` |
| [DD-L002](DD-L002.md) | error | transition references an undefined state |
| [DD-L003](DD-L003.md) | error | command talks to the device but has no `timeout_ms` |
| [DD-L004](DD-L004.md) | error | numeric write argument has no safety limit or bounds |
| [DD-L005](DD-L005.md) | error | overlapping register ranges |
| [DD-L006](DD-L006.md) | error | unit or type mismatch in an expression or against a class profile |
| [DD-L007](DD-L007.md) | error | `expose` references an unknown command, output or event |
| [DD-L008](DD-L008.md) | error | test references an unknown command or output |
| [DD-L009](DD-L009.md) | error | actuator class without a `safety` block |
| [DD-L010](DD-L010.md) | error | unknown spec version |
| [DD-L011](DD-L011.md) | warning | USB match too broad (vendor id only) |
| [DD-L012](DD-L012.md) | error | CEL expression does not compile |
| [DD-L013](DD-L013.md) | error | unknown checksum algorithm |
| [DD-L014](DD-L014.md) | error | `timeout_ms` shorter than the sum of the command's waits |
| [DD-L015](DD-L015.md) | error | write to a read-only register or field |
| [DD-L016](DD-L016.md) | warning | safety block not signed |
| [DD-L017](DD-L017.md) | warning | safety limit without a `rejected_by_safety` test |
| [DD-L018](DD-L018.md) | error | probe would write to the device |
| [DD-L019](DD-L019.md) | error | command declared `reads_only` contains a write |
| [DD-L020](DD-L020.md) | error | reference to an unknown register or field |
| [DD-L021](DD-L021.md) | error | field bits outside the register or overlapping |
| [DD-L022](DD-L022.md) | error | unknown variable, argument or local in an expression |
| [DD-L023](DD-L023.md) | warning | provisional section used |
| [DD-L024](DD-L024.md) | error | variant `exclude` names something that does not exist |
| [DD-L025](DD-L025.md) | error | `retries` on a non-idempotent command |
| [DD-L026](DD-L026.md) | error | unknown UCUM unit |
| [DD-L027](DD-L027.md) | error | YAML 1.1 boolean-like plain scalar (`on`, `off`, `yes`, `no`, `y`, `n`) used as a key or string |
| [DD-L028](DD-L028.md) | error | integer with a leading zero (octal in YAML 1.1, decimal in 1.2) |
| [DD-L029](DD-L029.md) | error | reference to an unknown command, sequence, variable, event, queue, memory region, endpoint, report, type or frame field |
