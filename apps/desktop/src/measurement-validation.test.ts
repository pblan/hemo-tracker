import { describe, expect, it } from "vitest";
import { validateMeasurementRow } from "./measurement-validation";

describe("validateMeasurementRow", () => {
  it("identifies each missing required field", () => {
    expect(
      validateMeasurementRow({
        sourceLabel: "",
        sourceValue: "",
        sourceUnit: "",
        sourceReferenceInterval: "",
        sourceFlag: "",
      }),
    ).toHaveLength(5);
  });
  it("accepts numeric, ordinal, and text source values", () => {
    for (const sourceValue of ["13,7", "positive", "trace"]) {
      expect(
        validateMeasurementRow({
          sourceLabel: "Result",
          sourceValue,
          sourceUnit: "unit",
          sourceReferenceInterval: "range",
          sourceFlag: "normal",
        }),
      ).toEqual([]);
    }
  });
});
