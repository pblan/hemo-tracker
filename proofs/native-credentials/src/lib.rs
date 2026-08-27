use std::collections::HashMap;
use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

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

    fn expose(&self) -> &[u8; 32] {
        &self.0
    }
}

pub trait CredentialStore: Send + Sync {
    fn save(&self, account: &str, encoded_key: &str) -> Result<(), CredentialError>;
    fn load(&self, account: &str) -> Result<String, CredentialError>;
    fn delete(&self, account: &str) -> Result<(), CredentialError>;
}

pub struct NativeCredentialStore;

impl CredentialStore for NativeCredentialStore {
    fn save(&self, account: &str, encoded_key: &str) -> Result<(), CredentialError> {
        keyring::Entry::new(SERVICE, account)
            .and_then(|entry| entry.set_password(encoded_key))
            .map_err(|_| CredentialError::Store)
    }

    fn load(&self, account: &str) -> Result<String, CredentialError> {
        keyring::Entry::new(SERVICE, account)
            .and_then(|entry| entry.get_password())
            .map_err(|error| match error {
                keyring::Error::NoEntry => CredentialError::NotFound,
                _ => CredentialError::Store,
            })
    }

    fn delete(&self, account: &str) -> Result<(), CredentialError> {
        keyring::Entry::new(SERVICE, account)
            .and_then(|entry| entry.delete_credential())
            .map_err(|error| match error {
                keyring::Error::NoEntry => CredentialError::NotFound,
                _ => CredentialError::Store,
            })
    }
}

pub struct TrustedDeviceCredentials<S> {
    store: S,
}

impl<S: CredentialStore> TrustedDeviceCredentials<S> {
    pub fn new(store: S) -> Self {
        Self { store }
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
        self.store
            .save(account, &STANDARD_NO_PAD.encode(key.expose()))
    }

    pub fn use_key<T>(
        &self,
        account: &str,
        operation: impl FnOnce(&DeviceUnlockKey) -> T,
    ) -> Result<T, CredentialError> {
        let mut encoded = self.store.load(account)?;
        let decoded = STANDARD_NO_PAD
            .decode(&encoded)
            .map_err(|_| CredentialError::InvalidFormat)?;
        encoded.zeroize();
        let bytes: [u8; 32] = decoded
            .try_into()
            .map_err(|_| CredentialError::InvalidFormat)?;
        let key = DeviceUnlockKey::new(bytes);
        Ok(operation(&key))
    }

    pub fn logout(&self, account: &str) -> Result<(), CredentialError> {
        self.store.delete(account)
    }

    pub fn revoke_device(&self, account: &str) -> Result<(), CredentialError> {
        self.store.delete(account)
    }
}

#[derive(Default)]
pub struct MemoryCredentialStore(Mutex<HashMap<String, String>>);

impl CredentialStore for MemoryCredentialStore {
    fn save(&self, account: &str, encoded_key: &str) -> Result<(), CredentialError> {
        self.0
            .lock()
            .map_err(|_| CredentialError::Store)?
            .insert(account.into(), encoded_key.into());
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

    #[test]
    fn approval_is_required_before_save() {
        let credentials = TrustedDeviceCredentials::new(MemoryCredentialStore::default());
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
        let credentials = TrustedDeviceCredentials::new(MemoryCredentialStore::default());
        let key = DeviceUnlockKey::new([9; 32]);
        credentials
            .save_after_approval("account", SavedAccessApproval::Approved, &key)
            .unwrap();
        let checksum = credentials
            .use_key("account", |stored| {
                stored
                    .expose()
                    .iter()
                    .map(|byte| u32::from(*byte))
                    .sum::<u32>()
            })
            .unwrap();
        assert_eq!(checksum, 288);
    }

    #[test]
    fn logout_and_revocation_remove_saved_access() {
        for revoke in [false, true] {
            let credentials = TrustedDeviceCredentials::new(MemoryCredentialStore::default());
            let key = DeviceUnlockKey::new([3; 32]);
            credentials
                .save_after_approval("account", SavedAccessApproval::Approved, &key)
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
}
