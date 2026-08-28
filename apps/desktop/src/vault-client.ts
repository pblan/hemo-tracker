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
