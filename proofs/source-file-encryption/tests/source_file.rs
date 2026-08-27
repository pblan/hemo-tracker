use hemo_source_file_encryption_proof::{
    DecryptedSourceMetadata, OpaqueObjectId, SourceFileContext, SourceFileError, SourceFileKey,
    SourceFileMetadata, SourceFileProbe, decrypt_source_file, encrypt_source_file,
    generate_opaque_object_id,
};
use std::{fs, io::Cursor};
use tempfile::tempdir;

const SOURCE_TEXT: &[u8] = b"Fictional ferritin result: 42.7 ug/L";

fn context() -> SourceFileContext {
    SourceFileContext::new("account-1", OpaqueObjectId::parse("a".repeat(64)).unwrap())
}

fn metadata() -> SourceFileMetadata {
    SourceFileMetadata {
        original_filename: "fictional-lab-report.pdf".to_owned(),
        media_type: "application/pdf".to_owned(),
    }
}

#[test]
fn source_file_round_trip_keeps_metadata_inside_ciphertext() {
    let directory = tempdir().unwrap();
    let key = SourceFileKey::generate().unwrap();
    let encrypted = encrypt_source_file(
        Cursor::new(SOURCE_TEXT),
        directory.path(),
        &context(),
        &key,
        &metadata(),
    )
    .unwrap();

    let ciphertext = fs::read(&encrypted).unwrap();
    for secret in [
        SOURCE_TEXT,
        metadata().original_filename.as_bytes(),
        metadata().media_type.as_bytes(),
    ] {
        assert!(!ciphertext.windows(secret.len()).any(|part| part == secret));
    }

    let mut plaintext = Vec::new();
    let restored = decrypt_source_file(&encrypted, &mut plaintext, &context(), &key).unwrap();
    assert_eq!(plaintext, SOURCE_TEXT);
    assert_eq!(restored.original_filename, metadata().original_filename);
    assert_eq!(restored.media_type, metadata().media_type);
    assert_eq!(restored.plaintext_bytes, SOURCE_TEXT.len() as u64);
}

#[test]
fn changed_truncated_reordered_and_duplicated_frames_are_rejected() {
    let directory = tempdir().unwrap();
    let key = SourceFileKey::generate().unwrap();
    let encrypted = encrypt_source_file(
        Cursor::new(vec![7_u8; 3 * 1024 * 1024]),
        directory.path(),
        &context(),
        &key,
        &metadata(),
    )
    .unwrap();
    let original = fs::read(&encrypted).unwrap();

    for (name, changed) in [
        ("changed", change_ciphertext(&original)),
        ("truncated", original[..original.len() - 10].to_vec()),
        ("reordered", reorder_data_frames(&original)),
        ("duplicated", duplicate_data_frame(&original)),
    ] {
        let path = directory.path().join(format!("{name}.encrypted"));
        fs::write(&path, changed).unwrap();
        assert!(matches!(
            decrypt_source_file(&path, &mut Vec::new(), &context(), &key),
            Err(SourceFileError::InvalidSourceFile)
        ));
    }
}

#[test]
fn account_object_and_version_are_authenticated() {
    let directory = tempdir().unwrap();
    let key = SourceFileKey::generate().unwrap();
    let encrypted = encrypt_source_file(
        Cursor::new(SOURCE_TEXT),
        directory.path(),
        &context(),
        &key,
        &metadata(),
    )
    .unwrap();

    for wrong in [
        SourceFileContext::new("account-2", OpaqueObjectId::parse("a".repeat(64)).unwrap()),
        SourceFileContext::new("account-1", OpaqueObjectId::parse("b".repeat(64)).unwrap()),
    ] {
        assert!(decrypt_source_file(&encrypted, &mut Vec::new(), &wrong, &key).is_err());
    }
    let mut changed = fs::read(&encrypted).unwrap();
    changed[8] = 2;
    let changed_path = directory.path().join("wrong-version.encrypted");
    fs::write(&changed_path, changed).unwrap();
    assert!(decrypt_source_file(&changed_path, &mut Vec::new(), &context(), &key).is_err());
}

#[test]
fn object_identifiers_and_per_object_keys_are_separate() {
    let first_id = generate_opaque_object_id().unwrap();
    let second_id = generate_opaque_object_id().unwrap();
    assert_ne!(first_id, second_id);
    assert_eq!(first_id.len(), 64);

    let directory = tempdir().unwrap();
    let key = SourceFileKey::generate().unwrap();
    let first_context = SourceFileContext::new("account-1", first_id.clone());
    let second_context = SourceFileContext::new("account-1", second_id);
    let encrypted = encrypt_source_file(
        Cursor::new(SOURCE_TEXT),
        directory.path(),
        &first_context,
        &key,
        &metadata(),
    )
    .unwrap();
    assert!(decrypt_source_file(&encrypted, &mut Vec::new(), &second_context, &key).is_err());
}

#[test]
fn identifying_or_unsafe_object_identifiers_are_rejected() {
    for value in [
        "lab-report.pdf".to_owned(),
        "../outside".to_owned(),
        "A".repeat(64),
        "g".repeat(64),
    ] {
        assert!(matches!(
            OpaqueObjectId::parse(value),
            Err(SourceFileError::InvalidObjectId)
        ));
    }
}

#[test]
fn a_failed_read_removes_the_partial_ciphertext() {
    let directory = tempdir().unwrap();
    let key = SourceFileKey::generate().unwrap();
    let result = encrypt_source_file(
        FailingReader::default(),
        directory.path(),
        &context(),
        &key,
        &metadata(),
    );
    assert!(result.is_err());
    assert!(fs::read_dir(directory.path()).unwrap().next().is_none());
}

#[test]
fn process_interruption_leaves_only_ciphertext_partial_output() {
    let directory = tempdir().unwrap();
    let key = SourceFileKey::generate().unwrap();
    let status = SourceFileProbe::interrupt_encryption(
        env!("CARGO_BIN_EXE_hemo-source-file-encryption-proof"),
        directory.path(),
        &context(),
        &key,
        SOURCE_TEXT,
    )
    .unwrap();
    assert_eq!(status.code(), Some(17));

    let files = fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].extension().unwrap(), "partial");
    let partial = fs::read(&files[0]).unwrap();
    assert!(
        !partial
            .windows(SOURCE_TEXT.len())
            .any(|part| part == SOURCE_TEXT)
    );
}

#[test]
#[ignore = "run as the manual multi-gigabyte release proof"]
fn multi_gigabyte_stream_has_bounded_buffers() {
    let directory = tempdir().unwrap();
    let key = SourceFileKey::generate().unwrap();
    let size = 2_u64 * 1024 * 1024 * 1024 + 1;
    let encrypted = encrypt_source_file(
        PatternReader::new(size),
        directory.path(),
        &context(),
        &key,
        &metadata(),
    )
    .unwrap();
    let mut sink = CountingSink::default();
    let restored: DecryptedSourceMetadata =
        decrypt_source_file(&encrypted, &mut sink, &context(), &key).unwrap();
    assert_eq!(sink.bytes, size);
    assert_eq!(restored.plaintext_bytes, size);
}

fn frames(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut offset = 9 + 24;
    let mut result = Vec::new();
    while offset < bytes.len() {
        let length = u32::from_be_bytes(bytes[offset + 1..offset + 5].try_into().unwrap()) as usize;
        let end = offset + 5 + length;
        result.push((offset, end));
        offset = end;
    }
    result
}

fn change_ciphertext(original: &[u8]) -> Vec<u8> {
    let mut changed = original.to_vec();
    let index = changed.len() / 2;
    changed[index] ^= 0x80;
    changed
}

fn reorder_data_frames(original: &[u8]) -> Vec<u8> {
    let ranges = frames(original);
    let mut changed = original[..ranges[0].0].to_vec();
    changed.extend_from_slice(&original[ranges[1].0..ranges[1].1]);
    changed.extend_from_slice(&original[ranges[0].0..ranges[0].1]);
    for range in &ranges[2..] {
        changed.extend_from_slice(&original[range.0..range.1]);
    }
    changed
}

fn duplicate_data_frame(original: &[u8]) -> Vec<u8> {
    let ranges = frames(original);
    let mut changed = original[..ranges[1].1].to_vec();
    changed.extend_from_slice(&original[ranges[1].0..ranges[1].1]);
    changed.extend_from_slice(&original[ranges[1].1..]);
    changed
}

#[derive(Default)]
struct FailingReader {
    reads: usize,
}

impl std::io::Read for FailingReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.reads += 1;
        if self.reads > 1 {
            return Err(std::io::Error::other("planned read failure"));
        }
        buffer[..16].fill(1);
        Ok(16)
    }
}

struct PatternReader {
    remaining: u64,
}

impl PatternReader {
    fn new(remaining: u64) -> Self {
        Self { remaining }
    }
}

impl std::io::Read for PatternReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = usize::try_from(self.remaining.min(buffer.len() as u64)).unwrap();
        buffer[..count].fill(0x5a);
        self.remaining -= count as u64;
        Ok(count)
    }
}

#[derive(Default)]
struct CountingSink {
    bytes: u64,
}

impl std::io::Write for CountingSink {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.bytes += buffer.len() as u64;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
