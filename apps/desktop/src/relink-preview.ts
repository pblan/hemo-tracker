import { normalizeMeasurement } from "./measurement-normalization";
import type { AnalyteDefinition, ReportSummary } from "./vault-client";

export type RelinkCandidate = {
  reportId: string;
  measurement: ReportSummary["measurements"][number];
  status: "safe" | "blocked";
  reason?: string;
};

export function previewRelinking(
  reports: ReportSummary[],
  target: AnalyteDefinition,
): RelinkCandidate[] {
  const labels = new Set(
    [target.name, ...target.aliases].map((value) => value.trim().toLowerCase()),
  );
  return reports.flatMap((report) =>
    report.measurements.flatMap((measurement) => {
      if (
        measurement.analyteId === target.id ||
        !labels.has(measurement.sourceLabel.trim().toLowerCase())
      )
        return [];
      const normalized = normalizeMeasurement(
        { ...measurement, analyteId: target.id },
        target,
      );
      return [
        {
          reportId: report.id,
          measurement,
          status: normalized.status === "normalized" ? "safe" : "blocked",
          ...(normalized.status === "blocked"
            ? { reason: normalized.reason }
            : {}),
        },
      ];
    }),
  );
}
