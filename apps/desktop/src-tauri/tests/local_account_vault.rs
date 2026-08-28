use hemo_tracker_desktop_lib::account_vault::{
    CreateLabReportDraft, CreateLocalAccount, LocalAccountVault, NewAnalyte, NewMeasurement,
    NewSourceFile, ReportStatus, VaultStatus,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn user_can_create_lock_and_reopen_a_local_account_vault() {
    let directory = tempdir().unwrap();
    let account = directory.path().join("account");
    let created = LocalAccountVault::create(
        &account,
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
    let mut reopened = LocalAccountVault::open(&account).unwrap();
    assert_eq!(reopened.status(), VaultStatus::Locked);
    reopened
        .unlock_with_passphrase("correct horse battery staple".to_owned())
        .unwrap();
    assert_eq!(reopened.status(), VaultStatus::Unlocked);
}

#[test]
fn recovery_unlocks_the_vault_and_wrong_credentials_do_not_damage_it() {
    let directory = tempdir().unwrap();
    let account = directory.path().join("account");
    let created = LocalAccountVault::create(
        &account,
        CreateLocalAccount {
            account_id: "recovery-test".to_owned(),
            passphrase: "valid passphrase".to_owned(),
        },
    )
    .unwrap();
    let recovery_code = created.recovery_code().to_owned();
    drop(created);

    let mut vault = LocalAccountVault::open(&account).unwrap();
    assert!(
        vault
            .unlock_with_passphrase("wrong passphrase".to_owned())
            .is_err()
    );
    assert_eq!(vault.status(), VaultStatus::Locked);

    assert!(
        vault
            .unlock_with_recovery("HTRK1-invalid".to_owned())
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

#[test]
fn user_records_and_reopens_one_complete_lab_report_with_encrypted_evidence() {
    let parent = tempdir().unwrap();
    let account = parent.path().join("account");
    let created = LocalAccountVault::create(
        &account,
        CreateLocalAccount {
            account_id: "first-report".to_owned(),
            passphrase: "valid passphrase".to_owned(),
        },
    )
    .unwrap();
    let mut vault = created.into_vault();

    let hemoglobin_id = vault
        .add_analyte(NewAnalyte {
            name: "Hemoglobin".to_owned(),
            component: "Hemoglobin".to_owned(),
            property: "MCnc".to_owned(),
            specimen: "Blood".to_owned(),
            scale: "Quantitative".to_owned(),
            method: None,
            aliases: vec!["Hb".to_owned()],
            loinc_code: Some("718-7".to_owned()),
        })
        .unwrap();
    assert!(vault.list_analytes().unwrap().len() >= 5);

    let report_id = vault
        .create_lab_report_draft(CreateLabReportDraft {
            collection_time: "2026-08-20T08:30:00+02:00".to_owned(),
            report_date: Some("2026-08-21".to_owned()),
            laboratory: Some("Fictional Central Laboratory".to_owned()),
            ordering_clinician: None,
            fasting_state: Some("fasting".to_owned()),
            notes: Some("Routine fictional check".to_owned()),
            tags: vec!["annual".to_owned()],
        })
        .unwrap();
    let source_marker = b"fictional-pdf-source-marker";
    let source_path = parent.path().join("original-report.pdf");
    fs::write(&source_path, source_marker).unwrap();
    vault
        .add_source_file(
            &report_id,
            fs::File::open(&source_path).unwrap(),
            NewSourceFile {
                original_filename: "fictional-report.pdf".to_owned(),
                media_type: "application/pdf".to_owned(),
                role: "primary".to_owned(),
            },
        )
        .unwrap();
    vault
        .add_measurement(
            &report_id,
            NewMeasurement {
                source_label: "Hemoglobin (original label)".to_owned(),
                source_value: "13,7".to_owned(),
                source_unit: "g/dL".to_owned(),
                source_reference_interval: "12,0–16,0".to_owned(),
                source_flag: "within range".to_owned(),
                analyte_id: Some(hemoglobin_id.clone()),
            },
        )
        .unwrap();
    vault.complete_lab_report(&report_id).unwrap();

    let report = vault.get_lab_report(&report_id).unwrap();
    assert_eq!(report.status, ReportStatus::Complete);
    assert_eq!(report.collection_time, "2026-08-20T08:30:00+02:00");
    assert_eq!(report.report_date.as_deref(), Some("2026-08-21"));
    assert_eq!(
        report.laboratory.as_deref(),
        Some("Fictional Central Laboratory")
    );
    assert_eq!(report.fasting_state.as_deref(), Some("fasting"));
    assert_eq!(report.notes.as_deref(), Some("Routine fictional check"));
    assert_eq!(report.tags, vec!["annual"]);
    assert_eq!(report.measurements[0].source_value, "13,7");
    assert_eq!(report.measurements[0].source_unit, "g/dL");
    assert_eq!(
        report.measurements[0].source_reference_interval,
        "12,0–16,0"
    );
    assert_eq!(
        report.source_files[0].original_filename,
        "fictional-report.pdf"
    );

    vault.lock();
    drop(vault);
    let mut reopened = LocalAccountVault::open(&account).unwrap();
    reopened
        .unlock_with_passphrase("valid passphrase".to_owned())
        .unwrap();
    let report = reopened.get_lab_report(&report_id).unwrap();
    assert_eq!(report.status, ReportStatus::Complete);
    assert_eq!(
        report.measurements[0].source_label,
        "Hemoglobin (original label)"
    );
    assert_eq!(report.measurements[0].source_unit, "g/dL");
    assert_eq!(report.measurements[0].source_value, "13,7");
    assert_eq!(
        report.measurements[0].source_reference_interval,
        "12,0–16,0"
    );
    assert_eq!(report.measurements[0].source_flag, "within range");
    assert_eq!(
        report.measurements[0].analyte_id.as_deref(),
        Some(hemoglobin_id.as_str())
    );
    assert_eq!(
        report.source_files[0].original_filename,
        "fictional-report.pdf"
    );

    assert_eq!(fs::read(&source_path).unwrap(), source_marker);

    assert_directory_does_not_contain(&account, source_marker);
    assert_directory_does_not_contain(&account, b"13,7");
}

fn assert_directory_does_not_contain(directory: &std::path::Path, marker: &[u8]) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            assert_directory_does_not_contain(&path, marker);
        } else {
            let bytes = fs::read(path).unwrap();
            assert!(!bytes.windows(marker.len()).any(|part| part == marker));
        }
    }
}
