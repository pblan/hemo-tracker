# Backup and recovery

## Purpose

Use an encrypted backup to preserve a local account vault. Keep at least one backup on a separate device.

## Encrypted backup

An encrypted backup contains the account manifest, encrypted database, encrypted source files, and format metadata. The backup does not contain a plaintext passphrase or recovery key.

Store the backup and the recovery material separately. Test a restore on a clean computer before you depend on the backup.

## Plaintext export

A decrypted export is a plaintext ZIP copy. It contains one JSON file for each report, a `measurements.csv` file, and decrypted source files. Use it only when you need to share data with a trusted person or tool. Check the destination before export. Delete the export after use. Do not place it in a shared folder or cloud service unless that service is approved for the data.

Restore validates the backup before it replaces the active vault. The application stages the replacement and keeps the prior vault until the replacement opens and passes integrity checks.

## Recovery limit

Loss of every local vault, encrypted backup, passphrase, and recovery key is permanent. Hemo Tracker cannot recover these items for you.

## Deletion

Archive a report for normal removal. Permanent deletion is a separate action and needs explicit confirmation. Permanent deletion cannot recover the report or its encrypted source files.
