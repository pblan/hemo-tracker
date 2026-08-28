import { describe, expect, it } from "vitest";
import analyteFixture from "../../../fixtures/v1/analytes.json";
import reportFixture from "../../../fixtures/v1/reports.json";
import expectedFixture from "../../../fixtures/v1/expected-normalization.json";
import { normalizeMeasurement } from "./measurement-normalization";

describe("V1 fictional fixture", () => {
  it("contains the documented coverage and safe glucose conversion", () => {
    expect(analyteFixture.analytes.length).toBeGreaterThanOrEqual(25);
    expect(reportFixture.reports.length).toBeGreaterThanOrEqual(12);
    const glucose = analyteFixture.analytes.find(
      (item) => item.id === "glucose",
    );
    const report = reportFixture.reports[0];
    const glucoseMeasurement = report?.measurements.find(
      (item) => item.analyteId === "glucose",
    );
    expect(glucose).toBeDefined();
    expect(glucoseMeasurement).toBeDefined();
    const source = JSON.parse(JSON.stringify(glucoseMeasurement));
    const result = normalizeMeasurement(glucoseMeasurement!, glucose!);
    expect(result).toMatchObject({
      status: "normalized",
      ruleId: expectedFixture.ruleIds[0],
    });
    expect(result.status === "normalized" ? result.value : 0).toBeCloseTo(
      expectedFixture.glucoseMgDlToMmolL,
      4,
    );
    expect(glucoseMeasurement).toEqual(source);
  });

  it("keeps missing and non-numeric fixture results out of trends", () => {
    const ferritin = analyteFixture.analytes.find(
      (item) => item.id === "ferritin",
    );
    const report = reportFixture.reports[6];
    const missing = report?.measurements.find(
      (item) => item.analyteId === "ferritin",
    );
    const custom = report?.measurements.find(
      (item) => item.analyteId === "custom-marker",
    );
    expect(normalizeMeasurement(missing!, ferritin!)).toEqual({
      status: "blocked",
      reason: "non-numeric-value",
    });
    expect(
      normalizeMeasurement(
        custom!,
        analyteFixture.analytes.find((item) => item.id === "custom-marker"),
      ),
    ).toEqual({
      status: "blocked",
      reason: "non-numeric-value",
    });
  });
});
