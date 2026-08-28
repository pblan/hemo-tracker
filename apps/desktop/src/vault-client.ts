import { invoke } from "@tauri-apps/api/core";

export type VaultState = {
  accountExists: boolean;
  status: "missing" | "locked" | "unlocked";
};

export type CreatedVault = { recoveryCode: string };

export const getVaultState = () => invoke<VaultState>("get_vault_state");
export const createLocalAccount = (passphrase: string) =>
  invoke<CreatedVault>("create_local_account", { passphrase });
export const unlockWithPassphrase = (passphrase: string) =>
  invoke<VaultState>("unlock_with_passphrase", { passphrase });
export const unlockWithRecovery = (recoveryCode: string) =>
  invoke<VaultState>("unlock_with_recovery", { recoveryCode });
export const lockVault = () => invoke<VaultState>("lock_vault");
export const createLabReport = (request: {
  collectionTime: string;
  reportDate?: string;
  laboratory?: string;
  fastingState?: string;
  notes?: string;
  tags: string[];
}) => invoke<string>("create_lab_report", { request });
export const selectAndAttachSourceFile = (reportId: string, role = "primary") =>
  invoke<{ id: string; originalFilename: string } | null>(
    "select_and_attach_source_file",
    { reportId, role },
  );
export const addLabMeasurement = (
  reportId: string,
  request: {
    sourceLabel: string;
    sourceValue: string;
    sourceUnit: string;
    sourceReferenceInterval: string;
    sourceFlag: string;
    parsedNumericValue?: string;
    analyteId?: string;
  },
) => invoke<string>("add_lab_measurement", { reportId, request });
export const completeLabReport = (reportId: string) =>
  invoke<void>("complete_lab_report", { reportId });
export const archiveLabReport = (reportId: string) =>
  invoke<void>("archive_lab_report", { reportId });
export const correctLabMeasurement = (
  measurementId: string,
  request: Parameters<typeof addLabMeasurement>[1],
  updatedBy = "local-user",
) =>
  invoke<void>("correct_lab_measurement", {
    measurementId,
    request,
    updatedBy,
  });
export const addAnalyteDefinition = (request: {
  name: string;
  component: string;
  property: string;
  specimen: string;
  scale: string;
  method?: string;
  aliases: string[];
  loincCode?: string;
  canonicalUnit?: string;
  personalTargetRanges: PersonalTargetRangeInput[];
}) => invoke<string>("add_analyte_definition", { request });
export type PersonalTargetRangeInput = {
  lowerBound?: string;
  upperBound?: string;
  unit: string;
  validFrom?: string;
  validTo?: string;
  context?: string;
  notes?: string;
};
export type PersonalTargetRange = PersonalTargetRangeInput & { id: string };
export type AnalyteDefinition = {
  id: string;
  name: string;
  component: string;
  property: string;
  specimen: string;
  scale: string;
  method?: string;
  aliases: string[];
  loincCode?: string;
  canonicalUnit?: string;
  personalTargetRanges: PersonalTargetRange[];
};
export const listAnalyteDefinitions = () =>
  invoke<AnalyteDefinition[]>("list_analyte_definitions");
export const addPersonalTargetRange = (
  analyteId: string,
  request: PersonalTargetRangeInput,
) => invoke<string>("add_personal_target_range", { analyteId, request });
export type ReportSummary = {
  id: string;
  collectionTime: string;
  laboratory?: string;
  notes?: string;
  tags: string[];
  status: "draft" | "complete" | "archived";
  sourceFileCount: number;
  measurementCount: number;
  sourceFiles: {
    id: string;
    filename: string;
    mediaType: string;
    role: string;
  }[];
  measurements: {
    id: string;
    sourceLabel: string;
    sourceValue: string;
    sourceUnit: string;
    sourceReferenceInterval: string;
    sourceFlag: string;
    parsedNumericValue?: string;
    analyteId?: string;
    updatedAt: string;
    updatedBy: string;
  }[];
};
export const getLabReport = (reportId: string) =>
  invoke<ReportSummary>("get_lab_report", { reportId });
export const readSourceFile = (reportId: string, sourceFileId: string) =>
  invoke<{ filename: string; mediaType: string; bytes: number[] }>(
    "read_source_file",
    { reportId, sourceFileId },
  );
export const listLabReports = () => invoke<string[]>("list_lab_reports");
export const permanentlyDeleteLabReport = (
  reportId: string,
  confirmed: boolean,
) => invoke<void>("permanently_delete_lab_report", { reportId, confirmed });
export const backupLocalVault = (destination: string) =>
  invoke<void>("backup_local_vault", { destination });
export const chooseAndBackupLocalVault = () =>
  invoke<boolean>("choose_and_backup_local_vault");
export const restoreLocalVault = (backup: string, passphrase: string) =>
  invoke<void>("restore_local_vault", { backup, passphrase });
export const chooseAndRestoreLocalVault = (passphrase: string) =>
  invoke<boolean>("choose_and_restore_local_vault", { passphrase });
export const chooseAndExportPlaintextZip = () =>
  invoke<boolean>("choose_and_export_plaintext_zip");
