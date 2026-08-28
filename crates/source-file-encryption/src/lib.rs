use hkdf::Hkdf;
use libsodium_rs::crypto_secretstream::xchacha20poly1305::{
    ABYTES, HEADERBYTES, Key, PullState, PushState, TAG_FINAL, TAG_MESSAGE,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(feature = "proofs")]
use std::process::{Command, ExitStatus, Stdio};
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

const MAGIC: &[u8; 8] = b"HEMOSRC\0";
const FORMAT_VERSION: u8 = 1;
const CHUNK_BYTES: usize = 1024 * 1024;
const MAX_METADATA_BYTES: usize = 64 * 1024;
const FRAME_DATA: u8 = 0;
const FRAME_METADATA: u8 = 1;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SourceFileKey([u8; 32]);

impl SourceFileKey {
    pub fn generate() -> Result<Self, SourceFileError> {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).map_err(|_| SourceFileError::OperationFailed)?;
        Ok(Self(bytes))
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone)]
pub struct SourceFileContext {
    account_id: String,
    object_id: OpaqueObjectId,
}

impl SourceFileContext {
    pub fn new(account_id: impl Into<String>, object_id: OpaqueObjectId) -> Self {
        Self {
            account_id: account_id.into(),
            object_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueObjectId(String);

impl OpaqueObjectId {
    pub fn parse(value: impl Into<String>) -> Result<Self, SourceFileError> {
        let value = value.into();
        if value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value))
        } else {
            Err(SourceFileError::InvalidObjectId)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for OpaqueObjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone)]
pub struct SourceFileMetadata {
    pub original_filename: String,
    pub media_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecryptedSourceMetadata {
    pub original_filename: String,
    pub media_type: String,
    pub sha256: String,
    pub plaintext_bytes: u64,
}

#[derive(Serialize, Deserialize)]
struct StoredMetadata {
    original_filename: String,
    media_type: String,
    sha256: String,
    plaintext_bytes: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SourceFileError {
    #[error("the encrypted source file is invalid")]
    InvalidSourceFile,
    #[error("the source file operation failed")]
    OperationFailed,
    #[error("the opaque object identifier is invalid")]
    InvalidObjectId,
}

pub fn generate_opaque_object_id() -> Result<OpaqueObjectId, SourceFileError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| SourceFileError::OperationFailed)?;
    OpaqueObjectId::parse(encode_hex(&bytes))
}

pub fn encrypt_source_file(
    mut source: impl Read,
    storage_directory: impl AsRef<Path>,
    context: &SourceFileContext,
    source_key: &SourceFileKey,
    metadata: &SourceFileMetadata,
) -> Result<PathBuf, SourceFileError> {
    libsodium_rs::ensure_init().map_err(|_| SourceFileError::OperationFailed)?;
    let directory = storage_directory.as_ref();
    fs::create_dir_all(directory).map_err(|_| SourceFileError::OperationFailed)?;
    let final_path = directory.join(format!("{}.hemo", context.object_id));
    let partial_path = directory.join(format!("{}.partial", context.object_id));

    let result = (|| {
        let file_key = derive_file_key(source_key, context)?;
        let (mut stream, header) =
            PushState::init_push(&file_key).map_err(|_| SourceFileError::OperationFailed)?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&partial_path)
            .map_err(|_| SourceFileError::OperationFailed)?;
        let mut destination = BufWriter::new(file);
        destination
            .write_all(MAGIC)
            .and_then(|_| destination.write_all(&[FORMAT_VERSION]))
            .and_then(|_| destination.write_all(&header))
            .map_err(|_| SourceFileError::OperationFailed)?;

        let mut buffer = vec![0_u8; CHUNK_BYTES];
        let mut hash = Sha256::new();
        let mut plaintext_bytes = 0_u64;
        let mut frame_index = 0_u64;
        loop {
            let count = source
                .read(&mut buffer)
                .map_err(|_| SourceFileError::OperationFailed)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
            plaintext_bytes = plaintext_bytes
                .checked_add(count as u64)
                .ok_or(SourceFileError::OperationFailed)?;
            let aad = frame_aad(context, FRAME_DATA, frame_index);
            let ciphertext = stream
                .push(&buffer[..count], Some(&aad), TAG_MESSAGE)
                .map_err(|_| SourceFileError::OperationFailed)?;
            write_frame(&mut destination, FRAME_DATA, &ciphertext)?;
            frame_index += 1;
        }

        let stored_metadata = StoredMetadata {
            original_filename: metadata.original_filename.clone(),
            media_type: metadata.media_type.clone(),
            sha256: encode_hex(&hash.finalize()),
            plaintext_bytes,
        };
        let metadata_bytes =
            serde_json::to_vec(&stored_metadata).map_err(|_| SourceFileError::OperationFailed)?;
        if metadata_bytes.len() > MAX_METADATA_BYTES {
            return Err(SourceFileError::OperationFailed);
        }
        let aad = frame_aad(context, FRAME_METADATA, frame_index);
        let ciphertext = stream
            .push(&metadata_bytes, Some(&aad), TAG_FINAL)
            .map_err(|_| SourceFileError::OperationFailed)?;
        write_frame(&mut destination, FRAME_METADATA, &ciphertext)?;
        destination
            .flush()
            .map_err(|_| SourceFileError::OperationFailed)?;
        destination
            .get_ref()
            .sync_all()
            .map_err(|_| SourceFileError::OperationFailed)?;
        drop(destination);
        fs::rename(&partial_path, &final_path).map_err(|_| SourceFileError::OperationFailed)?;
        Ok(final_path.clone())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&partial_path);
    }
    result
}

pub fn decrypt_source_file(
    encrypted_path: impl AsRef<Path>,
    mut plaintext_destination: impl Write,
    context: &SourceFileContext,
    source_key: &SourceFileKey,
) -> Result<DecryptedSourceMetadata, SourceFileError> {
    libsodium_rs::ensure_init().map_err(|_| SourceFileError::InvalidSourceFile)?;
    let mut source =
        BufReader::new(File::open(encrypted_path).map_err(|_| SourceFileError::InvalidSourceFile)?);
    let mut prefix = [0_u8; 9];
    source
        .read_exact(&mut prefix)
        .map_err(|_| SourceFileError::InvalidSourceFile)?;
    if &prefix[..8] != MAGIC || prefix[8] != FORMAT_VERSION {
        return Err(SourceFileError::InvalidSourceFile);
    }
    let mut header = [0_u8; HEADERBYTES];
    source
        .read_exact(&mut header)
        .map_err(|_| SourceFileError::InvalidSourceFile)?;
    let file_key =
        derive_file_key(source_key, context).map_err(|_| SourceFileError::InvalidSourceFile)?;
    let mut stream =
        PullState::init_pull(&header, &file_key).map_err(|_| SourceFileError::InvalidSourceFile)?;
    let mut hash = Sha256::new();
    let mut plaintext_bytes = 0_u64;
    let mut frame_index = 0_u64;

    loop {
        let Some((frame_type, ciphertext)) = read_frame(&mut source)? else {
            return Err(SourceFileError::InvalidSourceFile);
        };
        let aad = frame_aad(context, frame_type, frame_index);
        let (plaintext, tag) = stream
            .pull(&ciphertext, Some(&aad))
            .map_err(|_| SourceFileError::InvalidSourceFile)?;
        match frame_type {
            FRAME_DATA if tag == TAG_MESSAGE => {
                if plaintext.len() > CHUNK_BYTES {
                    return Err(SourceFileError::InvalidSourceFile);
                }
                plaintext_destination
                    .write_all(&plaintext)
                    .map_err(|_| SourceFileError::OperationFailed)?;
                hash.update(&plaintext);
                plaintext_bytes = plaintext_bytes
                    .checked_add(plaintext.len() as u64)
                    .ok_or(SourceFileError::InvalidSourceFile)?;
            }
            FRAME_METADATA if tag == TAG_FINAL => {
                if plaintext.len() > MAX_METADATA_BYTES || read_frame(&mut source)?.is_some() {
                    return Err(SourceFileError::InvalidSourceFile);
                }
                let metadata: StoredMetadata = serde_json::from_slice(&plaintext)
                    .map_err(|_| SourceFileError::InvalidSourceFile)?;
                let actual_hash = encode_hex(&hash.finalize());
                if metadata.sha256 != actual_hash || metadata.plaintext_bytes != plaintext_bytes {
                    return Err(SourceFileError::InvalidSourceFile);
                }
                plaintext_destination
                    .flush()
                    .map_err(|_| SourceFileError::OperationFailed)?;
                return Ok(DecryptedSourceMetadata {
                    original_filename: metadata.original_filename,
                    media_type: metadata.media_type,
                    sha256: metadata.sha256,
                    plaintext_bytes,
                });
            }
            _ => return Err(SourceFileError::InvalidSourceFile),
        }
        frame_index += 1;
    }
}

fn derive_file_key(
    source_key: &SourceFileKey,
    context: &SourceFileContext,
) -> Result<Key, SourceFileError> {
    let hkdf = Hkdf::<Sha256>::new(Some(context.account_id.as_bytes()), &source_key.0);
    let mut bytes = [0_u8; 32];
    let info = format!(
        "hemo-tracker/source-file/v{FORMAT_VERSION}/{}",
        context.object_id
    );
    hkdf.expand(info.as_bytes(), &mut bytes)
        .map_err(|_| SourceFileError::OperationFailed)?;
    let result = Key::from_bytes(&bytes).map_err(|_| SourceFileError::OperationFailed);
    bytes.zeroize();
    result
}

fn frame_aad(context: &SourceFileContext, frame_type: u8, frame_index: u64) -> Vec<u8> {
    format!(
        "hemo-tracker-source-file\0{FORMAT_VERSION}\0{}\0{}\0{frame_type}\0{frame_index}",
        context.account_id, context.object_id
    )
    .into_bytes()
}

#[cfg(feature = "proofs")]
pub struct SourceFileProbe;

#[cfg(feature = "proofs")]
impl SourceFileProbe {
    pub fn interrupt_encryption(
        executable: impl AsRef<Path>,
        storage_directory: impl AsRef<Path>,
        context: &SourceFileContext,
        source_key: &SourceFileKey,
        source_marker: &[u8],
    ) -> Result<ExitStatus, SourceFileError> {
        let mut child = Command::new(executable.as_ref())
            .arg("interrupt-encryption")
            .arg(storage_directory.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| SourceFileError::OperationFailed)?;
        let input = child
            .stdin
            .as_mut()
            .ok_or(SourceFileError::OperationFailed)?;
        write_probe_field(input, &source_key.0)?;
        write_probe_field(input, context.account_id.as_bytes())?;
        write_probe_field(input, context.object_id.as_str().as_bytes())?;
        write_probe_field(input, source_marker)?;
        child.wait().map_err(|_| SourceFileError::OperationFailed)
    }
}

#[cfg(feature = "proofs")]
fn write_probe_field(destination: &mut impl Write, value: &[u8]) -> Result<(), SourceFileError> {
    let length = u32::try_from(value.len()).map_err(|_| SourceFileError::OperationFailed)?;
    destination
        .write_all(&length.to_be_bytes())
        .and_then(|_| destination.write_all(value))
        .map_err(|_| SourceFileError::OperationFailed)
}

fn write_frame(
    destination: &mut impl Write,
    frame_type: u8,
    ciphertext: &[u8],
) -> Result<(), SourceFileError> {
    let length = u32::try_from(ciphertext.len()).map_err(|_| SourceFileError::OperationFailed)?;
    destination
        .write_all(&[frame_type])
        .and_then(|_| destination.write_all(&length.to_be_bytes()))
        .and_then(|_| destination.write_all(ciphertext))
        .map_err(|_| SourceFileError::OperationFailed)
}

fn read_frame(source: &mut impl Read) -> Result<Option<(u8, Vec<u8>)>, SourceFileError> {
    let mut frame_type = [0_u8; 1];
    let count = source
        .read(&mut frame_type)
        .map_err(|_| SourceFileError::InvalidSourceFile)?;
    if count == 0 {
        return Ok(None);
    }
    let mut length = [0_u8; 4];
    source
        .read_exact(&mut length)
        .map_err(|_| SourceFileError::InvalidSourceFile)?;
    let length = u32::from_be_bytes(length) as usize;
    let maximum = match frame_type[0] {
        FRAME_DATA => CHUNK_BYTES + ABYTES,
        FRAME_METADATA => MAX_METADATA_BYTES + ABYTES,
        _ => return Err(SourceFileError::InvalidSourceFile),
    };
    if length < ABYTES || length > maximum {
        return Err(SourceFileError::InvalidSourceFile);
    }
    let mut ciphertext = vec![0_u8; length];
    source
        .read_exact(&mut ciphertext)
        .map_err(|_| SourceFileError::InvalidSourceFile)?;
    Ok(Some((frame_type[0], ciphertext)))
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
