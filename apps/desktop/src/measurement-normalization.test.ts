import { describe, expect, it } from "vitest";

import {
  normalizeMeasurement,
  resolveApplicableTargetRange,
} from "./measurement-normalization";
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

  it("uses a reviewed glucose rule only for the exact identity", () => {
    const glucose = {
      ...analyte,
      id: "glucose",
      name: "Glucose",
      component: "Glucose",
      property: "MCnc",
      specimen: "Serum or plasma",
      loincCode: "2345-7",
      canonicalUnit: "mmol/L",
    };
    const glucoseMeasurement = {
      ...measurement,
      analyteId: "glucose",
      sourceLabel: "Glucose",
      sourceValue: "90",
      parsedNumericValue: "90",
      sourceUnit: "mg/dL",
    };
    const result = normalizeMeasurement(glucoseMeasurement, glucose);
    expect(result).toMatchObject({
      status: "normalized",
      unit: "mmol/L",
      ruleId: "curated:2345-7->14749-6:nist-srd-69:50-99-7",
    });
    expect(result.status === "normalized" ? result.value : 0).toBeCloseTo(
      4.99567,
      5,
    );
    expect(
      normalizeMeasurement(glucoseMeasurement, {
        ...glucose,
        specimen: "Whole blood",
      }),
    ).toEqual({ status: "blocked", reason: "incompatible-unit" });
  });

  it("uses the reviewed creatinine rule and blocks a near match", () => {
    const creatinine = {
      ...analyte,
      id: "creatinine",
      name: "Creatinine",
      component: "Creatinine",
      property: "MCnc",
      specimen: "Serum or plasma",
      loincCode: "2160-0",
      canonicalUnit: "umol/L",
    };
    const creatinineMeasurement = {
      ...measurement,
      analyteId: "creatinine",
      sourceLabel: "Creatinine",
      sourceValue: "1.2",
      parsedNumericValue: "1.2",
      sourceUnit: "mg/dL",
    };
    const result = normalizeMeasurement(creatinineMeasurement, creatinine);
    expect(result).toMatchObject({
      status: "normalized",
      unit: "umol/L",
      ruleId: "curated:2160-0->14682-9:nist-srd-69:60-27-5",
    });
    expect(result.status === "normalized" ? result.value : 0).toBeCloseTo(
      106.084006,
      5,
    );
    expect(
      normalizeMeasurement(creatinineMeasurement, {
        ...creatinine,
        loincCode: "38483-4",
      }),
    ).toEqual({ status: "blocked", reason: "incompatible-unit" });
  });

  it("normalizes one date-applicable personal target range", () => {
    expect(
      resolveApplicableTargetRange(
        "2026-06-01T08:00:00Z",
        [
          {
            id: "range-1",
            lowerBound: "12000",
            upperBound: "16000",
            unit: "mg/L",
            validFrom: "2026-01-01",
            validTo: "2026-12-31",
          },
        ],
        analyte,
      ),
    ).toEqual({
      status: "applicable",
      lowerBound: 1.2,
      upperBound: 1.6,
      unit: "g/dL",
      rangeId: "range-1",
    });
  });

  it("does not evaluate contextual or overlapping personal ranges", () => {
    const baseRange = {
      id: "range-1",
      lowerBound: "12",
      upperBound: "16",
      unit: "g/L",
    };
    expect(
      resolveApplicableTargetRange(
        "2026-06-01",
        [{ ...baseRange, context: "Fasting" }],
        analyte,
      ),
    ).toEqual({ status: "unavailable", reason: "context-unverified" });
    expect(
      resolveApplicableTargetRange(
        "2026-06-01",
        [baseRange, { ...baseRange, id: "range-2" }],
        analyte,
      ),
    ).toEqual({ status: "unavailable", reason: "ambiguous" });
  });
});
