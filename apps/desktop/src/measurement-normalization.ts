import { UcumLhcUtils } from "@lhncbc/ucum-lhc";

import type { AnalyteDefinition, ReportSummary } from "./vault-client";

const RULE_ID = "ucum-lhc@7.1.9:automatic";
const ucum = UcumLhcUtils.getInstance();

export type NormalizationResult =
  | {
      status: "normalized";
      value: number;
      unit: string;
      ruleId: typeof RULE_ID;
    }
  | {
      status: "blocked";
      reason:
        | "missing-analyte"
        | "non-numeric-value"
        | "missing-canonical-unit"
        | "non-quantitative-analyte"
        | "arbitrary-unit"
        | "invalid-unit"
        | "incompatible-unit";
    };

type Measurement = ReportSummary["measurements"][number];

export function normalizeMeasurement(
  measurement: Measurement,
  analyte: AnalyteDefinition | undefined,
): NormalizationResult {
  if (!analyte || measurement.analyteId !== analyte.id)
    return { status: "blocked", reason: "missing-analyte" };
  if (!measurement.parsedNumericValue)
    return { status: "blocked", reason: "non-numeric-value" };
  if (!analyte.canonicalUnit)
    return { status: "blocked", reason: "missing-canonical-unit" };
  if (analyte.scale.toLowerCase() !== "quantitative")
    return { status: "blocked", reason: "non-quantitative-analyte" };
  if (analyte.property.toLowerCase().includes("arb"))
    return { status: "blocked", reason: "arbitrary-unit" };

  const source = ucum.validateUnitString(measurement.sourceUnit);
  const target = ucum.validateUnitString(analyte.canonicalUnit);
  if (source.status !== "valid" || target.status !== "valid")
    return { status: "blocked", reason: "invalid-unit" };
  const value = Number(measurement.parsedNumericValue);
  if (!Number.isFinite(value))
    return { status: "blocked", reason: "non-numeric-value" };
  const conversion = ucum.convertUnitTo(
    source.ucumCode ?? measurement.sourceUnit,
    value,
    target.ucumCode ?? analyte.canonicalUnit,
  );
  if (conversion.status !== "succeeded" || !Number.isFinite(conversion.toVal))
    return { status: "blocked", reason: "incompatible-unit" };
  return {
    status: "normalized",
    value: conversion.toVal as number,
    unit: target.ucumCode ?? analyte.canonicalUnit,
    ruleId: RULE_ID,
  };
}
