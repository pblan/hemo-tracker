import { mkdir, writeFile } from "node:fs/promises";

const root = new URL("../fixtures/v1/", import.meta.url);
const generatedAt = "2026-08-28T00:00:00Z";

const analytes = [
  ["hemoglobin", "Hemoglobin", "MCnc", "Blood", "g/dL", "718-7"],
  ["hematocrit", "Hematocrit", "VFr", "Blood", "%", "4544-3"],
  ["wbc", "Leukocytes", "NCnc", "Blood", "10*3/uL", "6690-2"],
  ["platelets", "Platelets", "NCnc", "Blood", "10*3/uL", "777-3"],
  ["glucose", "Glucose", "MCnc", "Serum or plasma", "mmol/L", "2345-7"],
  ["creatinine", "Creatinine", "MCnc", "Serum or plasma", "umol/L", "2160-0"],
  ["egfr", "eGFR", "ArVRat", "Serum or plasma", "mL/min/{1.73_m2}", "62238-1"],
  [
    "alt",
    "Alanine aminotransferase",
    "CCat",
    "Serum or plasma",
    "U/L",
    "1742-6",
  ],
  [
    "ast",
    "Aspartate aminotransferase",
    "CCat",
    "Serum or plasma",
    "U/L",
    "1920-8",
  ],
  ["ferritin", "Ferritin", "MCnc", "Serum or plasma", "ug/L", "2276-4"],
  ["tsh", "Thyrotropin", "MCnc", "Serum or plasma", "mIU/L", "3016-3"],
  ["crp", "C reactive protein", "MCnc", "Serum or plasma", "mg/L", "1988-5"],
  ["cholesterol", "Cholesterol", "MCnc", "Serum or plasma", "mmol/L", "2093-3"],
  ["custom-marker", "Example marker", "Arb", "Blood", "arb", undefined],
].map(([id, name, property, specimen, canonicalUnit, loincCode]) => ({
  id,
  name,
  component: name,
  property,
  specimen,
  scale: property === "Arb" ? "Ordinal" : "Quantitative",
  canonicalUnit,
  ...(loincCode ? { loincCode } : {}),
  aliases: [],
  personalTargetRanges:
    id === "hemoglobin"
      ? [
          {
            id: "fixture-range-hgb-1",
            lowerBound: "12",
            upperBound: "16",
            unit: "g/dL",
            validFrom: "2025-01-01",
            validTo: "2026-06-30",
            notes: "Fictional personal target.",
          },
          {
            id: "fixture-range-hgb-2",
            lowerBound: "120",
            upperBound: "160",
            unit: "g/L",
            validFrom: "2026-07-01",
            context: "Fasting; manual review required.",
          },
        ]
      : id === "glucose"
        ? [
            {
              id: "fixture-range-glucose-1",
              lowerBound: "4.0",
              upperBound: "6.0",
              unit: "mmol/L",
              notes: "Fictional personal target.",
            },
          ]
        : [],
}));

const reportDates = [
  "2025-01-14T08:10:00Z",
  "2025-03-29T07:45:00Z",
  "2025-06-02T09:20:00Z",
  "2025-08-18T08:05:00Z",
  "2025-11-30T10:15:00Z",
  "2026-01-09T07:55:00Z",
  "2026-03-21T09:40:00Z",
  "2026-06-30T08:25:00Z",
  "2026-07-18T08:35:00Z",
  "2026-08-20T09:05:00Z",
];
const values: Record<string, [string, string]> = {
  hemoglobin: ["13.8", "g/dL"],
  hematocrit: ["42", "%"],
  wbc: ["6.4", "10*3/uL"],
  platelets: ["245", "10*3/uL"],
  glucose: ["95", "mg/dL"],
  creatinine: ["1.0", "mg/dL"],
  egfr: ["92", "mL/min/{1.73_m2}"],
  alt: ["24", "U/L"],
  ast: ["22", "U/L"],
  ferritin: ["85", "ug/L"],
  tsh: ["2.1", "mIU/L"],
  crp: ["1.8", "mg/L"],
  cholesterol: ["5.2", "mmol/L"],
  "custom-marker": ["not reported", "not applicable"],
};

const reports = reportDates.map((collectionTime, reportIndex) => ({
  id: `fixture-report-${String(reportIndex + 1).padStart(3, "0")}`,
  collectionTime,
  reportDate: collectionTime.slice(0, 10),
  laboratory:
    reportIndex % 3 === 0
      ? "Fictional Specialist Laboratory"
      : "Fictional Central Laboratory",
  orderingClinician: "Dr. Example",
  fastingState: reportIndex % 2 ? "unknown" : "fasting",
  status:
    reportIndex === 4 ? "archived" : reportIndex === 9 ? "draft" : "complete",
  notes: "Fictional test data. Do not use for clinical decisions.",
  tags: [reportIndex % 2 ? "follow-up" : "routine"],
  sourceFiles: [
    {
      id: `fixture-source-${reportIndex + 1}-primary`,
      filename: `demo-report-${collectionTime.slice(0, 10)}.pdf`,
      mediaType: "application/pdf",
      role: "primary",
    },
    ...(reportIndex === 9
      ? [
          {
            id: `fixture-source-${reportIndex + 1}-supplement`,
            filename: `demo-report-${collectionTime.slice(0, 10)}-supplement.png`,
            mediaType: "image/png",
            role: "supplement",
          },
          {
            id: `fixture-source-${reportIndex + 1}-correction`,
            filename: `demo-report-${collectionTime.slice(0, 10)}-correction.heic`,
            mediaType: "image/heic",
            role: "correction",
          },
        ]
      : []),
  ],
  measurements: analytes.map((analyte, analyteIndex) => {
    const [baseValue, baseUnit] = values[analyte.id] ?? [
      "0",
      analyte.canonicalUnit,
    ];
    const isMissing = reportIndex === 6 && analyte.id === "ferritin";
    const isCorrected = reportIndex === 9 && analyte.id === "hemoglobin";
    const value = isMissing ? "" : isCorrected ? "14.1" : baseValue;
    const unit =
      analyte.id === "glucose" && reportIndex % 2 ? "mmol/L" : baseUnit;
    return {
      id: `fixture-measurement-${reportIndex + 1}-${String(analyteIndex + 1).padStart(2, "0")}`,
      analyteId: analyte.id,
      sourceLabel: analyte.name,
      sourceValue: value,
      sourceUnit: isMissing ? "" : unit,
      sourceReferenceInterval:
        analyte.id === "hemoglobin"
          ? reportIndex < 5
            ? "12-16 g/dL"
            : "11.5-15.5 g/dL"
          : "Fictional interval",
      parsedNumericValue:
        isMissing || value === "not reported"
          ? undefined
          : value.replace(",", "."),
      sourceFlag: isMissing
        ? "not available"
        : analyte.id === "crp"
          ? "high"
          : analyte.id === "egfr"
            ? "low"
            : "",
      updatedAt: isCorrected ? generatedAt : "",
      updatedBy: isCorrected ? "fixture-reviewer" : "",
    };
  }),
}));

await mkdir(root, { recursive: true });
await writeFile(
  new URL("analytes.json", root),
  `${JSON.stringify({ fixtureVersion: "v1.0.0", generatedAt, analytes }, null, 2)}\n`,
);
await writeFile(
  new URL("reports.json", root),
  `${JSON.stringify({ fixtureVersion: "v1.0.0", generatedAt, reports }, null, 2)}\n`,
);
await writeFile(
  new URL("expected-normalization.json", root),
  `${JSON.stringify({ glucoseMgDlToMmolL: 4.99567, creatinineMgDlToUmolL: 88.4, ruleIds: ["curated:2345-7->14749-6:nist-srd-69:50-99-7", "curated:2160-0->14682-9:nist-srd-69:60-27-5"] }, null, 2)}\n`,
);
await writeFile(
  new URL("expected-trends.json", root),
  `${JSON.stringify({ reportCount: reports.length, analyteCount: analytes.length, irregularDates: true, missingMeasurement: "fixture-measurement-7-10", archivedReport: "fixture-report-005", draftReport: "fixture-report-010" }, null, 2)}\n`,
);
