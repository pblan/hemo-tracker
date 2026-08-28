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
export const selectAndAttachSourceFile = (reportId: string) =>
  invoke<{ id: string; originalFilename: string } | null>(
    "select_and_attach_source_file",
    { reportId },
  );
export const addLabMeasurement = (
  reportId: string,
  request: {
    sourceLabel: string;
    sourceValue: string;
    sourceUnit: string;
    sourceReferenceInterval: string;
    sourceFlag: string;
    analyteId?: string;
  },
) => invoke<string>("add_lab_measurement", { reportId, request });
export const completeLabReport = (reportId: string) =>
  invoke<void>("complete_lab_report", { reportId });
export const addAnalyteDefinition = (request: {
  name: string;
  component: string;
  property: string;
  specimen: string;
  scale: string;
  method?: string;
  aliases: string[];
  loincCode?: string;
}) => invoke<string>("add_analyte_definition", { request });
