import { describe, expect, it } from "vitest";

import { normalizeMeasurement } from "./measurement-normalization";
import type { AnalyteDefinition, ReportSummary } from "./vault-client";

const analyte: AnalyteDefinition = {
  id: "hemoglobin",
  name: "Hemoglobin",
  component: "Hemoglobin",
  property: "MCnc",
  specimen: "Blood",
  scale: "Quantitative",
  aliases: [],
  canonicalUnit: "g/dL",
  personalTargetRanges: [],
};

const measurement: ReportSummary["measurements"][number] = {
  id: "measurement-1",
  sourceLabel: "Hemoglobin",
  sourceValue: "138",
  sourceUnit: "g/L",
  sourceReferenceInterval: "120-160",
  sourceFlag: "",
  parsedNumericValue: "138",
  analyteId: "hemoglobin",
  updatedAt: "",
  updatedBy: "local-user",
};

describe("normalizeMeasurement", () => {
  it("uses UCUM for commensurable units and identifies the rule", () => {
    expect(normalizeMeasurement(measurement, analyte)).toEqual({
      status: "normalized",
      value: 13.8,
      unit: "g/dL",
      ruleId: "ucum-lhc@7.1.9:automatic",
    });
  });

  it("blocks incompatible and arbitrary units", () => {
    expect(
      normalizeMeasurement({ ...measurement, sourceUnit: "mmol/L" }, analyte),
    ).toEqual({ status: "blocked", reason: "incompatible-unit" });
    expect(
      normalizeMeasurement(measurement, { ...analyte, property: "Arb" }),
    ).toEqual({ status: "blocked", reason: "arbitrary-unit" });
  });

  it("blocks text values and unknown units", () => {
    expect(
      normalizeMeasurement(
        { ...measurement, parsedNumericValue: undefined },
        analyte,
      ),
    ).toEqual({ status: "blocked", reason: "non-numeric-value" });
    expect(
      normalizeMeasurement({ ...measurement, sourceUnit: "bananas" }, analyte),
    ).toEqual({ status: "blocked", reason: "invalid-unit" });
  });
});
