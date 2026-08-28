---
status: accepted
---

# Use NLM UCUM for automatic normalization

## Context

Hemo Tracker must compare measurements that use different commensurable units. UCUM codes are case-sensitive. A text map or a regular expression cannot implement UCUM dimensional semantics safely.

The application must work without a network connection. It must not send laboratory data to a unit-conversion service.

## Decision

Use `@lhncbc/ucum-lhc` 7.1.9 from the U.S. National Library of Medicine for automatic UCUM validation and conversion.

Keep the library behind the local measurement-normalization module. The module accepts a confirmed numeric value, source unit, analyte identity, and canonical unit. It returns a normalized value with the rule identifier, or a specific blocked reason.

Store the confirmed parsed numeric value as derived data. Keep the source value and source unit unchanged. Recalculate normalized views from the current analyte definition.

Block text results, non-quantitative results, arbitrary-unit properties, invalid units, incompatible units, and measurements without one analyte identity. Add reviewed rules separately for conversions that need analyte-specific molar mass.

## Consequences

The application can convert commensurable units without a network request. Each normalized value identifies `ucum-lhc@7.1.9:automatic` as its rule.

The UCUM definitions increase the desktop JavaScript bundle. The release build must measure the bundle. Plot benchmarks must use production normalization behavior.

The application does not convert mass concentration to substance concentration automatically. A separate reviewed rule needs the exact analyte identity, molar mass, and provenance.

## Rejected alternatives

Do not use the NLM web service. It would break offline use and could disclose health data.

Do not implement a local unit parser with regular expressions or a manual conversion table. This approach cannot prove UCUM validity or commensurability.

## References

- [NLM UCUM-LHC overview](https://ucum.nlm.nih.gov/ucum-lhc/)
- [UCUM specification](https://ucum.org/ucum)
- [Laboratory unit normalization research](../research/lab-unit-normalization.md)
