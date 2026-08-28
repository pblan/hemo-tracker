use hemo_encrypted_vault_proof::{
    LabReportDraft, NativeVaultManager, VaultCommand, VaultCommandFacade, VaultCommandResult,
    VaultError, VaultKey, VaultProbe, CURRENT_SCHEMA_VERSION,
};
use std::fs;
use tempfile::tempdir;

const CLINICAL_MARKER: &str = "Ferritin result 42.7 ug/L";

#[test]
fn encrypted_vault_migrates_and_round_trips_typed_reports() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("account.vault");
    let key = VaultKey::generate().unwrap();
    let vault = NativeVaultManager::open(&path, &key).unwrap();

    vault
        .add_lab_report(&LabReportDraft {
            title: CLINICAL_MARKER.to_owned(),
            observed_at: "2026-08-27T08:00:00Z".to_owned(),
        })
        .unwrap();

    assert_eq!(vault.schema_version().unwrap(), CURRENT_SCHEMA_VERSION);
    assert_eq!(vault.list_lab_reports().unwrap()[0].title, CLINICAL_MARKER);
    assert!(
        !fs::read(&path)
            .unwrap()
            .windows(CLINICAL_MARKER.len())
            .any(|part| part == CLINICAL_MARKER.as_bytes())
    );
    assert_ne!(&fs::read(&path).unwrap()[..16], b"SQLite format 3\0");
}

#[test]
fn wrong_keys_and_corruption_fail_closed() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("account.vault");
    let key = VaultKey::generate().unwrap();
    let vault = NativeVaultManager::open(&path, &key).unwrap();
    vault
        .add_lab_report(&LabReportDraft {
            title: CLINICAL_MARKER.to_owned(),
            observed_at: "2026-08-27".to_owned(),
        })
        .unwrap();
    vault.checkpoint().unwrap();
    drop(vault);

    assert!(matches!(
        NativeVaultManager::open(&path, &VaultKey::generate().unwrap()),
        Err(VaultError::InvalidKeyOrVault)
    ));

    let mut bytes = fs::read(&path).unwrap();
    let index = bytes.len() / 2;
    bytes[index] ^= 0x80;
    fs::write(&path, bytes).unwrap();
    let reopened = NativeVaultManager::open(&path, &key);
    assert!(reopened.is_err() || reopened.unwrap().integrity_check().is_err());
}

#[test]
fn backup_and_restore_remain_encrypted() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("account.vault");
    let backup = directory.path().join("backup.vault");
    let restored = directory.path().join("restored.vault");
    let key = VaultKey::generate().unwrap();
    let vault = NativeVaultManager::open(&path, &key).unwrap();
    vault
        .add_lab_report(&LabReportDraft {
            title: CLINICAL_MARKER.to_owned(),
            observed_at: "2026-08-27".to_owned(),
        })
        .unwrap();
    NativeVaultManager::back_up(&vault, &backup).unwrap();

    assert_ne!(&fs::read(&backup).unwrap()[..16], b"SQLite format 3\0");
    NativeVaultManager::restore(&backup, &restored, &key).unwrap();
    let restored_vault = NativeVaultManager::open(&restored, &key).unwrap();
    assert_eq!(
        restored_vault.list_lab_reports().unwrap()[0].title,
        CLINICAL_MARKER
    );
}

#[test]
fn wal_journal_and_temporary_storage_do_not_contain_source_text() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("account.vault");
    let key = VaultKey::generate().unwrap();
    let vault = NativeVaultManager::open(&path, &key).unwrap();
    vault
        .add_lab_report(&LabReportDraft {
            title: CLINICAL_MARKER.to_owned(),
            observed_at: "2026-08-27".to_owned(),
        })
        .unwrap();

    assert_eq!(vault.temporary_store().unwrap(), "memory");
    for suffix in ["-wal", "-shm"] {
        let sidecar = path.with_file_name(format!("account.vault{suffix}"));
        if sidecar.exists() {
            assert!(
                !fs::read(sidecar)
                    .unwrap()
                    .windows(CLINICAL_MARKER.len())
                    .any(|part| part == CLINICAL_MARKER.as_bytes())
            );
        }
    }

    VaultProbe::begin_rollback_journal(&vault, CLINICAL_MARKER).unwrap();
    let journal = path.with_file_name("account.vault-journal");
    assert!(journal.exists());
    assert!(
        !fs::read(journal)
            .unwrap()
            .windows(CLINICAL_MARKER.len())
            .any(|part| part == CLINICAL_MARKER.as_bytes())
    );
    VaultProbe::finish_rollback_journal(&vault).unwrap();
}

#[test]
fn a_stock_sqlite_tool_cannot_read_the_vault() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("account.vault");
    let key = VaultKey::generate().unwrap();
    NativeVaultManager::open(&path, &key).unwrap();

    let output = std::process::Command::new("sqlite3")
        .arg(&path)
        .arg("SELECT name FROM sqlite_schema")
        .output()
        .expect("the proof runner must provide a stock sqlite3 command");
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("lab_reports"));
}

#[test]
fn a_crash_during_a_transaction_does_not_commit_partial_data() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("account.vault");
    let key = VaultKey::generate().unwrap();
    let vault = NativeVaultManager::open(&path, &key).unwrap();
    let status = VaultProbe::run_crash_write(
        &vault,
        env!("CARGO_BIN_EXE_hemo-encrypted-vault-proof"),
        CLINICAL_MARKER,
    )
    .unwrap();
    assert_eq!(status.code(), Some(17));
    drop(vault);

    let vault = NativeVaultManager::open(&path, &key).unwrap();
    assert!(vault.list_lab_reports().unwrap().is_empty());
    vault.integrity_check().unwrap();
}

#[test]
fn the_command_facade_accepts_only_typed_domain_requests() {
    let directory = tempdir().unwrap();
    let key = VaultKey::generate().unwrap();
    let vault = NativeVaultManager::open(directory.path().join("account.vault"), &key).unwrap();
    let facade = VaultCommandFacade::new(vault);

    let added = facade
        .execute(VaultCommand::AddLabReport(LabReportDraft {
            title: CLINICAL_MARKER.to_owned(),
            observed_at: "2026-08-27".to_owned(),
        }))
        .unwrap();
    assert!(matches!(
        added,
        VaultCommandResult::LabReportAdded { id: 1 }
    ));

    let listed = facade.execute(VaultCommand::ListLabReports).unwrap();
    let VaultCommandResult::LabReports(reports) = listed else {
        panic!("unexpected command result")
    };
    assert_eq!(reports[0].title, CLINICAL_MARKER);
}
