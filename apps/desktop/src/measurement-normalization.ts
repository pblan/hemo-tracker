import { UcumLhcUtils } from "@lhncbc/ucum-lhc";

import type {
  AnalyteDefinition,
  PersonalTargetRange,
  ReportSummary,
} from "./vault-client";

const RULE_ID = "ucum-lhc@7.1.9:automatic";
const ucum = UcumLhcUtils.getInstance();

const curatedRules = [
  {
    sourceLoinc: "2345-7",
    targetLoinc: "14749-6",
    component: "glucose",
    sourceProperty: "mcnc",
    targetProperty: "scnc",
    specimen: "serum or plasma",
    sourceUnit: "mg/dL",
    targetUnit: "mmol/L",
    molecularWeight: 180.1559,
    provenance: "nist-srd-69:50-99-7",
  },
  {
    sourceLoinc: "2160-0",
    targetLoinc: "14682-9",
    component: "creatinine",
    sourceProperty: "mcnc",
    targetProperty: "scnc",
    specimen: "serum or plasma",
    sourceUnit: "mg/dL",
    targetUnit: "umol/L",
    molecularWeight: 113.1179,
    provenance: "nist-srd-69:60-27-5",
  },
] as const;

export type NormalizationResult =
  | {
      status: "normalized";
      value: number;
      unit: string;
      ruleId: string;
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
  if (conversion.status !== "succeeded" || !Number.isFinite(conversion.toVal)) {
    const curated = curatedRules.find(
      (rule) =>
        analyte.loincCode === rule.sourceLoinc &&
        analyte.component.toLowerCase() === rule.component &&
        analyte.property.toLowerCase() === rule.sourceProperty &&
        analyte.specimen.toLowerCase() === rule.specimen &&
        (source.ucumCode ?? measurement.sourceUnit) === rule.sourceUnit &&
        analyte.canonicalUnit === rule.targetUnit,
    );
    if (!curated) return { status: "blocked", reason: "incompatible-unit" };
    const reviewedConversion = ucum.convertUnitTo(
      source.ucumCode ?? measurement.sourceUnit,
      value,
      target.ucumCode ?? analyte.canonicalUnit,
      { molecularWeight: curated.molecularWeight },
    );
    if (
      reviewedConversion.status !== "succeeded" ||
      !Number.isFinite(reviewedConversion.toVal)
    )
      return { status: "blocked", reason: "incompatible-unit" };
    return {
      status: "normalized",
      value: reviewedConversion.toVal as number,
      unit: target.ucumCode ?? analyte.canonicalUnit,
      ruleId: `curated:${curated.sourceLoinc}->${curated.targetLoinc}:${curated.provenance}`,
    };
  }
  return {
    status: "normalized",
    value: conversion.toVal as number,
    unit: target.ucumCode ?? analyte.canonicalUnit,
    ruleId: RULE_ID,
  };
}

export type ApplicableTargetRange =
  | {
      status: "applicable";
      lowerBound?: number;
      upperBound?: number;
      unit: string;
      rangeId: string;
    }
  | {
      status: "unavailable";
      reason: "none" | "ambiguous" | "context-unverified" | "incompatible-unit";
    };

export function resolveApplicableTargetRange(
  date: string,
  ranges: PersonalTargetRange[],
  analyte: AnalyteDefinition,
): ApplicableTargetRange {
  const calendarDate = date.slice(0, 10);
  const dated = ranges.filter(
    (range) =>
      (!range.validFrom || calendarDate >= range.validFrom) &&
      (!range.validTo || calendarDate <= range.validTo),
  );
  if (dated.some((range) => range.context?.trim()))
    return { status: "unavailable", reason: "context-unverified" };
  if (dated.length === 0) return { status: "unavailable", reason: "none" };
  if (dated.length > 1) return { status: "unavailable", reason: "ambiguous" };

  const range = dated[0];
  if (!range) return { status: "unavailable", reason: "none" };
  const normalizeBound = (sourceValue: string | undefined) => {
    if (!sourceValue) return undefined;
    const result = normalizeMeasurement(
      {
        id: `target-range:${range.id}`,
        sourceLabel: analyte.name,
        sourceValue,
        sourceUnit: range.unit,
        sourceReferenceInterval: "",
        sourceFlag: "",
        parsedNumericValue: sourceValue.replace(",", "."),
        analyteId: analyte.id,
        updatedAt: "",
        updatedBy: "local-user",
      },
      analyte,
    );
    return result.status === "normalized" ? result.value : null;
  };
  const lowerBound = normalizeBound(range.lowerBound);
  const upperBound = normalizeBound(range.upperBound);
  if (lowerBound === null || upperBound === null)
    return { status: "unavailable", reason: "incompatible-unit" };
  return {
    status: "applicable",
    lowerBound,
    upperBound,
    unit: analyte.canonicalUnit ?? range.unit,
    rangeId: range.id,
  };
}
