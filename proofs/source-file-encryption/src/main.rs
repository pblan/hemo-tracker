use hemo_source_file_encryption_proof::{
    OpaqueObjectId, SourceFileContext, SourceFileKey, SourceFileMetadata, encrypt_source_file,
};
use std::io::{self, Read};

fn main() {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.get(1).map(String::as_str) != Some("interrupt-encryption") {
        println!("source-file encryption proof helper");
        return;
    }
    let Some(directory) = arguments.get(2) else {
        std::process::exit(2);
    };
    let mut input = io::stdin();
    let Ok(key) = read_field(&mut input) else {
        std::process::exit(2);
    };
    let Ok(account_id) = read_field(&mut input) else {
        std::process::exit(2);
    };
    let Ok(object_id) = read_field(&mut input) else {
        std::process::exit(2);
    };
    let Ok(marker) = read_field(&mut input) else {
        std::process::exit(2);
    };
    let Ok(key_bytes) = <[u8; 32]>::try_from(key) else {
        std::process::exit(2);
    };
    let Ok(account_id) = String::from_utf8(account_id) else {
        std::process::exit(2);
    };
    let Ok(object_id) = String::from_utf8(object_id)
        .map_err(|_| ())
        .and_then(|value| OpaqueObjectId::parse(value).map_err(|_| ()))
    else {
        std::process::exit(2);
    };

    let reader = InterruptingReader {
        marker,
        first_read: true,
    };
    let _ = encrypt_source_file(
        reader,
        directory,
        &SourceFileContext::new(account_id, object_id),
        &SourceFileKey::from_bytes(key_bytes),
        &SourceFileMetadata {
            original_filename: "probe.pdf".to_owned(),
            media_type: "application/pdf".to_owned(),
        },
    );
    std::process::exit(2);
}

fn read_field(source: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut length = [0_u8; 4];
    source.read_exact(&mut length)?;
    let mut value = vec![0_u8; u32::from_be_bytes(length) as usize];
    source.read_exact(&mut value)?;
    Ok(value)
}

struct InterruptingReader {
    marker: Vec<u8>,
    first_read: bool,
}

impl Read for InterruptingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if !self.first_read {
            std::process::exit(17);
        }
        self.first_read = false;
        for (index, byte) in buffer.iter_mut().enumerate() {
            *byte = self.marker[index % self.marker.len()];
        }
        Ok(buffer.len())
    }
}
