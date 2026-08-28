export type MeasurementDraft = {
  sourceLabel: string;
  sourceValue: string;
  sourceUnit: string;
  sourceReferenceInterval: string;
  sourceFlag: string;
};

export function validateMeasurementRow(row: MeasurementDraft): string[] {
  const errors: string[] = [];
  if (!row.sourceLabel.trim()) errors.push("Enter the source label.");
  if (!row.sourceValue.trim()) errors.push("Enter the source value.");
  if (!row.sourceUnit.trim()) errors.push("Enter the source unit.");
  if (!row.sourceReferenceInterval.trim())
    errors.push("Enter the reference interval.");
  if (!row.sourceFlag.trim()) errors.push("Enter the source flag.");
  return errors;
}
