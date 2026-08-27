# Native credential storage research

Date: 2026-08-28

## Question

How can a trusted Hemo Tracker device store a small device-unlock key on macOS and Windows?

## Platform facts

Apple Keychain can restrict an item to an unlocked device and can require user presence. A `ThisDeviceOnly` accessibility class prevents migration to a different device through a backup. A signed application identifier and keychain access group control application access. See [Restricting keychain item accessibility](https://developer.apple.com/documentation/security/restricting-keychain-item-accessibility) and [Sharing access to keychain items](https://developer.apple.com/documentation/security/sharing-access-to-keychain-items-among-a-collection-of-apps).

An application upgrade with the same signed application identity and access group can keep access to a Keychain item. A different signing team cannot claim the protected access group without an authorized provisioning profile. A signed test must verify the exact Hemo Tracker entitlements.

Windows Credential Locker stores small credentials through `PasswordVault`. Microsoft states that credentials can roam with the user's Microsoft account. The store has a limit of 20 credentials per application. Microsoft requires the user to opt in before an application saves a credential. See [Credential Locker for Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/security/credential-locker).

The `keyring` 4.1.6 Rust crate provides one interface for Apple Keychain and Windows Credential Locker. Its default V1 facade selects the native store on each supported target. The proof pins this version and keeps its interface behind a project-owned module. See [`keyring` 4.1.6](https://docs.rs/keyring/4.1.6/keyring/).

## Recommendation

Use `keyring` behind the trusted Rust module. Store one 32-byte device-unlock key for each account as a native credential. Require an explicit in-application approval before the first save.

Do not return the raw key from a Tauri command. Load the key inside Rust, perform the requested key operation, and return only the operation result.

Delete the credential during logout and device revocation. Treat a missing credential after reinstall, operating-system restore, account change, or signing-identity change as a normal state that requires the account passphrase or recovery process.

For macOS production, select a non-synchronizing data-protection Keychain item with a `ThisDeviceOnly` accessibility class. For Windows, tell the user that Credential Locker can roam with a Microsoft account. Do not claim that the Windows item is device-only.
