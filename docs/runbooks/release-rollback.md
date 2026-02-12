# Release Rollback Runbook

Last updated: February 12, 2026

## Scope

Use this runbook when a freshly published tag release (`v*`) is broken, incomplete, or contains invalid assets.

## Triggers

- Release workflow failed after uploading partial assets.
- Smoke checks detect invalid `pf_lsp` binary or VSIX package.
- Post-release validation reports a critical regression.

## Immediate Actions

1. Freeze new release tags and communicate incident in the team channel.
2. Identify affected tag and workflow run URL.
3. Confirm impact:
   - missing assets
   - invalid checksum bundle
   - runtime startup failure

## Rollback Procedure

1. Delete the GitHub Release for the affected tag.
2. Delete the local and remote git tag.
3. Revert or fix the offending commit(s) on `main`.
4. Re-run CI and CodeQL on `main`.
5. Create a new patch tag and publish a clean release.

## Commands

```bash
# Replace vX.Y.Z with the broken tag.
gh release delete vX.Y.Z --yes
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
```

## Validation Before Re-Tag

- `CI` is green on `main`.
- `CodeQL` is green on `main`.
- Release smoke checks pass for all supported platforms.
- `SHA256SUMS.txt` includes all expected assets.

## Post-Incident

1. Add a short incident note to `CHANGELOG.md` under `[Unreleased]`.
2. Add or tighten smoke checks that would have caught the issue earlier.
3. Record follow-up actions in the next milestone plan.
