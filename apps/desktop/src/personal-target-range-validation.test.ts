import { describe, expect, it } from "vitest";

import { validatePersonalTargetRange } from "./personal-target-range-validation";

const validDraft = {
  analyteId: "hemoglobin",
  lowerBound: "120",
  upperBound: "180",
  unit: "g/L",
  validFrom: "2026-01-01",
  validTo: "",
};

describe("validatePersonalTargetRange", () => {
  it("accepts decimal commas and an open-ended validity period", () => {
    expect(
      validatePersonalTargetRange({
        ...validDraft,
        lowerBound: "3,9",
        upperBound: "5,5",
      }),
    ).toEqual([]);
  });

  it("rejects incomplete, inverted, and invalid ranges", () => {
    expect(
      validatePersonalTargetRange({
        ...validDraft,
        analyteId: "",
        lowerBound: "180",
        upperBound: "120",
        unit: "",
        validFrom: "2027-01-01",
        validTo: "2026-01-01",
      }),
    ).toEqual([
      "Select an analyte.",
      "Enter the unit for this range.",
      "The lower limit must not exceed the upper limit.",
      "The valid-from date must not follow the valid-to date.",
    ]);
  });
});
