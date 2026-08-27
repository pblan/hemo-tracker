# Laboratory unit normalization

Status: Research note. This document does not make an architecture decision.

## Scope

This note explains how a blood-result tracker can keep source values and also make safe normalized values. It uses primary standards and official terminology sources.

## Facts from standards

### A unit does not identify a laboratory result

LOINC identifies a laboratory observation with these parts:

- Component or analyte
- Property
- Time
- System or specimen
- Scale
- Method, when the method is relevant

For example, sodium in whole blood is not the same LOINC observation as sodium in serum or plasma. The analyte name alone is not enough. The specimen, property, scale, and method can change the meaning of a result. [LOINC term structure](https://loinc.org/kb/users-guide/major-parts-of-a-loinc-term) [LOINC content FAQ](https://loinc.org/kb/faq/content)

LOINC also separates different properties for the same component. These properties include mass concentration, substance concentration, catalytic concentration, number concentration, ratio, fraction, and entitic quantity. For example, a value per red blood cell is not the same property as a value per liter of blood. [LOINC property guide](https://loinc.org/kb/users-guide/major-parts-of-a-loinc-term/property)

### UCUM gives units a computable meaning

UCUM gives unit expressions a precise meaning. It uses dimensional analysis to find units that are commensurable. Two commensurable units can differ only by magnitude. A conforming implementation must compare the meaning of unit expressions. A text comparison is not sufficient. UCUM codes are case-sensitive, and white space is not part of a unit expression. [UCUM specification](https://ucum.org/ucum)

UCUM can normalize units in the same dimension. Examples include:

- `g/dL` and `mg/dL` for mass concentration
- `mmol/L` and `umol/L` for substance concentration
- `10*3/uL` and `10*9/L` for number concentration
- `%` and `1` for a fraction when the observation property confirms the representation

Some UCUM conversions need a conversion function and not only a multiplication factor. UCUM tells implementers to use the specification or a reference implementation for these conversions. [UCUM example unit terms](https://ucum.org/ucum#section-Example-Unit-Terms)

UCUM does not permit conversion or comparison between different arbitrary units. This rule applies since UCUM version 1.7. [UCUM arbitrary units](https://ucum.org/ucum#section-Semantics)

### Mass and substance concentration are different properties

Mass concentration uses a mass per volume, such as `mg/dL`. Substance concentration uses an amount of substance per volume, such as `mmol/L`. LOINC can assign different codes to these properties for the same component. Albumin in `g/dL` and albumin in `umol/L` are the official example. [LOINC content FAQ](https://loinc.org/kb/faq/content)

A conversion between mass concentration and substance concentration needs the molar mass of the exact measured substance. NIST defines molar mass as mass divided by amount of substance. The conversion is valid only when the chemical entity has a definite composition. [NIST Guide to the SI, chapter 8](https://www.nist.gov/pml/special-publication-811/nist-guide-si-chapter-8)

### Results are not always simple numbers

FHIR Observation permits many result types. These types include Quantity, CodeableConcept, string, boolean, integer, Range, and Ratio. FHIR also has a data-absent reason, an interpretation such as high or low, notes, specimen, method, and reference ranges. [HL7 FHIR R5 Observation](https://hl7.org/fhir/R5/observation.html)

FHIR Quantity keeps a decimal value, an optional comparator, a display unit, a unit code, and the code system. If a unit code is present, the code system must also be present. [HL7 FHIR R5 Quantity](https://hl7.org/fhir/R5/datatypes.html#Quantity)

LOINC distinguishes quantitative, semi-quantitative, ordinal, and nominal results. A titer such as `1:8` is semi-quantitative. A result such as positive or negative can express presence or a threshold. These results are not continuous numeric quantities. [LOINC property guide](https://loinc.org/kb/users-guide/major-parts-of-a-loinc-term/property)

### A reference range belongs to its source result

FHIR defines a reference range as a guide for the interpretation of an observation. The range can have low and high quantities. It can also have a type, a population, an age range, and text. FHIR keeps laboratory interpretation separate from the measured value. [HL7 FHIR R5 Observation](https://hl7.org/fhir/R5/observation.html)

This structure shows that one analyte can have more than one applicable range. It also shows that a text range can be necessary when numeric limits are not sufficient.

## Product recommendations

These recommendations follow from the facts above. They are not clinical rules.

### Keep the source fact and the normalized fact

Keep these fields for every entered result:

- Original analyte label
- Original value text
- Parsed result type
- Parsed numeric value, if present
- Comparator, such as `<` or `>`
- Original unit text
- Mapped UCUM code, if mapping is known
- Source reference range and source flag
- Specimen
- Method, if the report gives it
- LOINC code, if mapping is known
- Entry source and source document

Store a normalized value as derived data. Do not replace the original value or unit. Store the conversion rule version with the derived value.

### Use a result identity before a conversion

Define a comparable result series with at least:

`component + property + specimen + scale + method relevance`

Use a LOINC code when a confident mapping exists. Keep a local definition when no confident LOINC mapping exists. Do not merge series only because their display names match.

### Use three conversion classes

1. **Automatic**: UCUM says that the source and target units are commensurable, and the result identity is the same.
2. **Curated**: The conversion crosses from mass concentration to substance concentration. A reviewed rule specifies the exact component, molar mass, source property, target property, and provenance.
3. **Blocked**: The system cannot prove equivalence. Keep the source result and show it in its original unit.

Use a tested UCUM implementation. Do not implement unit parsing with a list of regular expressions.

### Block these automatic conversions

Block conversion or series merging in these cases:

- The component identity is unknown or ambiguous.
- The specimen differs, such as whole blood and serum or plasma.
- A relevant method differs or is not known.
- The properties differ and no reviewed cross-property rule exists.
- The result uses arbitrary units.
- The result is nominal, ordinal, textual, a titer, or a threshold category.
- The value has an unparsed comparator.
- The result is a calculated value and the formula or adjustment differs. An example is an estimated glomerular filtration rate with a named formula and body-surface-area adjustment.
- A fraction can mean either percent or decimal fraction and the property is not known.
- The chemical entity does not have one definite molar mass. Protein mixtures and method-defined measurands are examples.
- The result depends on calibration or assay-specific traceability that the application does not know.

Do not infer an analyte identity from the unit. Many different observations use the same unit.

### Treat healthy ranges as a different concept

Keep the laboratory reference range on each source result. Also permit user-defined target ranges on an analyte definition. Label them as user-defined target ranges, not laboratory reference ranges and not medical advice.

A target range needs these optional applicability fields:

- Specimen
- Method
- Unit and property
- Sex or other population criteria
- Minimum and maximum age
- Effective start and end date
- Source or note

Convert a target range only when the associated result conversion is safe. Do not change the source laboratory range when an analyte definition changes.

## Representative starter set

The starter set is a data-entry aid. It is not a fixed schema and it is not a clinical recommendation.

Start with the required members of the official LOINC automated CBC panel and the official LOINC comprehensive metabolic panel. These panels give representative examples of different properties and units. [LOINC CBC panel 58410-2](https://loinc.org/58410-2) [LOINC comprehensive metabolic panel 24323-8](https://loinc.org/24323-8)

Suggested initial definitions:

| Group | Component | Example property | Example UCUM unit |
| --- | --- | --- | --- |
| CBC | Leukocytes | Number concentration | `10*3/uL` |
| CBC | Erythrocytes | Number concentration | `10*6/uL` |
| CBC | Hemoglobin | Mass concentration | `g/dL` |
| CBC | Hematocrit | Volume fraction | `%` |
| CBC | MCV | Entitic mean volume | `fL` |
| CBC | MCH | Entitic mass | `pg` |
| CBC | MCHC | Entitic mass concentration | `g/dL` |
| CBC | Platelets | Number concentration | `10*3/uL` |
| Metabolic | Glucose | Mass concentration | `mg/dL` |
| Metabolic | Urea nitrogen | Mass concentration | `mg/dL` |
| Metabolic | Creatinine | Mass concentration | `mg/dL` |
| Metabolic | Sodium | Substance concentration | `mmol/L` |
| Metabolic | Potassium | Substance concentration | `mmol/L` |
| Metabolic | Chloride | Substance concentration | `mmol/L` |
| Metabolic | Carbon dioxide | Substance concentration | `mmol/L` |
| Metabolic | Calcium | Mass concentration | `mg/dL` |
| Metabolic | Total protein | Mass concentration | `g/dL` |
| Metabolic | Albumin | Mass concentration | `g/dL` |
| Metabolic | Total bilirubin | Mass concentration | `mg/dL` |
| Metabolic | Alkaline phosphatase | Catalytic concentration | `[IU]/L` |
| Metabolic | AST | Catalytic concentration | `[IU]/L` |
| Metabolic | ALT | Catalytic concentration | `[IU]/L` |

Add the exact LOINC term, specimen, property, and method status to each seed definition. Do not treat the component name in this table as a complete identity.

## Open questions for an ADR

- Which UCUM library and version will the project use?
- Will the canonical unit be global for each result identity or configurable per user?
- Who can approve a curated mass-to-substance conversion rule?
- How will the project version and recalculate derived normalized values?
- Will user-defined target ranges support population and method criteria in the first release?
- How will the UI show two results that have the same component name but are not comparable?
