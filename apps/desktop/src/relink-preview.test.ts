import { describe, expect, it } from "vitest";
import { previewRelinking } from "./relink-preview";

const target = {
  id: "hb",
  name: "Hemoglobin",
  component: "Hemoglobin",
  property: "MCnc",
  specimen: "Blood",
  scale: "Quantitative",
  aliases: ["Hb"],
  canonicalUnit: "g/dL",
  personalTargetRanges: [],
};

const measurement = {
  id: "m1",
  sourceLabel: "Hb",
  sourceValue: "13.7",
  sourceUnit: "g/dL",
  sourceReferenceInterval: "12-16",
  sourceFlag: "",
  parsedNumericValue: "13.7",
  updatedAt: "",
  updatedBy: "local-user",
};

describe("previewRelinking", () => {
  it("matches aliases and marks safely normalized candidates", () => {
    expect(
      previewRelinking(
        [
          {
            id: "report-1",
            collectionTime: "2026-01-01",
            status: "complete",
            tags: [],
            sourceFileCount: 0,
            measurementCount: 1,
            sourceFiles: [],
            measurements: [measurement],
          },
        ],
        target,
      ),
    ).toMatchObject([{ reportId: "report-1", status: "safe" }]);
  });

  it("marks ambiguous numeric links blocked", () => {
    expect(
      previewRelinking(
        [
          {
            id: "report-1",
            collectionTime: "2026-01-01",
            status: "complete",
            tags: [],
            sourceFileCount: 0,
            measurementCount: 1,
            sourceFiles: [],
            measurements: [{ ...measurement, sourceUnit: "bananas" }],
          },
        ],
        target,
      )[0],
    ).toMatchObject({ status: "blocked", reason: "invalid-unit" });
  });
});
