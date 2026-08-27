use hemo_encrypted_vault_proof::{NativeVaultManager, VaultKey, VaultProbe};
use std::io::{self, Read};

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.get(1).map(String::as_str) != Some("crash-write") {
        println!("encrypted vault proof helper");
        return;
    }

    let Some(path) = arguments.get(2) else {
        std::process::exit(2);
    };
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.len() < 32 {
        std::process::exit(2);
    }
    let mut key_bytes = [0_u8; 32];
    key_bytes.copy_from_slice(&input[..32]);
    let Ok(source_text) = String::from_utf8(input[32..].to_vec()) else {
        std::process::exit(2);
    };
    let key = VaultKey::from_bytes(key_bytes);
    let Ok(vault) = NativeVaultManager::open(path, &key) else {
        std::process::exit(2);
    };
    if VaultProbe::write_uncommitted(&vault, &source_text).is_err() {
        std::process::exit(2);
    }
    std::process::exit(17);
}
