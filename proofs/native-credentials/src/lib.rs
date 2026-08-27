use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
#[cfg(test)]
use std::{collections::HashMap, sync::Mutex};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

const SERVICE: &str = "dev.hemo-tracker.device-unlock.v1";

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("the user did not approve saved device access")]
    ApprovalRequired,
    #[error("the device credential is not available")]
    NotFound,
    #[error("the native credential store rejected the operation")]
    Store,
    #[error("the stored device credential has an invalid format")]
    InvalidFormat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SavedAccessApproval {
    Approved,
    Declined,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DeviceUnlockKey([u8; 32]);
impl DeviceUnlockKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

trait CredentialStore: Send + Sync {
    fn save(&self, account: &str, encoded: &str) -> Result<(), CredentialError>;
    fn load(&self, account: &str) -> Result<String, CredentialError>;
    fn delete(&self, account: &str) -> Result<(), CredentialError>;
}
struct NativeCredentialStore;

#[cfg(target_os = "macos")]
fn apple_entry(account: &str) -> Result<keyring_core::Entry, CredentialError> {
    use apple_native_keyring_store::protected::{AccessPolicy, Cred};
    Cred::build(
        SERVICE,
        account,
        AccessPolicy::WhenUnlockedThisDeviceOnly,
        None,
        false,
    )
    .map_err(map_apple_error)
}
#[cfg(target_os = "macos")]
fn map_apple_error(error: keyring_core::Error) -> CredentialError {
    match error {
        keyring_core::Error::NoEntry => CredentialError::NotFound,
        _ => CredentialError::Store,
    }
}
#[cfg(target_os = "macos")]
impl CredentialStore for NativeCredentialStore {
    fn save(&self, account: &str, encoded: &str) -> Result<(), CredentialError> {
        apple_entry(account)?
            .set_password(encoded)
            .map_err(map_apple_error)
    }
    fn load(&self, account: &str) -> Result<String, CredentialError> {
        apple_entry(account)?
            .get_password()
            .map_err(map_apple_error)
    }
    fn delete(&self, account: &str) -> Result<(), CredentialError> {
        apple_entry(account)?
            .delete_credential()
            .map_err(map_apple_error)
    }
}

#[cfg(target_os = "windows")]
impl CredentialStore for NativeCredentialStore {
    fn save(&self, account: &str, encoded: &str) -> Result<(), CredentialError> {
        use windows::Security::Credentials::{PasswordCredential, PasswordVault};
        use windows::core::HSTRING;
        let vault = PasswordVault::new().map_err(|_| CredentialError::Store)?;
        let item = PasswordCredential::CreatePasswordCredential(
            &HSTRING::from(SERVICE),
            &HSTRING::from(account),
            &HSTRING::from(encoded),
        )
        .map_err(|_| CredentialError::Store)?;
        vault.Add(&item).map_err(|_| CredentialError::Store)
    }
    fn load(&self, account: &str) -> Result<String, CredentialError> {
        use windows::Security::Credentials::PasswordVault;
        use windows::core::HSTRING;
        let vault = PasswordVault::new().map_err(|_| CredentialError::Store)?;
        let item = vault
            .Retrieve(&HSTRING::from(SERVICE), &HSTRING::from(account))
            .map_err(|_| CredentialError::NotFound)?;
        item.RetrievePassword()
            .map_err(|_| CredentialError::Store)?;
        item.Password()
            .map(|value| value.to_string())
            .map_err(|_| CredentialError::Store)
    }
    fn delete(&self, account: &str) -> Result<(), CredentialError> {
        use windows::Security::Credentials::PasswordVault;
        use windows::core::HSTRING;
        let vault = PasswordVault::new().map_err(|_| CredentialError::Store)?;
        let item = vault
            .Retrieve(&HSTRING::from(SERVICE), &HSTRING::from(account))
            .map_err(|_| CredentialError::NotFound)?;
        vault.Remove(&item).map_err(|_| CredentialError::Store)
    }
}

pub struct TrustedDeviceCredentials {
    store: Box<dyn CredentialStore>,
}
impl TrustedDeviceCredentials {
    pub fn native() -> Self {
        Self {
            store: Box::new(NativeCredentialStore),
        }
    }
    pub fn save_after_approval(
        &self,
        account: &str,
        approval: SavedAccessApproval,
        key: &DeviceUnlockKey,
    ) -> Result<(), CredentialError> {
        if approval != SavedAccessApproval::Approved {
            return Err(CredentialError::ApprovalRequired);
        }
        let encoded = Zeroizing::new(STANDARD_NO_PAD.encode(key.0));
        self.store.save(account, &encoded)
    }
    pub fn use_key<T>(
        &self,
        account: &str,
        operation: impl FnOnce(&DeviceUnlockKey) -> T,
    ) -> Result<T, CredentialError> {
        let encoded = Zeroizing::new(self.store.load(account)?);
        let decoded = Zeroizing::new(
            STANDARD_NO_PAD
                .decode(encoded.as_bytes())
                .map_err(|_| CredentialError::InvalidFormat)?,
        );
        let bytes: [u8; 32] = decoded
            .as_slice()
            .try_into()
            .map_err(|_| CredentialError::InvalidFormat)?;
        Ok(operation(&DeviceUnlockKey::new(bytes)))
    }
    pub fn logout(&self, account: &str) -> Result<(), CredentialError> {
        self.store.delete(account)
    }
    pub fn revoke_device(&self, account: &str) -> Result<(), CredentialError> {
        self.store.delete(account)
    }
}

#[cfg(test)]
#[derive(Default)]
struct MemoryCredentialStore(Mutex<HashMap<String, String>>);
#[cfg(test)]
impl CredentialStore for MemoryCredentialStore {
    fn save(&self, account: &str, encoded: &str) -> Result<(), CredentialError> {
        self.0
            .lock()
            .map_err(|_| CredentialError::Store)?
            .insert(account.into(), encoded.into());
        Ok(())
    }
    fn load(&self, account: &str) -> Result<String, CredentialError> {
        self.0
            .lock()
            .map_err(|_| CredentialError::Store)?
            .get(account)
            .cloned()
            .ok_or(CredentialError::NotFound)
    }
    fn delete(&self, account: &str) -> Result<(), CredentialError> {
        self.0
            .lock()
            .map_err(|_| CredentialError::Store)?
            .remove(account)
            .map(|_| ())
            .ok_or(CredentialError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn memory() -> TrustedDeviceCredentials {
        TrustedDeviceCredentials {
            store: Box::new(MemoryCredentialStore::default()),
        }
    }
    #[test]
    fn approval_is_required_before_save() {
        let credentials = memory();
        let key = DeviceUnlockKey::new([7; 32]);
        assert!(matches!(
            credentials.save_after_approval("account", SavedAccessApproval::Declined, &key),
            Err(CredentialError::ApprovalRequired)
        ));
        assert!(matches!(
            credentials.use_key("account", |_| ()),
            Err(CredentialError::NotFound)
        ));
    }
    #[test]
    fn raw_key_stays_inside_the_operation_callback() {
        let credentials = memory();
        let key = DeviceUnlockKey::new([9; 32]);
        credentials
            .save_after_approval("account", SavedAccessApproval::Approved, &key)
            .unwrap();
        assert_eq!(
            credentials
                .use_key("account", |stored| stored
                    .0
                    .iter()
                    .map(|byte| u32::from(*byte))
                    .sum::<u32>())
                .unwrap(),
            288
        );
    }
    #[test]
    fn logout_and_revocation_remove_saved_access() {
        for revoke in [false, true] {
            let credentials = memory();
            credentials
                .save_after_approval(
                    "account",
                    SavedAccessApproval::Approved,
                    &DeviceUnlockKey::new([3; 32]),
                )
                .unwrap();
            if revoke {
                credentials.revoke_device("account").unwrap();
            } else {
                credentials.logout("account").unwrap();
            }
            assert!(matches!(
                credentials.use_key("account", |_| ()),
                Err(CredentialError::NotFound)
            ));
        }
    }
    #[test]
    #[ignore = "changes and cleans up the signed user's native credential store"]
    fn native_store_round_trip_and_revocation() {
        let account = format!("proof-{}", std::process::id());
        let credentials = TrustedDeviceCredentials::native();
        struct Cleanup<'a>(&'a TrustedDeviceCredentials, &'a str);
        impl Drop for Cleanup<'_> {
            fn drop(&mut self) {
                let _ = self.0.revoke_device(self.1);
            }
        }
        let _cleanup = Cleanup(&credentials, &account);
        credentials
            .save_after_approval(
                &account,
                SavedAccessApproval::Approved,
                &DeviceUnlockKey::new([0x5a; 32]),
            )
            .unwrap();
        assert_eq!(
            credentials
                .use_key(&account, |stored| stored
                    .0
                    .iter()
                    .map(|byte| u32::from(*byte))
                    .sum::<u32>())
                .unwrap(),
            2_880
        );
        credentials.revoke_device(&account).unwrap();
    }
}
