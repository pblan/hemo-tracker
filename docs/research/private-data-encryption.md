# Private data encryption architecture

Status: Research note. This note does not record a decision.

Date: 2026-08-27

## Purpose

This note compares encryption designs for a self-hosted health-data application. The application stores structured lab data and source files. The target is to prevent disclosure to another user. A stronger target is to prevent disclosure to a server administrator.

The two targets are not equal. The design must define the threat model before it selects an encryption layer. OWASP states that the correct encryption layer depends on the threat model. It also states that hardware encryption does not protect data after a remote server compromise. [OWASP cryptographic storage guidance](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)

## Threat models

### Facts

The following actors have different access:

1. An unauthorised application user can send normal application requests.
2. A database or object-storage operator can read stored records, files, and backups.
3. A server administrator can read process memory, change server configuration, and replace application code.
4. A network observer can inspect connection endpoints, traffic times, and traffic sizes. TLS protects content in transit. TLS does not hide all traffic patterns. TLS 1.3 supplies optional record padding because record size can disclose information. [RFC 8446, section 5.4](https://www.rfc-editor.org/rfc/rfc8446.html#section-5.4)
5. A device attacker can inspect browser storage or browser memory on a user's device.

Database access control can isolate normal users. Database encryption at rest does not protect data from a database superuser or from a process that can read database memory. MongoDB documents these limits for its encryption-at-rest feature. [MongoDB Queryable Encryption features](https://www.mongodb.com/docs/manual/core/queryable-encryption/features/)

Application-level encryption can protect a database dump if the keys are separate from the dump. It cannot fully protect a key from an attacker who has full control of the application that uses the key. OWASP states this key-storage limit. [OWASP cryptographic storage guidance](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html#key-storage)

Browser-side encryption can keep plaintext and unwrapped user keys away from the server during normal operation. However, a server administrator who can replace the web application can supply hostile JavaScript on the user's next visit. The script can read plaintext after decryption or can request cryptographic operations. The Web Cryptography specification treats script injection as remote code execution and warns that it can disclose keys or data. [Web Cryptography Level 2, security considerations](https://www.w3.org/TR/WebCryptoAPI/#security-considerations)

Therefore, a web application cannot make a complete claim against a malicious administrator if that administrator controls the server that supplies the client code. A separately installed and signed client, or a client with a verified immutable build, gives a stronger boundary. This boundary still does not protect a compromised user device.

## Architecture comparison

### Server-side encryption at rest

#### Facts

The storage system encrypts database files, object data, disks, or all three. The running server can decrypt the data. This design protects lost disks, copied storage media, and some backup disclosure. It does not protect plaintext in server memory. It does not protect data from an administrator who can use the running application or access its keys.

This design keeps normal database indexes and queries. It also keeps server-side validation, background jobs, plots, and exports simple.

#### Possible use

Use this layer as defence in depth. Do not describe it as protection from a privileged server administrator.

### Per-user envelope encryption on the server

#### Facts

The application creates one or more data-encryption keys for each user. It encrypts each data key with a key-encryption key. This is envelope encryption. A record stores the ciphertext, a nonce, an algorithm version, a wrapped data key or key identifier, and authenticated non-secret context.

A per-user key limits the effect of an accidental cross-user database read. It also supports deletion by key destruction and independent key rotation. The server can still decrypt data during a request if it can unwrap the user key. A malicious server administrator can use or capture that capability.

Authenticated context can bind a ciphertext to a user, record type, and schema version. The context is not secret. KMS implementations can write it to audit logs. AWS warns that an encryption context must not contain sensitive information because it appears in plaintext logs. This warning also applies to equivalent self-hosted KMS metadata. [AWS KMS encryption context](https://docs.aws.amazon.com/kms/latest/developerguide/encrypt_context.html)

#### Possible use

Use a separate data key for each user. Consider a separate data key for each source file. Bind ciphertext to opaque user and object identifiers with authenticated additional data. Keep the key-encryption key outside the database and object store.

This option gives good protection from storage operators and backup disclosure. It does not meet the strongest administrator threat model.

### Client-side or end-to-end encryption

#### Facts

The browser encrypts structured data and files before upload. The server stores ciphertext. The browser decrypts data after download. The server must not receive the user's unwrapped vault key.

This design can protect stored data from a database operator and a server operator during normal operation. It also prevents most server-side processing of protected fields. The server cannot validate plaintext values, create plaintext plots, run plaintext exports, or perform unrestricted searches.

The browser can derive a key-encryption key from a user secret. Argon2id is a memory-hard password-based function. RFC 9106 specifies Argon2id and recommends a unique 16-byte salt for password hashing. The application must tune memory and time cost for its supported devices. [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html)

The Web Cryptography API can use non-extractable `CryptoKey` objects. A non-extractable key cannot pass through `exportKey()` or `wrapKey()`. [MDN `CryptoKey.extractable`](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey/extractable) The Web Cryptography specification permits storage of a `CryptoKey` in IndexedDB. It does not require the browser to encrypt key material on disk. It also warns that users can clear origin storage and destroy a stored key. [Web Cryptography Level 2, key storage](https://www.w3.org/TR/WebCryptoAPI/#concepts-key-storage)

The `extractable` property does not stop hostile same-origin code from asking the key to decrypt data. It is not a defence against a malicious application update.

#### Possible use

Use client-side encryption only after the project defines its trust path for client code. Possible paths include an installed signed client, a browser extension, or a published static client with independent integrity verification. A normal server-supplied web client gives strong protection from passive database access, but only conditional protection from an active malicious administrator.

Keep the vault key in memory only while the vault is unlocked. Store only a password-wrapped vault key or a device-wrapped vault key. Do not store the clear vault key in local storage.

### Searchable and indexed encrypted data

#### Facts

Random authenticated encryption prevents the database from indexing plaintext values. The client can download and decrypt all records, or the design can retain selected plaintext indexes. A third option is a specialised encrypted-query system.

Deterministic encryption permits equality matching, but equal plaintext values produce equal ciphertext values. This discloses equality patterns. MongoDB documents this property for client-side field-level encryption. Its Queryable Encryption system uses a specialised randomised scheme and supports a limited set of query types. Supported operations and field types remain restricted. [MongoDB encryption comparison](https://www.mongodb.com/docs/manual/core/queryable-encryption/about-qe-csfle/) [MongoDB supported operations](https://www.mongodb.com/docs/manual/core/queryable-encryption/reference/supported-operations/)

MongoDB also states that its encrypted query systems do not protect against an attacker who has the customer master key and data keys. They do not protect against arbitrary writes to encrypted collections. A server-controlled schema can also make a client send a field as plaintext. The client must supply and enforce its own schema. [MongoDB encryption comparison](https://www.mongodb.com/docs/manual/core/queryable-encryption/about-qe-csfle/)

Specialised encrypted search adds indexes, metadata, key services, driver constraints, and maintenance work. It does not make every SQL query available on ciphertext.

#### Possible use

For a personal health tracker, first measure whether the client can download and decrypt the user's complete structured dataset. This method has the smallest encrypted-search attack surface. Keep only opaque ownership, synchronisation, object size, and version fields in plaintext.

If scale later requires server-side filtering, document each disclosed index. Separate equality indexes, range indexes, and time-bucket indexes. Do not assume that an encrypted value gives an encrypted access pattern.

## Key recovery and account recovery

### Facts

Encryption makes recovery a product requirement. If only the user has the key and loses it, the server cannot recover the data. If an administrator can reset the password and recover the key without a user-held recovery secret, the administrator can also recover the data.

NIST states that a stored-data encryption key must remain available for as long as the protected data is needed. NIST permits a wrapped copy of that key in backup or archive storage. [NIST SP 800-57 Part 1 Revision 5](https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final)

A password change and an encryption-key rotation are different actions. A password change can rewrap the same vault key. A vault-key rotation can re-encrypt all protected data or keep old wrapped keys until migration finishes.

#### Possible use

Offer a user-held recovery key. Generate it from cryptographic random data. Do not derive it from personal information. Ask the user to save it outside the application.

Do not offer an administrator recovery path if the requirement is administrator exclusion. State clearly that loss of both the password and the recovery key causes permanent data loss.

For a future multi-device feature, wrap the same vault key for each authorised device. Require an existing device or the recovery key to authorise a new device.

## Files and object storage

### Facts

Client-side file encryption can protect PDF and image contents. Plain object keys, original file names, media types, sizes, upload times, user identifiers, thumbnail objects, and access logs can still disclose health activity.

Uploaded files also create an active-content risk. OWASP recommends generated storage names, size limits, type and signature checks, authorised upload, and storage outside the web root. It warns that an external scanning service can disclose files. [OWASP file upload guidance](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html)

Server-side malware scanning cannot inspect end-to-end encrypted content unless the client gives the server plaintext or a temporary key. Client-side inspection has platform and library limits.

#### Possible use

Encrypt each file with a unique data key and an authenticated-encryption algorithm. Store an opaque random object name. Put the original name, media type, and checksum inside the encrypted manifest. Authenticate the file chunks, their order, the report identifier, and the encryption version.

Define file-size buckets if size disclosure is important. Padding increases storage and transfer cost. Do not send health files to a third-party scanner.

## Browser storage and session state

### Facts

IndexedDB uses the browser origin boundary. All application code from the same scheme, host, and port shares that trust boundary. The Web Cryptography specification does not guarantee encrypted disk storage, key zeroisation, or durable key retention. [Web Cryptography Level 2, security considerations](https://www.w3.org/TR/WebCryptoAPI/#security-considerations)

A service worker, an injected script, a compromised dependency, or a hostile application update can act inside the origin. Content Security Policy and TLS reduce some injection and network risks. They do not protect against a server administrator who intentionally supplies authorised hostile code.

#### Possible use

Cache only ciphertext for offline use. Keep decrypted measurements in memory. Clear decrypted state on lock, logout, and inactivity. Treat browser persistence as a cache, not as the only copy of a key.

Use a strict Content Security Policy. Pin dependencies in the build. Do not load third-party scripts on the vault origin. These controls reduce risk. They do not create an administrator-proof web client.

## Backups

### Facts

Backups must contain all ciphertext, wrapped keys, salts, nonces, algorithm versions, and required authenticated context. A backup without the related keys is not recoverable. A backup that contains an administrator-readable master key has the same administrator-access limit as the live system.

Key rotation must account for old backups. OWASP states that old keys can be necessary to decrypt old backups. It also recommends that the system has a rotation process before a compromise occurs. [OWASP cryptographic storage guidance](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html#key-lifetimes-and-rotation)

#### Possible use

Test a full restore regularly. Test the user recovery flow after restore. Include ciphertext and wrapped keys in normal backups. Store any user-held recovery secret separately. Define retention and deletion rules for backup copies.

## Logs and metadata

### Facts

Encryption of primary storage does not clean logs, error reports, traces, metrics, caches, swap, temporary files, or generated thumbnails. OWASP states that applications should not record health data, access tokens, passwords, encryption keys, or primary secrets directly in logs. It also states that file paths can need special treatment. [OWASP logging guidance](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html#data-to-exclude)

Even if content is encrypted, the service can observe account identifiers, IP addresses, report counts, object sizes, upload times, access times, and query patterns. TLS record padding can reduce size disclosure, but padding policy has operational cost and TLS does not define one universal policy. [RFC 8446, section 5.4](https://www.rfc-editor.org/rfc/rfc8446.html#section-5.4)

#### Possible use

Use opaque identifiers in logs. Do not log analyte names, values, file names, report dates, request bodies, cryptographic context, or decrypted errors. Set short and explicit retention periods. Audit key use and sensitive operations without recording the sensitive content.

Review reverse-proxy, database, object-storage, browser telemetry, and crash-report configuration. Disable third-party analytics on the health-data origin.

## Operational limits

### Facts

Client-side encryption moves work to each user device. It increases upload, download, memory, and battery use. It makes server-side rendering and background processing of plaintext unavailable. It makes password recovery and support more difficult. It can also make data migration dependent on an unlocked client.

Per-user server-side encryption is simpler. It supports normal SQL queries and server jobs. Its administrator protection is weaker because the server holds or can obtain the keys.

Encrypted search can retain some server queries. It adds specialised technology and limits query forms. It can also disclose schema, query type, result size, access time, and other metadata.

### Options for later ADR work

The project can compare these options in an ADR:

1. Server-side per-user envelope encryption, plus encrypted disks and backups.
2. Browser-side encryption with a server-supplied web client and an explicit active-administrator limitation.
3. End-to-end encryption with a separately installed or independently verified client.
4. A staged design. Start with envelope encryption, but make ciphertext envelopes and key ownership compatible with a later client-side boundary.

The ADR must state which administrator actions are inside the threat model. It must also state the recovery promise, the searchable fields, the permitted plaintext metadata, and the trusted client distribution method.

## Questions that need a project decision

1. Must protection resist passive administrator access only, or must it resist an active administrator who changes application code?
2. Can the project require a separately installed client?
3. Is permanent data loss acceptable after loss of the password and recovery key?
4. Must a user use the application on several devices?
5. Must the server query analyte values and dates, or can the client decrypt the full personal dataset?
6. Which metadata can remain in plaintext?
7. Can file size remain visible?
8. Which background tasks must run while the user's vault is locked?
9. What backup retention and deletion periods apply?
10. Which device types set the minimum performance target for key derivation and decryption?
