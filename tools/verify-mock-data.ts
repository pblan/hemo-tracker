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
if (reportData.reports.length !== 10 || analyteData.analytes.length !== 14)
  throw new Error("unexpected v1 fixture size");
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
