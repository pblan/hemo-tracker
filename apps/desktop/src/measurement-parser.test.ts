import { describe, expect, it } from "vitest";
import { parseMeasurementInput } from "./measurement-parser";

describe("parseMeasurementInput", () => {
  it("normalizes German decimals", () =>
    expect(parseMeasurementInput("13,7", "de-DE")).toMatchObject({
      kind: "number",
      normalized: "13.7",
    }));
  it("normalizes English thousands", () =>
    expect(parseMeasurementInput("1,234.5", "en-US")).toMatchObject({
      kind: "number",
      normalized: "1234.5",
    }));
  it("requests confirmation for ambiguous dates", () =>
    expect(parseMeasurementInput("01/02/2026", "en-US")).toMatchObject({
      kind: "ambiguous",
    }));
  it("normalizes locale dates", () =>
    expect(parseMeasurementInput("31.12.2026", "de-DE")).toMatchObject({
      kind: "date",
      normalized: "2026-12-31",
    }));
  it("keeps ordinal and text values exact", () =>
    expect(parseMeasurementInput("positive", "en-US")).toMatchObject({
      kind: "text",
      value: "positive",
    }));
  it("covers representative blood-count and metabolic values", () => {
    expect(parseMeasurementInput("4.8", "en-US")).toMatchObject({
      kind: "number",
      value: 4.8,
    });
    expect(parseMeasurementInput("5,6", "de-DE")).toMatchObject({
      kind: "number",
      value: 5.6,
    });
  });
  it("keeps custom analyte text values unchanged", () =>
    expect(parseMeasurementInput("trace", "en-US")).toMatchObject({
      kind: "text",
      normalized: "trace",
    }));
});
