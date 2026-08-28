use hemo_tracker_desktop_lib::account_vault::{CreateLocalAccount, LocalAccountVault, VaultStatus};
use tempfile::tempdir;

#[test]
fn user_can_create_lock_and_reopen_a_local_account_vault() {
    let directory = tempdir().unwrap();
    let created = LocalAccountVault::create(
        directory.path(),
        CreateLocalAccount {
            account_id: "personal".to_owned(),
            passphrase: "correct horse battery staple".to_owned(),
        },
    )
    .unwrap();
    let recovery_code = created.recovery_code().to_owned();
    let mut vault = created.into_vault();

    assert_eq!(vault.status(), VaultStatus::Unlocked);
    assert!(recovery_code.starts_with("HTRK1-"));

    vault.lock();
    assert_eq!(vault.status(), VaultStatus::Locked);
    vault
        .unlock_with_passphrase("correct horse battery staple".to_owned())
        .unwrap();
    assert_eq!(vault.status(), VaultStatus::Unlocked);

    drop(vault);
    let mut reopened = LocalAccountVault::open(directory.path()).unwrap();
    assert_eq!(reopened.status(), VaultStatus::Locked);
    reopened
        .unlock_with_passphrase("correct horse battery staple".to_owned())
        .unwrap();
    assert_eq!(reopened.status(), VaultStatus::Unlocked);
}

#[test]
fn recovery_unlocks_the_vault_and_wrong_credentials_do_not_damage_it() {
    let directory = tempdir().unwrap();
    let created = LocalAccountVault::create(
        directory.path(),
        CreateLocalAccount {
            account_id: "recovery-test".to_owned(),
            passphrase: "valid passphrase".to_owned(),
        },
    )
    .unwrap();
    let recovery_code = created.recovery_code().to_owned();
    drop(created);

    let mut vault = LocalAccountVault::open(directory.path()).unwrap();
    assert!(
        vault
            .unlock_with_passphrase("wrong passphrase".to_owned())
            .is_err()
    );
    assert_eq!(vault.status(), VaultStatus::Locked);

    vault.unlock_with_recovery(recovery_code).unwrap();
    assert_eq!(vault.status(), VaultStatus::Unlocked);

    vault.lock();
    vault
        .unlock_with_passphrase("valid passphrase".to_owned())
        .unwrap();
    assert_eq!(vault.status(), VaultStatus::Unlocked);
}
