export type PersonalTargetRangeDraft = {
  analyteId: string;
  lowerBound: string;
  upperBound: string;
  unit: string;
  validFrom: string;
  validTo: string;
};

const parseDecimal = (value: string) =>
  value ? Number(value.replace(",", ".")) : undefined;

export function validatePersonalTargetRange(
  draft: PersonalTargetRangeDraft,
): string[] {
  const errors: string[] = [];
  const lower = parseDecimal(draft.lowerBound);
  const upper = parseDecimal(draft.upperBound);
  if (!draft.analyteId) errors.push("Select an analyte.");
  if (!draft.lowerBound && !draft.upperBound)
    errors.push("Enter a lower limit or an upper limit.");
  if (!draft.unit.trim()) errors.push("Enter the unit for this range.");
  if (lower !== undefined && !Number.isFinite(lower))
    errors.push("Enter a numeric lower limit.");
  if (upper !== undefined && !Number.isFinite(upper))
    errors.push("Enter a numeric upper limit.");
  if (
    lower !== undefined &&
    upper !== undefined &&
    Number.isFinite(lower) &&
    Number.isFinite(upper) &&
    lower > upper
  )
    errors.push("The lower limit must not exceed the upper limit.");
  if (draft.validFrom && draft.validTo && draft.validFrom > draft.validTo)
    errors.push("The valid-from date must not follow the valid-to date.");
  return errors;
}
