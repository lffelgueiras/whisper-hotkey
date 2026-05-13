
## Regenerating IPC types

After changing any Rust type that derives `ts_rs::TS`:

```bash
pnpm gen-types
```

Generated files land in `src/ipc/generated/`. Commit them.

## Release secrets

Required GitHub Actions secrets for `.github/workflows/release.yml`:

- `APPLE_CERTIFICATE` — base64-encoded `.p12` Developer ID Application certificate
- `APPLE_CERTIFICATE_PASSWORD` — password protecting the `.p12`
- `APPLE_SIGNING_IDENTITY` — e.g. `Developer ID Application: Your Name (TEAMID)`
- `APPLE_ID` — Apple ID used for notarization
- `APPLE_PASSWORD` — app-specific password for the Apple ID
- `APPLE_TEAM_ID` — 10-character team identifier

To generate the macOS certificate:

1. Create a Developer ID Application certificate in Apple Developer portal.
2. Export the resulting cert from Keychain Access as `.p12`.
3. Encode with `base64 -i cert.p12 | pbcopy` and paste into the `APPLE_CERTIFICATE` secret.
