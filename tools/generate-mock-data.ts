import { mkdir, writeFile } from "node:fs/promises";
import prettier from "prettier";

const root = new URL("../fixtures/v1/", import.meta.url);
const generatedAt = "2026-08-28T00:00:00Z";

const analyteInputs = [
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
  ["rbc", "Erythrocytes", "NCnc", "Blood", "10*6/uL", "789-8"],
  ["mcv", "Mean corpuscular volume", "Vol", "Blood", "fL", "787-2"],
  ["mch", "Mean corpuscular hemoglobin", "MCnc", "Blood", "pg", "785-6"],
  [
    "mchc",
    "Mean corpuscular hemoglobin concentration",
    "MCnc",
    "Blood",
    "g/dL",
    "786-4",
  ],
  ["rdw", "Erythrocyte distribution width", "Disp", "Blood", "%", "788-0"],
  ["neutrophils", "Neutrophils", "NCnc", "Blood", "10*3/uL", "751-8"],
  ["lymphocytes", "Lymphocytes", "NCnc", "Blood", "10*3/uL", "731-0"],
  ["sodium", "Sodium", "SCnc", "Serum or plasma", "mmol/L", "2951-2"],
  ["potassium", "Potassium", "SCnc", "Serum or plasma", "mmol/L", "2823-3"],
  ["calcium", "Calcium", "SCnc", "Serum or plasma", "mmol/L", "17861-6"],
  ["albumin", "Albumin", "MCnc", "Serum or plasma", "g/L", "1751-7"],
  [
    "bilirubin",
    "Bilirubin total",
    "MCnc",
    "Serum or plasma",
    "umol/L",
    "1975-2",
  ],
  ["alp", "Alkaline phosphatase", "CCat", "Serum or plasma", "U/L", "6768-6"],
  ["iron", "Iron", "MCnc", "Serum or plasma", "umol/L", "2498-4"],
  [
    "vitamin-d",
    "25-hydroxyvitamin D",
    "MCnc",
    "Serum or plasma",
    "nmol/L",
    "62292-8",
  ],
  ["custom-marker", "Example marker", "Arb", "Blood", "arb", undefined],
];

const analytes = analyteInputs.map(
  ([id, name, property, specimen, canonicalUnit, loincCode]) => ({
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
  }),
);

const reportDates = [
  "2025-01-14T08:10:00Z",
  "2025-03-29T07:45:00Z",
  "2025-06-02T09:20:00Z",
  "2025-08-18T08:05:00Z",
  "2025-11-30T10:15:00Z",
  "2026-01-09T07:55:00Z",
  "2026-03-21T09:40:00Z",
  "2026-06-30T08:25:00Z",
  "2026-07-04T08:30:00Z",
  "2026-07-18T08:35:00Z",
  "2026-08-20T09:05:00Z",
  "2026-08-26T08:45:00Z",
];
const valueProfiles: Record<
  string,
  { base: number; step: number; unit: string }
> = {
  hemoglobin: { base: 13.8, step: 0.03, unit: "g/dL" },
  hematocrit: { base: 42, step: 0.12, unit: "%" },
  wbc: { base: 6.4, step: 0.08, unit: "10*3/uL" },
  platelets: { base: 245, step: 1.8, unit: "10*3/uL" },
  glucose: { base: 90, step: 1.4, unit: "mg/dL" },
  creatinine: { base: 1, step: 0.01, unit: "mg/dL" },
  egfr: { base: 92, step: -0.5, unit: "mL/min/{1.73_m2}" },
  alt: { base: 24, step: 0.7, unit: "U/L" },
  ast: { base: 22, step: 0.45, unit: "U/L" },
  ferritin: { base: 85, step: 2.5, unit: "ug/L" },
  tsh: { base: 2.1, step: 0.04, unit: "mIU/L" },
  crp: { base: 1.8, step: 0.22, unit: "mg/L" },
  cholesterol: { base: 5.2, step: 0.03, unit: "mmol/L" },
  rbc: { base: 4.7, step: 0.01, unit: "10*6/uL" },
  mcv: { base: 89, step: 0.2, unit: "fL" },
  mch: { base: 29.4, step: 0.08, unit: "pg" },
  mchc: { base: 33.1, step: 0.05, unit: "g/dL" },
  rdw: { base: 13.1, step: 0.04, unit: "%" },
  neutrophils: { base: 3.6, step: 0.05, unit: "10*3/uL" },
  lymphocytes: { base: 2.1, step: 0.03, unit: "10*3/uL" },
  sodium: { base: 140, step: 0.12, unit: "mmol/L" },
  potassium: { base: 4.2, step: 0.02, unit: "mmol/L" },
  calcium: { base: 2.35, step: 0.01, unit: "mmol/L" },
  albumin: { base: 44, step: -0.1, unit: "g/L" },
  bilirubin: { base: 12, step: 0.2, unit: "umol/L" },
  alp: { base: 72, step: 0.8, unit: "U/L" },
  iron: { base: 18, step: 0.3, unit: "umol/L" },
  "vitamin-d": { base: 74, step: 0.8, unit: "nmol/L" },
};
const referenceIntervals: Record<string, string> = {
  hemoglobin: "12.0-16.0 g/dL",
  hematocrit: "36-46 %",
  wbc: "4.0-10.0 10*3/uL",
  platelets: "150-400 10*3/uL",
  glucose: "70-99 mg/dL",
  creatinine: "0.6-1.2 mg/dL",
  egfr: ">=60 mL/min/{1.73_m2}",
  alt: "7-35 U/L",
  ast: "10-35 U/L",
  ferritin: "20-250 ug/L",
  tsh: "0.4-4.0 mIU/L",
  crp: "0-5 mg/L",
  cholesterol: "<5.0 mmol/L",
  sodium: "135-145 mmol/L",
  potassium: "3.5-5.1 mmol/L",
  calcium: "2.15-2.55 mmol/L",
  albumin: "35-50 g/L",
  bilirubin: "3-20 umol/L",
  "vitamin-d": "50-125 nmol/L",
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
    const profile = valueProfiles[analyte.id];
    const isLateAnalyte =
      ["vitamin-d", "custom-marker"].includes(analyte.id) && reportIndex < 3;
    const isMissing = reportIndex === 6 && analyte.id === "ferritin";
    const isComparator = reportIndex === 3 && analyte.id === "vitamin-d";
    const isTextual = reportIndex === 4 && analyte.id === "custom-marker";
    const isCorrected = reportIndex === 9 && analyte.id === "hemoglobin";
    const unit =
      analyte.id === "glucose" && reportIndex % 2
        ? "mmol/L"
        : analyte.id === "creatinine" && reportIndex % 3 === 1
          ? "umol/L"
          : (profile?.unit ?? analyte.canonicalUnit);
    const storyMultiplier =
      reportIndex >= 3 &&
      reportIndex <= 4 &&
      ["crp", "wbc", "neutrophils", "alt", "ast"].includes(analyte.id)
        ? 1.55
        : reportIndex === 5 &&
            ["hemoglobin", "hematocrit", "rbc"].includes(analyte.id)
          ? 0.9
          : 1;
    const numericValue = profile
      ? (profile.base + profile.step * reportIndex) * storyMultiplier
      : undefined;
    const alternateValue =
      analyte.id === "glucose" && unit === "mmol/L"
        ? numericValue
          ? (numericValue / 18.01559).toFixed(5)
          : ""
        : analyte.id === "creatinine" && unit === "umol/L"
          ? numericValue
            ? (numericValue * 88.4).toFixed(1)
            : ""
          : numericValue === undefined
            ? "not reported"
            : numericValue.toFixed(2).replace(/\.00$/, "");
    const value =
      isLateAnalyte || isMissing
        ? ""
        : isComparator
          ? "<50"
          : isTextual
            ? "not reported"
            : isCorrected
              ? "14.1"
              : alternateValue;
    const parsedNumericValue =
      isLateAnalyte ||
      isMissing ||
      isComparator ||
      isTextual ||
      value === "not reported"
        ? undefined
        : value.replace(",", ".");
    return {
      id: `fixture-measurement-${reportIndex + 1}-${String(analyteIndex + 1).padStart(2, "0")}`,
      analyteId: analyte.id,
      sourceLabel: analyte.name,
      sourceValue: value,
      sourceUnit: isMissing ? "" : unit,
      sourceReferenceInterval:
        analyte.id === "hemoglobin" && reportIndex >= 5
          ? "11.5-15.5 g/dL"
          : (referenceIntervals[analyte.id] ?? "Not supplied"),
      parsedNumericValue,
      sourceFlag:
        isMissing || isLateAnalyte
          ? "not available"
          : isComparator
            ? "below detection limit"
            : analyte.id === "crp" && storyMultiplier > 1
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
const json = (value: unknown) =>
  prettier.format(JSON.stringify(value), { parser: "json" });
await writeFile(
  new URL("analytes.json", root),
  await json({ fixtureVersion: "v1.0.0", generatedAt, analytes }),
);
await writeFile(
  new URL("reports.json", root),
  await json({ fixtureVersion: "v1.0.0", generatedAt, reports }),
);
await writeFile(
  new URL("expected-normalization.json", root),
  await json({
    glucoseMgDlToMmolL: 4.99567,
    creatinineMgDlToUmolL: 88.4,
    ruleIds: [
      "curated:2345-7->14749-6:nist-srd-69:50-99-7",
      "curated:2160-0->14682-9:nist-srd-69:60-27-5",
    ],
  }),
);
await writeFile(
  new URL("expected-trends.json", root),
  await json({
    reportCount: reports.length,
    analyteCount: analytes.length,
    irregularDates: true,
    missingMeasurement: "fixture-measurement-7-10",
    comparatorMeasurement: "fixture-measurement-4-29",
    sparseAnalyte: "vitamin-d",
    correctedMeasurement: "fixture-measurement-10-01",
    archivedReport: "fixture-report-005",
    draftReport: "fixture-report-010",
    storyCoverage: [
      "stable-baseline",
      "deviation-and-recovery",
      "alternate-units",
      "changed-reference-interval",
      "missing-and-non-numeric",
      "correction-provenance",
      "report-states-and-source-files",
      "cross-panel-cbc",
      "sparse-series",
    ],
  }),
);
