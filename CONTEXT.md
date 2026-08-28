# Laboratory Result Tracking

This context describes personal laboratory reports and longitudinal laboratory measurements. It keeps source facts separate from user guidance and derived display data.

## Language

**Lab report**:
A record of one laboratory event. It contains report metadata, source files, and measurements.
_Avoid_: Analysis, document, test result

**Source file**:
An original file that belongs to a lab report. One lab report can have multiple source files.
_Avoid_: Document, attachment

**Measurement**:
One recorded result in a lab report. It keeps the original value, unit, range, flag, and source label.
_Avoid_: Value, blood parameter, result row

**Analyte**:
The substance or property that a laboratory measured, such as hemoglobin.
_Avoid_: Parameter, marker

**Analyte identity**:
The structured identity that determines if measurements can form one series. It includes the component, property, specimen, scale, and a relevant method.
_Avoid_: Analyte name, label

**Analyte definition**:
The editable catalog entry that describes an analyte identity, aliases, display rules, and safe normalization rules.
_Avoid_: Parameter definition, schema

**Source value**:
The exact value text from a lab report. It does not change when the application normalizes or displays the measurement.
_Avoid_: Raw value

**Normalized value**:
A derived numeric value in a selected canonical unit. It exists only when a verified conversion is safe.
_Avoid_: Corrected value, converted source value

**Source reference interval**:
The interval that the source laboratory supplied for one measurement. It can depend on the laboratory, specimen, method, and reference population.
_Avoid_: Healthy range, normal range

**Personal target range**:
A dated range that the user sets for one analyte definition. It is guidance and does not replace the source reference interval.
_Avoid_: Healthy range, normal range

**Source flag**:
The interpretation that the source laboratory supplied, such as low, high, or abnormal.
_Avoid_: Application diagnosis, health status

**Comparability boundary**:
A point where a change in identity, unit, specimen, method, or laboratory can make measurements unsafe to connect or compare.
_Avoid_: Data break

**Draft report**:
A lab report for which manual entry is incomplete.
_Avoid_: Incomplete analysis

**Complete report**:
A lab report for which the user considers manual entry complete. The user can still add missing measurements later.
_Avoid_: Final report, frozen report

**Archived report**:
A lab report that the application hides from normal views without permanent deletion.
_Avoid_: Deleted report

**Demo report**:
A clearly tagged fictional report that a new local vault contains for interface
inspection. It is not health data and can be archived or permanently deleted.
_Avoid_: Sample result, test data

**Trusted device**:
A user device that holds an approved device key and can unlock the account vault.
_Avoid_: Logged-in device

**Account vault**:
The encrypted clinical dataset for one account. It includes lab reports, measurements, analyte definitions, personal target ranges, and source files.
_Avoid_: User database, server database
