# User documentation

This directory contains the V1 user guides. Issue 26 adds the procedures and current product screenshots after the related product workflows are complete.

Start with the [project README](../../README.md) until the V1 quickstart is available.

## Hemo Tracker user guide

Hemo Tracker stores laboratory reports in an encrypted local vault. It does not give medical advice.

## Quick start

1. Start Hemo Tracker.
2. Select **Create your local vault**.
3. Enter and confirm a strong passphrase.
4. Store the recovery key in a separate safe place.
5. Select **I stored the recovery key**.
6. Enter the report collection date and time.
7. Select a source-file role and choose the original report file.
8. Select a saved analyte, or enter a new analyte identity.
9. Enter the source label, value, unit, interval, and flag.
10. Select **Choose source file and save report**.

Result: The report is stored in the encrypted local vault. The original source file is not changed.

## Correct a measurement

Use the correction action for a value that was entered incorrectly. Review the source file before you confirm the correction.

The correction changes the current value. The source file remains immutable. Hemo Tracker records the update time and local user identity.

## Safety limits

Hemo Tracker is a record-keeping tool. It does not diagnose conditions or give medical advice. Check laboratory results with a qualified health professional.

The unsigned V1 application is intended for local use. Keep the device, passphrase, recovery key, backups, and exported files secure. A decrypted export is a plaintext copy. The operating system can show an unverified-publisher warning for an unsigned build. Do not disable global operating-system protections.
