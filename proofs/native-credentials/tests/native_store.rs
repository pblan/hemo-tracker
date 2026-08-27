use hemo_native_credentials_proof::{
    DeviceUnlockKey, NativeCredentialStore, SavedAccessApproval, TrustedDeviceCredentials,
};

#[test]
#[ignore = "changes the signed user's native credential store"]
fn native_store_round_trip_and_revocation() {
    let account = format!("proof-{}", std::process::id());
    let credentials = TrustedDeviceCredentials::new(NativeCredentialStore);
    let key = DeviceUnlockKey::new([0x5a; 32]);
    credentials
        .save_after_approval(&account, SavedAccessApproval::Approved, &key)
        .unwrap();
    let checksum = credentials.use_key(&account, |_| 0x5a_u32 * 32).unwrap();
    assert_eq!(checksum, 2_880);
    credentials.revoke_device(&account).unwrap();
}
