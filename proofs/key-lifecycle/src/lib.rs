use argon2::{Algorithm, Argon2, Params, Version};
use base64ct::{Base64UrlUnpadded, Encoding};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use subtle::ConstantTimeEq;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

const FORMAT: &str = "hemo-tracker-key-envelope";
const VERSION: u8 = 1;
const KEY_BYTES: usize = 32;
const SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 24;
const ARGON2_MEMORY_KIB: u32 = 65_536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_LANES: u32 = 1;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Passphrase(Vec<u8>);

impl Passphrase {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().as_bytes().to_vec())
    }
}

#[derive(Clone, Copy)]
pub enum Purpose {
    Database,
    SourceFiles,
    SyncManifest,
}

impl Purpose {
    fn label(self) -> &'static str {
        match self {
            Self::Database => "database",
            Self::SourceFiles => "source-files",
            Self::SyncManifest => "sync-manifest",
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum KeyLifecycleError {
    #[error("credentials or key envelope are invalid")]
    InvalidCredentialsOrEnvelope,
    #[error("key envelope format is not supported")]
    UnsupportedEnvelope,
    #[error("could not generate secure key material")]
    RandomGeneration,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct UnlockedKeys {
    account_id: String,
    account_data_key: [u8; KEY_BYTES],
}

impl fmt::Debug for UnlockedKeys {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UnlockedKeys")
            .field("account_id", &self.account_id)
            .field("account_data_key", &"[REDACTED]")
            .finish()
    }
}

impl UnlockedKeys {
    pub fn derive_purpose_key(&self, purpose: Purpose, generation: u32) -> PurposeKey {
        let hkdf = Hkdf::<Sha256>::new(Some(self.account_id.as_bytes()), &self.account_data_key);
        let info = format!(
            "hemo-tracker/v1/{}/generation/{generation}",
            purpose.label()
        );
        let mut bytes = [0_u8; KEY_BYTES];
        hkdf.expand(info.as_bytes(), &mut bytes)
            .expect("32 bytes is a valid HKDF output length");
        PurposeKey(bytes)
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct PurposeKey([u8; KEY_BYTES]);

impl PurposeKey {
    pub fn matches(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }

    pub fn bytes(&self) -> &[u8; KEY_BYTES] {
        &self.0
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct RecoveryKey([u8; KEY_BYTES]);

impl fmt::Debug for RecoveryKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecoveryKey([REDACTED])")
    }
}

impl RecoveryKey {
    pub fn to_code(&self) -> RecoveryCode {
        let checksum = Sha256::digest(self.0);
        let mut encoded = [0_u8; KEY_BYTES + 4];
        encoded[..KEY_BYTES].copy_from_slice(&self.0);
        encoded[KEY_BYTES..].copy_from_slice(&checksum[..4]);
        RecoveryCode(format!(
            "HTRK1-{}",
            Base64UrlUnpadded::encode_string(&encoded)
        ))
    }

    pub fn from_code(code: &str) -> Result<Self, KeyLifecycleError> {
        let payload = code
            .strip_prefix("HTRK1-")
            .ok_or(KeyLifecycleError::InvalidCredentialsOrEnvelope)?;
        let mut decoded = Base64UrlUnpadded::decode_vec(payload)
            .map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)?;
        if decoded.len() != KEY_BYTES + 4 {
            decoded.zeroize();
            return Err(KeyLifecycleError::InvalidCredentialsOrEnvelope);
        }
        let expected = Sha256::digest(&decoded[..KEY_BYTES]);
        if !bool::from(decoded[KEY_BYTES..].ct_eq(&expected[..4])) {
            decoded.zeroize();
            return Err(KeyLifecycleError::InvalidCredentialsOrEnvelope);
        }
        let mut key = [0_u8; KEY_BYTES];
        key.copy_from_slice(&decoded[..KEY_BYTES]);
        decoded.zeroize();
        Ok(Self(key))
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RecoveryCode(String);

impl RecoveryCode {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for RecoveryCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecoveryCode([REDACTED])")
    }
}

#[derive(Clone)]
pub struct AccountKeyBundle {
    unlocked_keys: UnlockedKeys,
    passphrase_envelope: KeyEnvelope,
    recovery_envelope: KeyEnvelope,
    recovery_key: RecoveryKey,
}

impl AccountKeyBundle {
    pub fn create(account_id: &str, passphrase: &Passphrase) -> Result<Self, KeyLifecycleError> {
        let account_data_key = random_array::<KEY_BYTES>()?;
        let recovery_key = RecoveryKey(random_array::<KEY_BYTES>()?);
        let unlocked_keys = UnlockedKeys {
            account_id: account_id.to_owned(),
            account_data_key,
        };
        let passphrase_envelope = KeyEnvelope::wrap_with_passphrase(&unlocked_keys, passphrase)?;
        let recovery_envelope = KeyEnvelope::wrap_with_recovery(&unlocked_keys, &recovery_key)?;

        Ok(Self {
            unlocked_keys,
            passphrase_envelope,
            recovery_envelope,
            recovery_key,
        })
    }

    pub fn unlocked_keys(&self) -> &UnlockedKeys {
        &self.unlocked_keys
    }

    pub fn passphrase_envelope(&self) -> &KeyEnvelope {
        &self.passphrase_envelope
    }

    pub fn recovery_envelope(&self) -> &KeyEnvelope {
        &self.recovery_envelope
    }

    pub fn recovery_key(&self) -> &RecoveryKey {
        &self.recovery_key
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyEnvelope {
    format: String,
    version: u8,
    account_id: String,
    protector: Protector,
    cipher: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum Protector {
    Passphrase {
        kdf: String,
        memory_kib: u32,
        iterations: u32,
        lanes: u32,
        salt: String,
    },
    RecoveryKey,
}

impl KeyEnvelope {
    fn wrap_with_passphrase(
        keys: &UnlockedKeys,
        passphrase: &Passphrase,
    ) -> Result<Self, KeyLifecycleError> {
        let salt = random_array::<SALT_BYTES>()?;
        let mut wrapping_key = derive_passphrase_key(passphrase, &salt)?;
        let protector = Protector::Passphrase {
            kdf: "argon2id-v19".to_owned(),
            memory_kib: ARGON2_MEMORY_KIB,
            iterations: ARGON2_ITERATIONS,
            lanes: ARGON2_LANES,
            salt: Base64UrlUnpadded::encode_string(&salt),
        };
        let result = Self::wrap(keys, protector, &wrapping_key);
        wrapping_key.zeroize();
        result
    }

    fn wrap_with_recovery(
        keys: &UnlockedKeys,
        recovery_key: &RecoveryKey,
    ) -> Result<Self, KeyLifecycleError> {
        Self::wrap(keys, Protector::RecoveryKey, &recovery_key.0)
    }

    fn wrap(
        keys: &UnlockedKeys,
        protector: Protector,
        wrapping_key: &[u8; KEY_BYTES],
    ) -> Result<Self, KeyLifecycleError> {
        let nonce = random_array::<NONCE_BYTES>()?;
        let mut envelope = Self {
            format: FORMAT.to_owned(),
            version: VERSION,
            account_id: keys.account_id.clone(),
            protector,
            cipher: "xchacha20-poly1305".to_owned(),
            nonce: Base64UrlUnpadded::encode_string(&nonce),
            ciphertext: String::new(),
        };
        let cipher = XChaCha20Poly1305::new(wrapping_key.into());
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &keys.account_data_key,
                    aad: envelope.aad().as_bytes(),
                },
            )
            .map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)?;
        envelope.ciphertext = Base64UrlUnpadded::encode_string(&ciphertext);
        Ok(envelope)
    }

    pub fn unlock_with_passphrase(
        &self,
        passphrase: &Passphrase,
    ) -> Result<UnlockedKeys, KeyLifecycleError> {
        let Protector::Passphrase {
            kdf,
            memory_kib,
            iterations,
            lanes,
            salt,
        } = &self.protector
        else {
            return Err(KeyLifecycleError::InvalidCredentialsOrEnvelope);
        };
        if kdf != "argon2id-v19"
            || *memory_kib != ARGON2_MEMORY_KIB
            || *iterations != ARGON2_ITERATIONS
            || *lanes != ARGON2_LANES
        {
            return Err(KeyLifecycleError::UnsupportedEnvelope);
        }
        let salt = decode_array::<SALT_BYTES>(salt)?;
        let mut wrapping_key = derive_passphrase_key(passphrase, &salt)?;
        let result = self.unlock(&wrapping_key);
        wrapping_key.zeroize();
        result
    }

    pub fn unlock_with_recovery(
        &self,
        recovery_key: &RecoveryKey,
    ) -> Result<UnlockedKeys, KeyLifecycleError> {
        if !matches!(self.protector, Protector::RecoveryKey) {
            return Err(KeyLifecycleError::InvalidCredentialsOrEnvelope);
        }
        self.unlock(&recovery_key.0)
    }

    fn unlock(&self, wrapping_key: &[u8; KEY_BYTES]) -> Result<UnlockedKeys, KeyLifecycleError> {
        self.validate_header()?;
        let nonce = decode_array::<NONCE_BYTES>(&self.nonce)?;
        let ciphertext = Base64UrlUnpadded::decode_vec(&self.ciphertext)
            .map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)?;
        let cipher = XChaCha20Poly1305::new(wrapping_key.into());
        let mut plaintext = cipher
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: self.aad().as_bytes(),
                },
            )
            .map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)?;
        if plaintext.len() != KEY_BYTES {
            plaintext.zeroize();
            return Err(KeyLifecycleError::InvalidCredentialsOrEnvelope);
        }
        let mut account_data_key = [0_u8; KEY_BYTES];
        account_data_key.copy_from_slice(&plaintext);
        plaintext.zeroize();
        Ok(UnlockedKeys {
            account_id: self.account_id.clone(),
            account_data_key,
        })
    }

    pub fn change_passphrase(
        &self,
        old_passphrase: &Passphrase,
        new_passphrase: &Passphrase,
    ) -> Result<Self, KeyLifecycleError> {
        let keys = self.unlock_with_passphrase(old_passphrase)?;
        Self::wrap_with_passphrase(&keys, new_passphrase)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("key envelope serialization cannot fail")
    }

    pub fn from_json(value: &str) -> Result<Self, KeyLifecycleError> {
        serde_json::from_str(value).map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    fn validate_header(&self) -> Result<(), KeyLifecycleError> {
        if self.format != FORMAT || self.version != VERSION || self.cipher != "xchacha20-poly1305" {
            return Err(KeyLifecycleError::UnsupportedEnvelope);
        }
        Ok(())
    }

    fn aad(&self) -> String {
        let protector = match &self.protector {
            Protector::Passphrase {
                kdf,
                memory_kib,
                iterations,
                lanes,
                salt,
            } => format!("passphrase:{kdf}:{memory_kib}:{iterations}:{lanes}:{salt}"),
            Protector::RecoveryKey => "recovery-key".to_owned(),
        };
        format!(
            "{}\0{}\0{}\0{}\0{}",
            self.format, self.version, self.account_id, self.cipher, protector
        )
    }
}

fn derive_passphrase_key(
    passphrase: &Passphrase,
    salt: &[u8; SALT_BYTES],
) -> Result<[u8; KEY_BYTES], KeyLifecycleError> {
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_LANES,
        Some(KEY_BYTES),
    )
    .map_err(|_| KeyLifecycleError::UnsupportedEnvelope)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut output = [0_u8; KEY_BYTES];
    argon2
        .hash_password_into(&passphrase.0, salt, &mut output)
        .map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)?;
    Ok(output)
}

fn random_array<const N: usize>() -> Result<[u8; N], KeyLifecycleError> {
    let mut value = [0_u8; N];
    getrandom::fill(&mut value).map_err(|_| KeyLifecycleError::RandomGeneration)?;
    Ok(value)
}

fn decode_array<const N: usize>(value: &str) -> Result<[u8; N], KeyLifecycleError> {
    let decoded = Base64UrlUnpadded::decode_vec(value)
        .map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)?;
    decoded
        .try_into()
        .map_err(|_| KeyLifecycleError::InvalidCredentialsOrEnvelope)
}
