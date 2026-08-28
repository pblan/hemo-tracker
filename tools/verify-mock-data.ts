import { readFile } from "node:fs/promises";

const root = new URL("../fixtures/v1/", import.meta.url);
const analyteData = JSON.parse(
  await readFile(new URL("analytes.json", root), "utf8"),
);
const reportData = JSON.parse(
  await readFile(new URL("reports.json", root), "utf8"),
);
const analyteIds = new Set<string>();
for (const analyte of analyteData.analytes) {
  if (analyteIds.has(analyte.id))
    throw new Error(`duplicate analyte id: ${analyte.id}`);
  analyteIds.add(analyte.id);
}
const reportIds = new Set<string>();
for (const report of reportData.reports) {
  if (reportIds.has(report.id))
    throw new Error(`duplicate report id: ${report.id}`);
  reportIds.add(report.id);
  for (const measurement of report.measurements) {
    if (!analyteIds.has(measurement.analyteId))
      throw new Error(`unknown analyte: ${measurement.analyteId}`);
  }
  for (const source of report.sourceFiles) {
    if (!(
      source.role === "primary" ||
      source.role === "supplement" ||
      source.role === "correction"
    ))
      throw new Error(`invalid source role: ${source.role}`);
  }
}
if (reportData.reports.length < 12 || analyteData.analytes.length < 25)
  throw new Error("fixture must contain at least 25 analytes and 12 reports");
const measurementsByAnalyte = new Map<string, number>();
for (const report of reportData.reports) {
  for (const measurement of report.measurements) {
    if (measurement.parsedNumericValue !== undefined)
      measurementsByAnalyte.set(
        measurement.analyteId,
        (measurementsByAnalyte.get(measurement.analyteId) ?? 0) + 1,
      );
  }
}
for (const analyte of analyteData.analytes) {
  if (
    analyte.id !== "custom-marker" &&
    analyte.id !== "vitamin-d" &&
    (measurementsByAnalyte.get(analyte.id) ?? 0) < 3
  )
    throw new Error(
      `analyte has fewer than three numeric points: ${analyte.id}`,
    );
}
if (
  !reportData.reports.some(
    (report: {
      measurements: Array<{
        analyteId: string;
        sourceValue: string;
        parsedNumericValue?: string;
      }>;
    }) =>
      report.measurements.some(
        (measurement) =>
          measurement.sourceValue.startsWith("<") &&
          measurement.parsedNumericValue === undefined,
      ),
  )
)
  throw new Error("missing comparator value story");
if (
  !reportData.reports.some(
    (report: {
      measurements: Array<{ analyteId: string; sourceValue: string }>;
    }) =>
      report.measurements.some(
        (measurement) =>
          measurement.analyteId === "vitamin-d" &&
          measurement.sourceValue === "",
      ),
  )
)
  throw new Error("missing sparse analyte story");
if (
  !reportData.reports.some(
    (report: { status: string }) => report.status === "archived",
  )
)
  throw new Error("missing archived report");
if (
  !reportData.reports.some(
    (report: { status: string }) => report.status === "draft",
  )
)
  throw new Error("missing draft report");
console.log(
  `Verified ${analyteIds.size} analytes and ${reportIds.size} reports.`,
);
