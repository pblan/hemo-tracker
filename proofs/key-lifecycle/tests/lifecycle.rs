use hemo_key_lifecycle_proof::{
    AccountKeyBundle, KeyEnvelope, KeyLifecycleError, Passphrase, Purpose, RecoveryKey,
};

const ACCOUNT_ID: &str = "account-018f5f2e";

#[test]
fn passphrase_unlock_restores_the_account_keys() {
    let passphrase = Passphrase::new("correct horse battery staple");
    let bundle = AccountKeyBundle::create(ACCOUNT_ID, &passphrase).unwrap();

    let unlocked = bundle
        .passphrase_envelope()
        .unlock_with_passphrase(&passphrase)
        .unwrap();

    assert!(
        bundle
            .unlocked_keys()
            .derive_purpose_key(Purpose::Database, 1)
            .matches(&unlocked.derive_purpose_key(Purpose::Database, 1)),
    );
}

#[test]
fn recovery_key_restores_the_account_keys() {
    let bundle =
        AccountKeyBundle::create(ACCOUNT_ID, &Passphrase::new("first passphrase")).unwrap();

    let recovery_code = bundle.recovery_key().to_code();
    let restored_recovery_key = RecoveryKey::from_code(recovery_code.expose()).unwrap();
    let recovered = bundle
        .recovery_envelope()
        .unlock_with_recovery(&restored_recovery_key)
        .unwrap();

    assert!(
        bundle
            .unlocked_keys()
            .derive_purpose_key(Purpose::SourceFiles, 1)
            .matches(&recovered.derive_purpose_key(Purpose::SourceFiles, 1)),
    );
}

#[test]
fn a_changed_recovery_code_is_rejected_before_unlock() {
    let bundle = AccountKeyBundle::create(ACCOUNT_ID, &Passphrase::new("passphrase")).unwrap();
    let code = bundle.recovery_key().to_code();
    let mut changed = code.expose().to_owned();
    let replacement = if changed.ends_with('A') { "B" } else { "A" };
    changed.replace_range(changed.len() - 1.., replacement);

    assert_eq!(
        RecoveryKey::from_code(&changed).unwrap_err(),
        KeyLifecycleError::InvalidCredentialsOrEnvelope,
    );
}

#[test]
fn passphrase_change_does_not_change_data_keys() {
    let old_passphrase = Passphrase::new("old passphrase");
    let new_passphrase = Passphrase::new("new passphrase");
    let bundle = AccountKeyBundle::create(ACCOUNT_ID, &old_passphrase).unwrap();

    let changed = bundle
        .passphrase_envelope()
        .change_passphrase(&old_passphrase, &new_passphrase)
        .unwrap();
    let unlocked = changed.unlock_with_passphrase(&new_passphrase).unwrap();

    assert!(
        bundle
            .unlocked_keys()
            .derive_purpose_key(Purpose::SyncManifest, 1)
            .matches(&unlocked.derive_purpose_key(Purpose::SyncManifest, 1)),
    );
    assert!(changed.unlock_with_passphrase(&old_passphrase).is_err());
}

#[test]
fn wrong_passphrases_and_changed_envelopes_return_one_safe_error() {
    let passphrase = Passphrase::new("correct passphrase");
    let bundle = AccountKeyBundle::create(ACCOUNT_ID, &passphrase).unwrap();

    let wrong = bundle
        .passphrase_envelope()
        .unlock_with_passphrase(&Passphrase::new("wrong passphrase"))
        .unwrap_err();

    let changed_json = bundle
        .passphrase_envelope()
        .to_json()
        .replace("\"ciphertext\":\"", "\"ciphertext\":\"A");
    let changed = KeyEnvelope::from_json(&changed_json)
        .unwrap()
        .unlock_with_passphrase(&passphrase)
        .unwrap_err();

    assert_eq!(wrong, KeyLifecycleError::InvalidCredentialsOrEnvelope);
    assert_eq!(changed, KeyLifecycleError::InvalidCredentialsOrEnvelope);
    assert_eq!(wrong.to_string(), "credentials or key envelope are invalid");
}

#[test]
fn purpose_and_generation_create_separate_keys() {
    let bundle = AccountKeyBundle::create(ACCOUNT_ID, &Passphrase::new("passphrase")).unwrap();
    let database_v1 = bundle
        .unlocked_keys()
        .derive_purpose_key(Purpose::Database, 1);
    let database_v2 = bundle
        .unlocked_keys()
        .derive_purpose_key(Purpose::Database, 2);
    let source_files_v1 = bundle
        .unlocked_keys()
        .derive_purpose_key(Purpose::SourceFiles, 1);

    assert!(!database_v1.matches(&database_v2));
    assert!(!database_v1.matches(&source_files_v1));
}

#[test]
fn envelopes_round_trip_through_the_versioned_json_format() {
    let passphrase = Passphrase::new("correct passphrase");
    let bundle = AccountKeyBundle::create(ACCOUNT_ID, &passphrase).unwrap();
    let json = bundle.passphrase_envelope().to_json();

    let parsed = KeyEnvelope::from_json(&json).unwrap();
    let unlocked = parsed.unlock_with_passphrase(&passphrase).unwrap();

    assert_eq!(parsed.version(), 1);
    assert_eq!(parsed.account_id(), ACCOUNT_ID);
    assert!(
        bundle
            .unlocked_keys()
            .derive_purpose_key(Purpose::Database, 1)
            .matches(&unlocked.derive_purpose_key(Purpose::Database, 1)),
    );
}

#[test]
fn separate_account_creation_uses_a_new_random_data_key() {
    let first = AccountKeyBundle::create(ACCOUNT_ID, &Passphrase::new("passphrase")).unwrap();
    let second = AccountKeyBundle::create(ACCOUNT_ID, &Passphrase::new("passphrase")).unwrap();

    assert!(
        !first
            .unlocked_keys()
            .derive_purpose_key(Purpose::Database, 1)
            .matches(
                &second
                    .unlocked_keys()
                    .derive_purpose_key(Purpose::Database, 1),
            ),
    );
}
