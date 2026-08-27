use hemo_key_lifecycle_proof::{AccountKeyBundle, Passphrase};

fn main() {
    let passphrase = Passphrase::new("local proof passphrase");
    let bundle = AccountKeyBundle::create("proof-account", &passphrase)
        .expect("the operating system must provide secure randomness");
    let envelope = bundle.passphrase_envelope();

    println!("format version: {}", envelope.version());
    println!("account: {}", envelope.account_id());
    println!(
        "passphrase unlock: {}",
        envelope.unlock_with_passphrase(&passphrase).is_ok()
    );
    println!(
        "recovery unlock: {}",
        bundle
            .recovery_envelope()
            .unlock_with_recovery(bundle.recovery_key())
            .is_ok()
    );
}
