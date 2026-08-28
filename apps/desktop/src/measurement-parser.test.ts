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
});
