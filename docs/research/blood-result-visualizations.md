# Blood Result Visualizations

## Scope

This note reviews displays for longitudinal blood and laboratory results.
It separates source evidence from product recommendations.
It does not give medical advice.

## Source evidence

### A laboratory report is not one time-series point

HL7 FHIR separates a diagnostic report from its atomic observations.
A report gives the clinical and workflow context.
Each observation gives one result or one component result.
An observation can contain a value, a unit, a clinically relevant time, an interpretation, a method, a specimen, and one or more reference ranges.
A reference range can apply to a specific population or age range.
An interpretation can state normal, abnormal, low, or high.
FHIR also supports a reason for an absent value.
See [HL7 FHIR Observation](https://www.hl7.org/fhir/observation.html) and [HL7 FHIR DiagnosticReport](https://hl7.org/fhir/diagnosticreport.html).

The US Core laboratory profile requires a result status and a test code.
It requires a value and a UCUM unit for a numeric value.
It also expects the observation time and the reference range when these data are available.
See [US Core Laboratory Result Observation](https://www.hl7.org/fhir/us/core/StructureDefinition-us-core-observation-lab.html).

LOINC identifies a test with up to six parts.
These parts include the analyte, property, time, specimen, scale, and method.
The method is material when it changes the clinical meaning or the reference range.
Thus, two results with the same common name do not always define one comparable series.
See [LOINC test structure](https://loinc.org/kb/faq/structure) and [LOINC method guidance](https://loinc.org/kb/users-guide/major-parts-of-a-loinc-term/method).

### Time-series plots are the main longitudinal display

A study compared laboratory graphs in eight electronic health record systems.
The study defined 11 criteria from the literature and expert review.
No system met all criteria.
One system put results in reverse time order.
One system used equal spacing for samples that had unequal time intervals.
That display changed the apparent slope and created a patient safety risk.
No system put both the analyte name and the unit on the y-axis.
See [Graphical display of diagnostic test results in electronic health records](https://pmc.ncbi.nlm.nih.gov/articles/PMC4482275/).

A controlled study used three displays for 28 laboratory tests.
Each display offered a result overview and a longitudinal detail view.
The displays also let users compare two tests.
The study kept the same time span across plots in one scenario.
This choice reduced errors from scale changes.
See [Presentation of laboratory test results in patient portals](https://pmc.ncbi.nlm.nih.gov/articles/PMC5809992/).

### Reference intervals need context

The CLSI EP28 standard defines how laboratories establish and verify reference intervals.
It covers subject selection, pre-analytical factors, analytical factors, calculation, transfer, presentation, and use.
It also covers a new method for an existing analyte.
See [CLSI EP28](https://clsi.org/shop/standards/ep28/).

HL7 permits more than one reference range for one result.
Each range can have a type, an applicable population, an age range, and free text.
A range can have only one bound.
Therefore, a display must not treat all ranges as one fixed global band.
See [HL7 FHIR Observation definitions](https://hl7.org/fhir/R5/observation-definitions.html).

The population reference interval does not measure change in one person.
Biological variation describes random variation around a person's homeostatic set point.
A reference change value estimates whether the difference between two results can come from analytical and within-person biological variation.
Reference change values can have asymmetric limits.
The evidence base also has limits and needs quality assessment.
See [Biological variation: recent development and future challenges](https://www.eflm.eu/upload/publications/2034-2022-ClinChemLabMed-Sandberg-et-al.pdf.pdf).

### Units and methods can break a trend

Laboratories can use different methods, units, and reference intervals.
Their results are not always directly comparable.
Users can still assume that results from different laboratories and dates are comparable.
This assumption can cause harm.
See [Harmonization of Clinical Laboratory Test Results](https://pmc.ncbi.nlm.nih.gov/articles/PMC4975212/).

LOINC distinguishes mass concentration from substance concentration.
For example, mg/dL and mmol/L do not only use different labels.
They express different kinds of quantity and need an analyte-specific conversion.
See [LOINC property guidance](https://loinc.org/kb/users-guide/major-parts-of-a-loinc-term/property).

UCUM gives units a machine-readable form.
It supports validated unit conversion.
It also states that many arbitrary biological units are method-dependent and are not convertible.
See [The Unified Code for Units of Measure](https://ucum.org/ucum).

### A reference interval is not a decision limit

A reference interval describes values from a defined reference population.
A clinical decision limit supports a clinical decision for a defined condition.
These two limits can have different sources and uses.
A value outside a reference interval does not by itself show disease.
See [Distinguishing reference intervals and clinical decision limits](https://pubmed.ncbi.nlm.nih.gov/30047297/).

### Result search and omission need visible rules

The US ONC SAFER guide recommends a longitudinal graph for laboratory results.
It recommends search across different test names and performing entities, with a code such as LOINC.
It also recommends a warning when the system excludes a result from a longitudinal display because its name or performing entity differs.
See [SAFER Test Results Reporting and Follow-Up](https://www.healthit.gov/sites/default/files/playbook/pdf/8-test-results-reporting-final.pdf).

### Abnormal flags and color have limits

Horizontal range bars can help users see where one result is relative to a standard range.
Other studies found better perceived usefulness and lower perceived urgency for near-normal results than with tables.
However, one controlled study found no improvement in correct risk interpretation from color, range cues, or grouping.
In that study, 65% of participants underestimated the need for action at least once.
See [Presentation of laboratory test results in patient portals](https://pmc.ncbi.nlm.nih.gov/articles/PMC5809992/).

The FHIR interpretation field is categorical.
It can preserve the source assessment, such as low, high, normal, or abnormal.
The source laboratory can make this assessment with information that a consumer application does not have.
See [HL7 FHIR Observation](https://www.hl7.org/fhir/observation.html).

## Product recommendations

The sources do not define one complete product architecture.
The items below are design recommendations from the evidence.

### 1. Use an overview and detail structure

Use a table or compact result list for one analysis document.
Show the measured value, source unit, source flag, source reference interval, and collection time.
Do not replace the exact values with a graph.

Open one analyte in a detail view.
Use a time-series point chart as the primary plot.
Connect points only when the connection does not imply continuous measurement.
Use the actual collection time on a left-to-right time axis.
Show the analyte name and unit on the y-axis.
Show exact values in an accessible data table and in point details.

Use small multiples for several analytes.
Align their time axes when users compare events.
Keep separate y-axes and units.
Do not put unrelated analytes on one numeric axis.

### 2. Draw the reference interval for each observation

Show a reference band behind the series when the source gives a numeric interval.
Bind each band segment to the result and source laboratory that supplied it.
Start a new band segment when the source interval changes.
Show one-sided intervals as one-sided limits.
Show textual or qualitative ranges as text.
Show a clinical decision limit with a different style and label.
Do not merge it with the reference band.

Keep the original source flag.
If the application calculates another flag, label it as calculated.
Do not use color as the only signal.
Use text, symbols, line styles, or patterns with color.
Use a separate style for critical results.
Do not infer a critical result from a normal reference interval.

### 3. Preserve comparability boundaries

Store the source unit, method, specimen, laboratory, and reference interval with each result.
Also store a normalized value only when a verified conversion exists.
Do not overwrite the source value.
Use UCUM codes for machine conversion when possible.
Do not convert arbitrary method-dependent units.

Mark a series boundary when the unit, specimen, method, or laboratory changes.
Do not connect results across that boundary unless the system can verify comparability.
Show the cause of the boundary in the chart and table.

### 4. Treat change metrics as derived evidence

Show absolute change and percentage change only for comparable numeric results.
Label the two points that define the change.
Do not classify a change as significant from the population reference interval.

Add a reference change value only when the system has a valid source for biological and analytical variation.
Store the formula, inputs, source, and version.
Show that the value is an estimate.
Do not present this estimate as a diagnosis.

### 5. Limit correlation views

A scatter plot can help a user explore two numeric analytes.
Each point must represent measurements from the same analysis or a defined time window.
The display must state the matching rule.
Show both units and the sample count.

Treat correlation as exploratory.
Do not imply causation.
Do not calculate a correlation for mixed methods, mixed specimens, very small samples, or unmatched dates without a clear warning.
Keep the two aligned time-series plots near the scatter plot.
This view helps users inspect time and outliers.

### 6. Use distribution views only for enough comparable data

A dot plot, box plot, or histogram can summarize repeated results for one analyte.
Show the raw points when the sample is small.
State the date range, sample count, unit, and inclusion rules.
Split the data when comparability changes.

Do not use a distribution plot as the main clinical view.
It removes time order and can hide a recent change.
Do not fit a normal distribution by default.
Laboratory result distributions can be skewed.

### 7. Keep safety information close to the data

State that a reference interval is not a diagnosis or a treatment target.
Show source comments and clinician interpretations when they exist.
Show missing, pending, corrected, and cancelled states explicitly.
Keep corrected results and their audit history.
Do not silently replace a released result.

Do not calculate urgency from color or distance outside a range.
The cited patient study shows that visual cues alone do not give safe risk interpretation.
Provide clear guidance for urgent or critical source flags.
Use text that a clinician or approved policy supplies.

## Suggested display layers

1. The analysis view shows one document and all source results.
2. The parameter view shows one comparable time series and its changing reference intervals.
3. The comparison view shows aligned small multiples for selected parameters.
4. The exploration view contains optional scatter plots and distribution plots.
5. Every view links back to the source document and source metadata.

This structure keeps the source record separate from derived displays.
It also permits a new parameter definition to include old documents after manual data entry.

## Open evidence questions

- The sources do not define a safe automatic urgency rule for arbitrary analytes.
- The sources do not support one universal normal-range color scheme.
- The sources do not define one correlation threshold for clinical use.
- The sources do not justify pooling results across methods or laboratories by default.
- The sources do not remove the need for clinical review.
