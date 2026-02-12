# Support Matrix

Last updated: February 12, 2026

## Platforms

| Platform | `pf_lsp` Release Binary | VS Code VSIX | CI Coverage | Status |
| --- | --- | --- | --- | --- |
| Linux x64 | Yes | Yes (`linux-x64`) | Yes | Supported |
| macOS x64 | Yes | Yes (`darwin-x64`) | Yes | Supported |
| Windows x64 | No (temporarily disabled) | No (temporarily disabled) | Limited | Paused |

## Policy Notes

- Windows release artifacts are paused until dedicated Windows smoke checks and rollback coverage are in place.
- Supported platforms must pass CI quality gates and release smoke checks.
- Any support status change must update this file, `README.md`, and `CHANGELOG.md` in the same pull request.

## Windows Re-Enable Criteria

Windows support may be restored only when all criteria below are met:

1. A `windows-latest` release job for `pf_lsp` builds and uploads artifacts successfully for two consecutive runs.
2. A `windows-latest` VSIX build succeeds and contains `extension/pf_lsp.exe`.
3. A Windows smoke check validates `pf_lsp.exe` startup behavior in CI.
4. The rollback runbook includes explicit Windows asset verification and tag rollback steps.
5. At least one real tagged release (`v*`) ships Windows artifacts with valid checksums.

## Windows Smoke Plan

1. Re-introduce Windows entries in release matrices behind a temporary feature flag branch.
2. Add smoke check for `target/release/pf_lsp.exe` analogous to Linux/macOS startup smoke test.
3. Add VSIX smoke assertion for `extension/pf_lsp.exe`.
4. Run manual `workflow_dispatch` release validation.
5. After two green dry-runs, remove pause status and update this matrix to Supported.
